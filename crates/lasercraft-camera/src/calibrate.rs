//! Fitting a lens model to pictures of the calibration sheet
//! (`docs/spec/camera.md`, "Calibration", fitting): Zhang's homographies
//! and closed-form first guess, then Levenberg–Marquardt on every
//! parameter.
//!
//! Pictures of a flat sheet lying on the bed, all seen from straight
//! above, cannot tell the focal length from the camera's height. Weak
//! priors keep the focal length, square pixels and the optical center
//! near plausible values in that case; with tilted views the pictures
//! decide.

use lasercraft_geom::Point;

use crate::CameraError;
use crate::homography::Homography;
use crate::lens::{Lens, Model};
use crate::linalg::{gram, smallest_eigenvector, solve};
use crate::synth::rodrigues;

/// Fewest pictures a calibration takes.
pub const MIN_VIEWS: usize = 3;
/// Largest acceptable reprojection error, pixels.
pub const MAX_RMS: f64 = 5.0;

/// Sheet point (mm) and its pixel, for one picture.
pub type View = Vec<(Point, Point)>;

fn rotation_vector(r: &[f64; 9]) -> [f64; 3] {
    let tr = r[0] + r[4] + r[8];
    let cos = ((tr - 1.0) / 2.0).clamp(-1.0, 1.0);
    let theta = cos.acos();
    if theta < 1e-12 {
        return [0.0; 3];
    }
    let s = 2.0 * theta.sin();
    if s.abs() < 1e-9 {
        // Near 180°: take the axis from the diagonal.
        let axis = [
            ((r[0] + 1.0) / 2.0).max(0.0).sqrt(),
            ((r[4] + 1.0) / 2.0).max(0.0).sqrt().copysign(r[1]),
            ((r[8] + 1.0) / 2.0).max(0.0).sqrt().copysign(r[2]),
        ];
        return axis.map(|a| a * theta);
    }
    [
        (r[7] - r[5]) / s * theta,
        (r[2] - r[6]) / s * theta,
        (r[3] - r[1]) / s * theta,
    ]
}

/// Closed-form intrinsics from homographies (Zhang, section 3.1), zero
/// skew. `None` when the views do not decide them.
fn zhang(hs: &[Homography]) -> Option<(f64, f64, f64, f64)> {
    if hs.len() < 3 {
        return None;
    }
    let v = |h: &Homography, i: usize, j: usize| {
        let col = |k: usize| [h.m[k], h.m[3 + k], h.m[6 + k]];
        let (a, b) = (col(i), col(j));
        vec![
            a[0] * b[0],
            a[0] * b[1] + a[1] * b[0],
            a[1] * b[1],
            a[2] * b[0] + a[0] * b[2],
            a[2] * b[1] + a[1] * b[2],
            a[2] * b[2],
        ]
    };
    let mut rows = Vec::new();
    for h in hs {
        rows.push(v(h, 0, 1));
        let (a, b) = (v(h, 0, 0), v(h, 1, 1));
        rows.push(a.iter().zip(&b).map(|(x, y)| x - y).collect());
    }
    // Zero skew: B12 = 0.
    rows.push(vec![0.0, 1000.0, 0.0, 0.0, 0.0, 0.0]);
    let b = smallest_eigenvector(gram(&rows, 6));
    let (b11, b12, b22, b13, b23, b33) = (b[0], b[1], b[2], b[3], b[4], b[5]);
    let d = b11 * b22 - b12 * b12;
    if d.abs() < 1e-300 || b11.abs() < 1e-300 {
        return None;
    }
    let v0 = (b12 * b13 - b11 * b23) / d;
    let lambda = b33 - (b13 * b13 + v0 * (b12 * b13 - b11 * b23)) / b11;
    let fx2 = lambda / b11;
    let fy2 = lambda * b11 / d;
    if !(fx2 > 0.0 && fy2 > 0.0) {
        return None;
    }
    let (fx, fy) = (fx2.sqrt(), fy2.sqrt());
    let u0 = -b13 * fx * fx / lambda;
    Some((fx, fy, u0, v0))
}

/// A view's pose from its homography and the intrinsics: Rodrigues vector
/// and translation.
fn pose(h: &Homography, fx: f64, fy: f64, cx: f64, cy: f64) -> Option<([f64; 3], [f64; 3])> {
    // K⁻¹ · H, column by column.
    let kinv = |c: [f64; 3]| [(c[0] - cx * c[2]) / fx, (c[1] - cy * c[2]) / fy, c[2]];
    let col = |k: usize| kinv([h.m[k], h.m[3 + k], h.m[6 + k]]);
    let (h1, h2, h3) = (col(0), col(1), col(2));
    let norm = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let mut l = 1.0 / norm(h1).max(1e-300);
    // The sheet is in front of the camera.
    if h3[2] * l < 0.0 {
        l = -l;
    }
    let r1 = h1.map(|v| v * l);
    let mut r2 = h2.map(|v| v * l);
    let dot = r1[0] * r2[0] + r1[1] * r2[1] + r1[2] * r2[2];
    r2 = [
        r2[0] - dot * r1[0],
        r2[1] - dot * r1[1],
        r2[2] - dot * r1[2],
    ];
    let n2 = norm(r2);
    if n2 < 1e-12 {
        return None;
    }
    r2 = r2.map(|v| v / n2);
    let r3 = [
        r1[1] * r2[2] - r1[2] * r2[1],
        r1[2] * r2[0] - r1[0] * r2[2],
        r1[0] * r2[1] - r1[1] * r2[0],
    ];
    let r = [
        r1[0], r2[0], r3[0], r1[1], r2[1], r3[1], r1[2], r2[2], r3[2],
    ];
    Some((rotation_vector(&r), h3.map(|v| v * l)))
}

/// The parameter vector: intrinsics, distortion, then six numbers per view.
struct Problem<'a> {
    model: Model,
    views: &'a [View],
    width: f64,
    height: f64,
    f0: f64,
}

impl Problem<'_> {
    fn intrinsic_count(&self) -> usize {
        let (nk, np) = self.model.terms();
        4 + nk + np
    }

    fn lens(&self, p: &[f64]) -> Lens {
        let (nk, np) = self.model.terms();
        Lens {
            model: self.model,
            width: self.width as u32,
            height: self.height as u32,
            fx: p[0],
            fy: p[1],
            cx: p[2],
            cy: p[3],
            k: p[4..4 + nk].to_vec(),
            p: p[4 + nk..4 + nk + np].to_vec(),
            rms: 0.0,
            created: String::new(),
        }
    }

    /// Picture residuals, then the priors.
    fn residuals(&self, p: &[f64], out: &mut Vec<f64>) {
        out.clear();
        let lens = self.lens(p);
        let base = self.intrinsic_count();
        for (v, view) in self.views.iter().enumerate() {
            let q = &p[base + v * 6..base + v * 6 + 6];
            let r = rodrigues([q[0], q[1], q[2]]);
            for (obj, px) in view {
                let c = [
                    r[0] * obj.x + r[1] * obj.y + q[3],
                    r[3] * obj.x + r[4] * obj.y + q[4],
                    r[6] * obj.x + r[7] * obj.y + q[5],
                ];
                let z = if c[2].abs() < 1e-9 { 1e-9 } else { c[2] };
                let proj = lens.project(Point::new(c[0] / z, c[1] / z));
                out.push(proj.x - px.x);
                out.push(proj.y - px.y);
            }
        }
        // Weak priors (pixels of error per relative change).
        out.push(0.5 * (p[0] - self.f0) / self.f0);
        out.push(50.0 * (p[0] - p[1]) / self.f0);
        out.push(0.5 * (p[2] - self.width / 2.0) / self.width);
        out.push(0.5 * (p[3] - self.height / 2.0) / self.height);
    }

    fn data_rms(&self, p: &[f64]) -> f64 {
        let mut r = Vec::new();
        self.residuals(p, &mut r);
        let n = self.views.iter().map(Vec::len).sum::<usize>();
        (r[..2 * n].iter().map(|v| v * v).sum::<f64>() / n as f64).sqrt()
    }
}

fn cost(r: &[f64]) -> f64 {
    r.iter().map(|v| v * v).sum()
}

/// Levenberg–Marquardt with a forward-difference Jacobian.
fn minimize(problem: &Problem<'_>, mut p: Vec<f64>) -> Vec<f64> {
    let n = p.len();
    let mut r = Vec::new();
    problem.residuals(&p, &mut r);
    let mut current = cost(&r);
    let mut lambda = 1e-3;
    let mut trial = Vec::new();
    for _ in 0..200 {
        let m = r.len();
        let mut jac = vec![vec![0.0; n]; m];
        for k in 0..n {
            let h = 1e-6 * p[k].abs().max(1e-3);
            let mut q = p.clone();
            q[k] += h;
            problem.residuals(&q, &mut trial);
            for i in 0..m {
                jac[i][k] = (trial[i] - r[i]) / h;
            }
        }
        let mut a = vec![vec![0.0; n]; n];
        let mut g = vec![0.0; n];
        for (row, ri) in jac.iter().zip(&r) {
            for i in 0..n {
                if row[i] == 0.0 {
                    continue;
                }
                g[i] -= row[i] * ri;
                for j in 0..n {
                    a[i][j] += row[i] * row[j];
                }
            }
        }
        let mut improved = false;
        for _ in 0..12 {
            let mut damped = a.clone();
            for i in 0..n {
                damped[i][i] += lambda * a[i][i].max(1e-9);
            }
            let Some(step) = solve(damped, g.clone()) else {
                lambda *= 10.0;
                continue;
            };
            let q: Vec<f64> = p.iter().zip(&step).map(|(a, b)| a + b).collect();
            problem.residuals(&q, &mut trial);
            let c = cost(&trial);
            if c.is_finite() && c < current {
                let gain = (current - c) / current.max(1e-300);
                p = q;
                std::mem::swap(&mut r, &mut trial);
                current = c;
                lambda = (lambda / 3.0).max(1e-12);
                improved = gain > 1e-12;
                break;
            }
            lambda *= 4.0;
        }
        if !improved {
            break;
        }
    }
    p
}

/// Fits `model` to pictures of `width × height` pixels.
///
/// # Errors
///
/// Too few pictures, a picture whose dots do not fit a plane, or a fit
/// worse than [`MAX_RMS`].
pub fn calibrate(
    model: Model,
    width: u32,
    height: u32,
    views: &[View],
) -> Result<Lens, CameraError> {
    if views.len() < MIN_VIEWS {
        return Err(CameraError::new(format!(
            "{} usable picture(s); at least {MIN_VIEWS} are needed",
            views.len()
        )));
    }
    let hs = views
        .iter()
        .map(|v| {
            let (from, to): (Vec<Point>, Vec<Point>) = v.iter().copied().unzip();
            Homography::fit(&from, &to)
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| CameraError::new("a picture's dots do not fit a flat sheet"))?;
    let (w, h) = (f64::from(width), f64::from(height));
    let f0 = w.max(h);
    let (fx, fy, cx, cy) = zhang(&hs)
        .filter(|(fx, fy, cx, cy)| {
            (0.2 * f0..5.0 * f0).contains(fx)
                && (fx / fy - 1.0).abs() < 0.2
                && (0.0..w).contains(cx)
                && (0.0..h).contains(cy)
        })
        .unwrap_or((f0, f0, w / 2.0, h / 2.0));
    let (nk, np) = model.terms();
    let mut p = vec![fx, fy, cx, cy];
    p.extend(std::iter::repeat_n(0.0, nk + np));
    for hm in &hs {
        let (rv, t) = pose(hm, fx, fy, cx, cy)
            .ok_or_else(|| CameraError::new("a picture's sheet is seen edge-on"))?;
        p.extend(rv);
        p.extend(t);
    }
    let problem = Problem {
        model,
        views,
        width: w,
        height: h,
        f0: (fx + fy) / 2.0,
    };
    let p = minimize(&problem, p);
    let mut lens = problem.lens(&p);
    lens.rms = problem.data_rms(&p);
    lens.validate()?;
    if lens.rms > MAX_RMS {
        return Err(CameraError::new(format!(
            "the {} model fits poorly ({:.1} pixels off on average); take the pictures again",
            model.title().to_lowercase(),
            lens.rms
        )));
    }
    Ok(lens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::Pose;

    /// Ideal dot positions of the sheet in a picture, no detection.
    fn view(lens: &Lens, pose: &Pose, offset: Point, angle: f64) -> View {
        let mut out = Vec::new();
        for j in 0..crate::sheet::ROWS {
            for i in 0..crate::sheet::COLUMNS {
                let local = crate::sheet::point(i, j);
                let (s, c) = angle.sin_cos();
                let world =
                    offset + Point::new(c * local.x - s * local.y, s * local.x + c * local.y);
                if let Some(ray) = pose.ray(world) {
                    let px = lens.project(ray);
                    if (0.0..f64::from(lens.width)).contains(&px.x)
                        && (0.0..f64::from(lens.height)).contains(&px.y)
                    {
                        out.push((local, px));
                    }
                }
            }
        }
        out
    }

    fn truth() -> Lens {
        let mut lens = Lens::ideal(Model::Standard, 800, 600, 640.0);
        lens.cx = 405.0;
        lens.cy = 296.0;
        lens.k = vec![-0.28, 0.09];
        lens.p = vec![0.0008, -0.0004];
        lens
    }

    #[test]
    fn tilted_views_recover_the_true_lens() {
        let truth = truth();
        let views: Vec<View> = [
            ([0.25, 0.0, 0.0], Point::new(60.0, 40.0), 0.0),
            ([0.0, 0.3, 0.1], Point::new(100.0, 70.0), 0.2),
            ([-0.2, 0.15, -0.1], Point::new(40.0, 60.0), -0.3),
            ([0.1, -0.25, 0.0], Point::new(80.0, 30.0), 0.1),
            ([0.0, 0.0, 0.0], Point::new(70.0, 50.0), 0.0),
        ]
        .iter()
        .map(|(tilt, at, angle)| {
            view(
                &truth,
                &Pose::overhead(150.0, 110.0, 380.0, *tilt),
                *at,
                *angle,
            )
        })
        .collect();
        let lens = calibrate(Model::Standard, 800, 600, &views).unwrap();
        assert!(lens.rms < 0.02, "{}", lens.rms);
        assert!((lens.fx - truth.fx).abs() < 2.0, "{lens:?}");
        assert!((lens.k[0] - truth.k[0]).abs() < 0.01, "{lens:?}");
        // Corrected pixels match the truth where the sheet was seen.
        for (u, v) in [
            (200.0, 150.0),
            (620.0, 160.0),
            (400.0, 300.0),
            (220.0, 450.0),
        ] {
            let a = truth.unproject(Point::new(u, v));
            let b = lens.unproject(Point::new(u, v));
            assert!((a - b).length() < 1e-3, "{a:?} {b:?}");
        }
    }

    #[test]
    fn flat_views_correct_straight_lines_and_bad_input_is_refused() {
        let truth = truth();
        let pose = Pose::overhead(150.0, 110.0, 380.0, [0.0; 3]);
        let views: Vec<View> = [
            (Point::new(-30.0, -40.0), 0.1),
            (Point::new(170.0, -40.0), -0.1),
            (Point::new(170.0, 140.0), 0.05),
            (Point::new(-30.0, 140.0), 0.0),
            (Point::new(70.0, 50.0), 0.2),
        ]
        .iter()
        .map(|(at, angle)| view(&truth, &pose, *at, *angle))
        .collect();
        for model in [Model::Standard, Model::Wide] {
            let lens = calibrate(model, 800, 600, &views).unwrap();
            assert!(lens.rms < 0.05, "{model:?} {}", lens.rms);
            // The degenerate scale is fixed by the priors, but a straight
            // line on the bed stays straight once corrected.
            let line: Vec<Point> = (0..=10)
                .map(|i| {
                    let world = Point::new(20.0 + f64::from(i) * 26.0, 15.0);
                    lens.correct(truth.project(pose.ray(world).unwrap()))
                })
                .collect();
            let (a, b) = (line[0], line[10]);
            for p in &line {
                let d =
                    ((b.x - a.x) * (a.y - p.y) - (a.x - p.x) * (b.y - a.y)).abs() / a.distance(b);
                assert!(d < 0.1, "{model:?} bends by {d}");
            }
        }
        assert!(calibrate(Model::Standard, 800, 600, &views[..2]).is_err());
    }
}
