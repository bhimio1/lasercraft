//! Taper correction (`docs/spec/taper.md`): narrows outlines across the
//! turn direction in proportion to a tapered object's diameter.

use crate::{Point, Rect};

/// Which design edge goes on the object's top end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopEdge {
    /// The design's top edge; the length runs down the page.
    Top,
    /// The left edge; the length runs to the right.
    Left,
    /// The right edge; the length runs to the left.
    Right,
}

impl TopEdge {
    /// From the command spelling.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "top" => Some(Self::Top),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    /// The command spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

/// An object's taper and where the design sits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Taper {
    /// Which edge is at the top end.
    pub top: TopEdge,
    /// Diameter at the top end, mm.
    pub top_diameter: f64,
    /// Length from the top end to the bottom end, mm.
    pub length: f64,
    /// Diameter at the bottom end, mm.
    pub bottom_diameter: f64,
}

/// Straight runs are split at least this often along the length, mm.
pub const STEP_MM: f64 = 1.0;

impl Taper {
    /// The object's diameter `s` mm from the top end.
    #[must_use]
    pub fn diameter(&self, s: f64) -> f64 {
        self.top_diameter + (self.bottom_diameter - self.top_diameter) * s / self.length
    }

    fn widest(&self) -> f64 {
        self.top_diameter.max(self.bottom_diameter)
    }

    /// Distance from the top edge and position across, for a point of a
    /// design with bounds `b`.
    fn split(&self, p: Point, b: &Rect) -> (f64, f64) {
        match self.top {
            TopEdge::Top => (p.y - b.min.y, p.x),
            TopEdge::Left => (p.x - b.min.x, p.y),
            TopEdge::Right => (b.max.x - p.x, p.y),
        }
    }

    /// The center line across the turn direction.
    fn center(&self, b: &Rect) -> f64 {
        match self.top {
            TopEdge::Top => b.center().x,
            TopEdge::Left | TopEdge::Right => b.center().y,
        }
    }

    /// One point moved for a design with bounds `b`.
    #[must_use]
    pub fn warp_point(&self, p: Point, b: &Rect) -> Point {
        let (s, t) = self.split(p, b);
        let c = self.center(b);
        let t2 = c + (t - c) * self.diameter(s) / self.widest();
        match self.top {
            TopEdge::Top => Point::new(t2, p.y),
            TopEdge::Left | TopEdge::Right => Point::new(p.x, t2),
        }
    }

    /// An outline moved, its straight runs split every [`STEP_MM`] along
    /// the length so they bend with the taper.
    #[must_use]
    pub fn warp(&self, points: &[Point], closed: bool, b: &Rect) -> Vec<Point> {
        let mut out = Vec::with_capacity(points.len() * 2);
        let n = points.len();
        let segments = if closed { n } else { n.saturating_sub(1) };
        if let Some(&first) = points.first() {
            out.push(self.warp_point(first, b));
        }
        for i in 0..segments {
            let (a, z) = (points[i], points[(i + 1) % n]);
            let along = (self.split(z, b).0 - self.split(a, b).0).abs();
            let steps = (along / STEP_MM).ceil().max(1.0) as usize;
            for k in 1..=steps {
                let t = k as f64 / steps as f64;
                let p = Point::new(a.x + (z.x - a.x) * t, a.y + (z.y - a.y) * t);
                if closed && i + 1 == segments && k == steps {
                    break;
                }
                out.push(self.warp_point(p, b));
            }
        }
        out
    }

    /// `1 − narrowest / widest` over a design with bounds `b`, 0–1.
    #[must_use]
    pub fn correction(&self, b: &Rect) -> f64 {
        let along = match self.top {
            TopEdge::Top => b.height(),
            TopEdge::Left | TopEdge::Right => b.width(),
        };
        let narrow = self.diameter(0.0).min(self.diameter(along)).max(0.0);
        1.0 - narrow / self.widest()
    }

    /// Checks the measurements.
    ///
    /// # Errors
    ///
    /// Names the first one out of range.
    pub fn validate(&self) -> Result<(), String> {
        let check = |name: &str, v: f64, max: f64| {
            if v.is_finite() && (1.0..=max).contains(&v) {
                Ok(())
            } else {
                Err(format!("{name} must be between 1 and {max} mm, got {v}"))
            }
        };
        check("top diameter", self.top_diameter, 1000.0)?;
        check("length", self.length, 2000.0)?;
        check("bottom diameter", self.bottom_diameter, 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cup(top: TopEdge) -> Taper {
        Taper {
            top,
            top_diameter: 80.0,
            length: 100.0,
            bottom_diameter: 60.0,
        }
    }

    #[test]
    fn the_wide_end_keeps_its_width_and_the_narrow_end_shrinks() {
        let b = Rect::from_corners(Point::new(0.0, 0.0), Point::new(40.0, 100.0));
        let t = cup(TopEdge::Top);
        assert_eq!(t.warp_point(Point::new(0.0, 0.0), &b), Point::new(0.0, 0.0));
        let bottom = t.warp_point(Point::new(0.0, 100.0), &b);
        // 60/80 of the 20 mm from the center line.
        assert!((bottom.x - 5.0).abs() < 1e-9 && bottom.y == 100.0);
        assert!((t.correction(&b) - 0.25).abs() < 1e-12);
        // A vertical edge is split so it slants: every point moved.
        let edge = t.warp(&[Point::new(0.0, 0.0), Point::new(0.0, 100.0)], false, &b);
        assert_eq!(edge.len(), 101);
        assert!((edge[50].x - 2.5).abs() < 1e-9);
    }

    #[test]
    fn left_and_right_tops_scale_y_from_their_side() {
        let b = Rect::from_corners(Point::new(0.0, 0.0), Point::new(100.0, 20.0));
        let left = cup(TopEdge::Left).warp_point(Point::new(100.0, 0.0), &b);
        assert!((left.y - 2.5).abs() < 1e-9 && left.x == 100.0);
        let right = cup(TopEdge::Right).warp_point(Point::new(0.0, 0.0), &b);
        assert!((right.y - 2.5).abs() < 1e-9);
        assert_eq!(
            cup(TopEdge::Right).warp_point(Point::new(100.0, 0.0), &b),
            Point::new(100.0, 0.0)
        );
        // A closed square keeps its point count rule: no repeated first point.
        let square = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let w = cup(TopEdge::Left).warp(&square, true, &b);
        assert_ne!(w.first(), w.last());
        assert!(cup(TopEdge::Top).validate().is_ok());
        assert!(
            Taper {
                length: 0.0,
                ..cup(TopEdge::Top)
            }
            .validate()
            .is_err()
        );
        assert_eq!(TopEdge::parse("left"), Some(TopEdge::Left));
    }
}
