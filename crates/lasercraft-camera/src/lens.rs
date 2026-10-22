//! The lens model (`docs/spec/camera.md`, "Lens correction"): a pinhole
//! camera with Brown–Conrady or Kannala–Brandt distortion.
//!
//! Picture pixels `(u, v)` have their origin at the top-left corner of the
//! picture, pixel centers at `i + 0.5`. A **ray** `(x, y)` is a direction
//! from the camera, `X / Z` and `Y / Z`, with x right and y down.

use lasercraft_geom::Point;
use serde::{Deserialize, Serialize};

/// The distortion model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Model {
    /// Radial `k1, k2` and tangential `p1, p2`.
    Standard,
    /// Radial `k1, k2, k3` and tangential `p1, p2`.
    Wide,
    /// Equidistant fish-eye with `k1 … k4`.
    Fisheye,
}

impl Model {
    /// Every model, in the wizard's order.
    pub const ALL: [Self; 3] = [Self::Standard, Self::Wide, Self::Fisheye];

    /// Radial and tangential term counts.
    #[must_use]
    pub const fn terms(self) -> (usize, usize) {
        match self {
            Self::Standard => (2, 2),
            Self::Wide => (3, 2),
            Self::Fisheye => (4, 0),
        }
    }

    /// The command spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Wide => "wide",
            Self::Fisheye => "fisheye",
        }
    }

    /// The wizard's name.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Wide => "Wide angle",
            Self::Fisheye => "Fish-eye",
        }
    }

    /// From the command spelling.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s)
    }
}

/// A lens correction for pictures of one size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lens {
    /// Distortion model.
    pub model: Model,
    /// Picture width the correction was made for, pixels.
    pub width: u32,
    /// Picture height, pixels.
    pub height: u32,
    /// Focal length in pixels, x.
    pub fx: f64,
    /// Focal length in pixels, y.
    pub fy: f64,
    /// Optical center, x pixels.
    pub cx: f64,
    /// Optical center, y pixels.
    pub cy: f64,
    /// Radial terms.
    pub k: Vec<f64>,
    /// Tangential terms.
    pub p: Vec<f64>,
    /// Reprojection error of the calibration, pixels.
    #[serde(default)]
    pub rms: f64,
    /// When it was made (RFC 3339), which alignments refer to.
    #[serde(default)]
    pub created: String,
}

impl Lens {
    /// A lens without distortion, the given focal length, centered.
    #[must_use]
    pub fn ideal(model: Model, width: u32, height: u32, focal: f64) -> Self {
        let (nk, np) = model.terms();
        Self {
            model,
            width,
            height,
            fx: focal,
            fy: focal,
            cx: f64::from(width) / 2.0,
            cy: f64::from(height) / 2.0,
            k: vec![0.0; nk],
            p: vec![0.0; np],
            rms: 0.0,
            created: String::new(),
        }
    }

    /// The fit score shown in the wizard: `100 / (1 + rms)`.
    #[must_use]
    pub fn score(&self) -> f64 {
        100.0 / (1.0 + self.rms)
    }

    /// The distorted ray of an ideal ray.
    #[must_use]
    pub fn distort(&self, ray: Point) -> Point {
        let (x, y) = (ray.x, ray.y);
        let k = |i: usize| self.k.get(i).copied().unwrap_or(0.0);
        let p = |i: usize| self.p.get(i).copied().unwrap_or(0.0);
        match self.model {
            Model::Standard | Model::Wide => {
                let r2 = x * x + y * y;
                let radial = 1.0 + r2 * (k(0) + r2 * (k(1) + r2 * k(2)));
                Point::new(
                    x * radial + 2.0 * p(0) * x * y + p(1) * (r2 + 2.0 * x * x),
                    y * radial + p(0) * (r2 + 2.0 * y * y) + 2.0 * p(1) * x * y,
                )
            }
            Model::Fisheye => {
                let r = (x * x + y * y).sqrt();
                if r < 1e-12 {
                    return ray;
                }
                let t = r.atan();
                let t2 = t * t;
                let td = t * (1.0 + t2 * (k(0) + t2 * (k(1) + t2 * (k(2) + t2 * k(3)))));
                Point::new(x * td / r, y * td / r)
            }
        }
    }

    /// The raw picture pixel an ideal ray lands on.
    #[must_use]
    pub fn project(&self, ray: Point) -> Point {
        let d = self.distort(ray);
        Point::new(self.fx * d.x + self.cx, self.fy * d.y + self.cy)
    }

    /// The ideal ray of a raw picture pixel (Newton steps on `distort`).
    #[must_use]
    pub fn unproject(&self, pixel: Point) -> Point {
        let target = Point::new((pixel.x - self.cx) / self.fx, (pixel.y - self.cy) / self.fy);
        let mut ray = target;
        for _ in 0..30 {
            let d = self.distort(ray);
            let (ex, ey) = (d.x - target.x, d.y - target.y);
            if ex * ex + ey * ey < 1e-24 {
                break;
            }
            let h = 1e-7;
            let dx = self.distort(Point::new(ray.x + h, ray.y));
            let dy = self.distort(Point::new(ray.x, ray.y + h));
            let (a, b) = ((dx.x - d.x) / h, (dy.x - d.x) / h);
            let (c, e) = ((dx.y - d.y) / h, (dy.y - d.y) / h);
            let det = a * e - b * c;
            if det.abs() < 1e-12 {
                break;
            }
            ray = Point::new(
                ray.x - (e * ex - b * ey) / det,
                ray.y - (-c * ex + a * ey) / det,
            );
        }
        ray
    }

    /// The lens-corrected pixel of a raw pixel: the ideal ray seen through
    /// the same focal length and center.
    #[must_use]
    pub fn correct(&self, raw: Point) -> Point {
        let r = self.unproject(raw);
        Point::new(self.fx * r.x + self.cx, self.fy * r.y + self.cy)
    }

    /// The raw pixel of a lens-corrected pixel.
    #[must_use]
    pub fn uncorrect(&self, corrected: Point) -> Point {
        self.project(Point::new(
            (corrected.x - self.cx) / self.fx,
            (corrected.y - self.cy) / self.fy,
        ))
    }

    /// The same lens for pictures of another size.
    #[must_use]
    pub fn for_size(&self, width: u32, height: u32) -> Self {
        let sx = f64::from(width) / f64::from(self.width);
        let sy = f64::from(height) / f64::from(self.height);
        Self {
            width,
            height,
            fx: self.fx * sx,
            fy: self.fy * sy,
            cx: self.cx * sx,
            cy: self.cy * sy,
            ..self.clone()
        }
    }

    /// Checks the numbers (for files).
    ///
    /// # Errors
    ///
    /// Wrong term counts, a size of zero or a value that is not finite.
    pub fn validate(&self) -> Result<(), crate::CameraError> {
        let (nk, np) = self.model.terms();
        let finite = [self.fx, self.fy, self.cx, self.cy, self.rms]
            .iter()
            .chain(&self.k)
            .chain(&self.p)
            .all(|v| v.is_finite());
        if self.k.len() != nk || self.p.len() != np {
            return Err(crate::CameraError::new(format!(
                "a {} lens correction has {nk} radial and {np} tangential terms",
                self.model.as_str()
            )));
        }
        if !finite || self.fx <= 0.0 || self.fy <= 0.0 || self.width == 0 || self.height == 0 {
            return Err(crate::CameraError::new(
                "the lens correction has a bad number",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unproject_inverts_project_for_every_model() {
        for model in Model::ALL {
            let mut lens = Lens::ideal(model, 640, 480, 500.0);
            lens.k = match model {
                Model::Standard => vec![-0.25, 0.08],
                Model::Wide => vec![-0.3, 0.1, -0.02],
                Model::Fisheye => vec![0.05, -0.01, 0.0, 0.0],
            };
            if !lens.p.is_empty() {
                lens.p = vec![0.001, -0.0005];
            }
            lens.validate().unwrap();
            for (u, v) in [(10.0, 15.0), (320.0, 240.0), (600.0, 400.0), (100.0, 470.0)] {
                let pixel = Point::new(u, v);
                let back = lens.project(lens.unproject(pixel));
                assert!(back.distance(pixel) < 1e-6, "{model:?} {pixel:?} {back:?}");
                let c = lens.correct(pixel);
                assert!(lens.uncorrect(c).distance(pixel) < 1e-6);
            }
        }
        let lens = Lens::ideal(Model::Standard, 640, 480, 500.0);
        let half = lens.for_size(320, 240);
        assert!((half.fx - 250.0).abs() < 1e-12 && (half.cx - 160.0).abs() < 1e-12);
        assert_eq!(Model::parse("wide"), Some(Model::Wide));
        let mut bad = lens.clone();
        bad.k.push(0.0);
        assert!(bad.validate().is_err());
    }
}
