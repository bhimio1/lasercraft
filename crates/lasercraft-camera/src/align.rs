//! Bed alignments (`docs/spec/camera.md`, "Bed alignment"): the map from
//! lens-corrected picture pixels to bed millimetres.

use lasercraft_geom::Point;
use serde::{Deserialize, Serialize};

use crate::CameraError;
use crate::homography::Homography;
use crate::lens::Lens;

/// One stored alignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alignment {
    /// Shown in lists.
    pub name: String,
    /// Material height it was made at, mm.
    #[serde(default)]
    pub height: f64,
    /// Corrected picture pixels to document mm.
    pub matrix: Homography,
    /// `created` of the lens correction it was made with.
    #[serde(default)]
    pub lens: String,
    /// Picture size it was made with, pixels.
    pub width: u32,
    /// Picture height, pixels.
    pub height_px: u32,
    /// When it was made (RFC 3339).
    #[serde(default)]
    pub created: String,
    /// Where the head was when the picture was taken (head cameras),
    /// document mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<Point>,
}

/// Twice the signed area of a polygon (positive: clockwise on screen).
fn signed_area(p: &[Point]) -> f64 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a.x * b.y - b.x * a.y
        })
        .sum()
}

fn convex(p: &[Point]) -> bool {
    let n = p.len();
    let turns: Vec<f64> = (0..n)
        .map(|i| {
            let (a, b, c) = (p[i], p[(i + 1) % n], p[(i + 2) % n]);
            (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
        })
        .collect();
    turns.iter().all(|t| *t > 0.0) || turns.iter().all(|t| *t < 0.0)
}

/// The map from the four raw picture points (seen through `lens`) to the
/// four bed points.
///
/// # Errors
///
/// Points that are not a convex quadrilateral turning the same way as the
/// bed points.
pub fn fit(lens: &Lens, raw: &[Point; 4], bed: &[Point; 4]) -> Result<Homography, CameraError> {
    let corrected: Vec<Point> = raw.iter().map(|p| lens.correct(*p)).collect();
    if !convex(&corrected) || signed_area(&corrected).signum() != signed_area(bed).signum() {
        return Err(CameraError::new(
            "The marks are not in order. Click them clockwise from the top left.",
        ));
    }
    Homography::fit(&corrected, bed)
        .ok_or_else(|| CameraError::new("the marks do not make a usable alignment"))
}

impl Alignment {
    /// The bed point of a raw picture pixel.
    #[must_use]
    pub fn to_bed(&self, lens: &Lens, raw: Point) -> Option<Point> {
        self.matrix.apply(lens.correct(raw))
    }

    /// The raw picture pixel of a bed point.
    #[must_use]
    pub fn to_picture(&self, lens: &Lens, bed: Point) -> Option<Point> {
        let inverse = self.matrix.inverse()?;
        Some(lens.uncorrect(inverse.apply(bed)?))
    }

    /// The alignment for a picture taken with the head at `head`: shifted
    /// by `head` minus where the head was for the alignment. Unchanged for
    /// an alignment without a head position.
    #[must_use]
    pub fn at_head(&self, head: Point) -> Self {
        let Some(from) = self.head else {
            return self.clone();
        };
        let (dx, dy) = (head.x - from.x, head.y - from.y);
        let shift = Homography {
            m: [1.0, 0.0, dx, 0.0, 1.0, dy, 0.0, 0.0, 1.0],
        };
        Self {
            matrix: shift.then(&self.matrix),
            head: Some(head),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::Model;
    use crate::synth::Pose;

    #[test]
    fn four_marks_map_the_picture_onto_the_bed() {
        let mut lens = Lens::ideal(Model::Standard, 640, 480, 600.0);
        lens.k = vec![-0.15, 0.02];
        let pose = Pose::overhead(150.0, 100.0, 380.0, [0.05, -0.02, 0.1]);
        let bed = [
            Point::new(45.0, 30.0),
            Point::new(255.0, 30.0),
            Point::new(255.0, 170.0),
            Point::new(45.0, 170.0),
        ];
        let raw = bed.map(|p| lens.project(pose.ray(p).unwrap()));
        let matrix = fit(&lens, &raw, &bed).unwrap();
        let a = Alignment {
            name: "test".into(),
            height: 0.0,
            matrix,
            lens: String::new(),
            width: 640,
            height_px: 480,
            created: String::new(),
            head: None,
        };
        // Any bed point, mapped through the picture and back.
        for p in [
            Point::new(150.0, 100.0),
            Point::new(10.0, 10.0),
            Point::new(290.0, 190.0),
        ] {
            let px = lens.project(pose.ray(p).unwrap());
            assert!(a.to_bed(&lens, px).unwrap().distance(p) < 1e-6);
            assert!(a.to_picture(&lens, p).unwrap().distance(px) < 1e-6);
        }
        let swapped = [raw[1], raw[0], raw[2], raw[3]];
        assert!(fit(&lens, &swapped, &bed).is_err());
        // A head camera's picture moves with the head.
        assert_eq!(
            a.at_head(Point::new(9.0, 9.0)),
            a,
            "no head position: unchanged"
        );
        let made = Alignment {
            head: Some(Point::new(100.0, 50.0)),
            ..a
        };
        let moved = made.at_head(Point::new(130.0, 40.0));
        let px = lens.project(pose.ray(Point::new(150.0, 100.0)).unwrap());
        let seen = moved.to_bed(&lens, px).unwrap();
        assert!(seen.distance(Point::new(180.0, 90.0)) < 1e-6, "{seen:?}");
        assert_eq!(moved.head, Some(Point::new(130.0, 40.0)));
    }
}
