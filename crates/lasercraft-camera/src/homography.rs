//! Plane-to-plane projective maps (Hartley and Zisserman, ch. 4: the
//! normalized direct linear transform).

use lasercraft_geom::Point;
use serde::{Deserialize, Serialize};

use crate::linalg::{gram, smallest_eigenvector};

/// A 3 × 3 projective map, row-major, normalized so `m[8] = 1` when
/// possible.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Homography {
    /// The nine entries, row by row.
    pub m: [f64; 9],
}

impl Homography {
    /// The map that changes nothing.
    pub const IDENTITY: Self = Self {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Maps a point; `None` at the horizon.
    #[must_use]
    pub fn apply(&self, p: Point) -> Option<Point> {
        let m = &self.m;
        let w = m[6] * p.x + m[7] * p.y + m[8];
        if w.abs() < 1e-12 {
            return None;
        }
        Some(Point::new(
            (m[0] * p.x + m[1] * p.y + m[2]) / w,
            (m[3] * p.x + m[4] * p.y + m[5]) / w,
        ))
    }

    /// The inverse map; `None` when singular.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let m = &self.m;
        let c = [
            m[4] * m[8] - m[5] * m[7],
            m[2] * m[7] - m[1] * m[8],
            m[1] * m[5] - m[2] * m[4],
            m[5] * m[6] - m[3] * m[8],
            m[0] * m[8] - m[2] * m[6],
            m[2] * m[3] - m[0] * m[5],
            m[3] * m[7] - m[4] * m[6],
            m[1] * m[6] - m[0] * m[7],
            m[0] * m[4] - m[1] * m[3],
        ];
        let det = m[0] * c[0] + m[1] * c[3] + m[2] * c[6];
        if det.abs() < 1e-18 {
            return None;
        }
        Some(Self::normalized(c.map(|v| v / det)))
    }

    /// `self` after `other`: `self(other(p))`.
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        let (a, b) = (&other.m, &self.m);
        let mut m = [0.0; 9];
        for r in 0..3 {
            for c in 0..3 {
                m[r * 3 + c] = (0..3).map(|k| b[r * 3 + k] * a[k * 3 + c]).sum();
            }
        }
        Self::normalized(m)
    }

    fn normalized(m: [f64; 9]) -> Self {
        let s = if m[8].abs() > 1e-12 { m[8] } else { 1.0 };
        Self {
            m: m.map(|v| v / s),
        }
    }

    /// The map from `from[i]` to `to[i]`, least squares for more than four
    /// pairs. `None` with fewer than four pairs or a degenerate set.
    #[must_use]
    pub fn fit(from: &[Point], to: &[Point]) -> Option<Self> {
        if from.len() < 4 || from.len() != to.len() {
            return None;
        }
        let (nf, tf) = normalizer(from)?;
        let (nt, tt) = normalizer(to)?;
        let mut rows = Vec::with_capacity(from.len() * 2);
        for (p, q) in nf.iter().zip(&nt) {
            let (x, y, u, v) = (p.x, p.y, q.x, q.y);
            rows.push(vec![-x, -y, -1.0, 0.0, 0.0, 0.0, u * x, u * y, u]);
            rows.push(vec![0.0, 0.0, 0.0, -x, -y, -1.0, v * x, v * y, v]);
        }
        let h = smallest_eigenvector(gram(&rows, 9));
        let mut m = [0.0; 9];
        m.copy_from_slice(&h);
        let normalized = Self { m };
        let back = tt.inverse()?;
        let out = back.then(&normalized.then(&tf));
        out.m.iter().all(|v| v.is_finite()).then_some(out)
    }
}

/// Points moved to their centroid and scaled to mean distance √2, with the
/// map that does it.
fn normalizer(points: &[Point]) -> Option<(Vec<Point>, Homography)> {
    let n = points.len() as f64;
    let cx = points.iter().map(|p| p.x).sum::<f64>() / n;
    let cy = points.iter().map(|p| p.y).sum::<f64>() / n;
    let mean = points
        .iter()
        .map(|p| ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt())
        .sum::<f64>()
        / n;
    if mean < 1e-12 {
        return None;
    }
    let s = std::f64::consts::SQRT_2 / mean;
    let t = Homography {
        m: [s, 0.0, -s * cx, 0.0, s, -s * cy, 0.0, 0.0, 1.0],
    };
    Some((
        points
            .iter()
            .map(|p| Point::new(s * (p.x - cx), s * (p.y - cy)))
            .collect(),
        t,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_points_give_an_exact_map_and_its_inverse() {
        let truth = Homography {
            m: [1.2, 0.1, 30.0, -0.05, 0.9, 12.0, 0.0004, -0.0002, 1.0],
        };
        let from = [
            Point::new(0.0, 0.0),
            Point::new(300.0, 0.0),
            Point::new(300.0, 200.0),
            Point::new(0.0, 200.0),
            Point::new(150.0, 90.0),
        ];
        let to: Vec<Point> = from.iter().map(|p| truth.apply(*p).unwrap()).collect();
        let h = Homography::fit(&from[..4], &to[..4]).unwrap();
        for (p, q) in from.iter().zip(&to) {
            assert!(h.apply(*p).unwrap().distance(*q) < 1e-6);
            let back = h.inverse().unwrap().apply(*q).unwrap();
            assert!(back.distance(*p) < 1e-6);
        }
        assert!(Homography::fit(&from[..3], &to[..3]).is_none());
        let line = [Point::new(0.0, 0.0); 4];
        assert!(Homography::fit(&line, &to[..4]).is_none());
    }
}
