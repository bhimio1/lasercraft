use crate::{Affine, Point};

/// An axis-aligned rectangle given by its minimum and maximum corners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Corner with the smallest x and y.
    pub min: Point,
    /// Corner with the largest x and y.
    pub max: Point,
}

impl Rect {
    /// The rectangle spanned by two corners, in any order.
    #[must_use]
    pub fn from_corners(a: Point, b: Point) -> Self {
        Self {
            min: Point::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    /// The smallest rectangle holding every point, or `None` for no points.
    #[must_use]
    pub fn bounding(points: impl IntoIterator<Item = Point>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        Some(points.fold(Self::from_corners(first, first), |r, p| r.including(p)))
    }

    /// This rectangle grown to include `p`.
    #[must_use]
    pub fn including(self, p: Point) -> Self {
        Self {
            min: Point::new(self.min.x.min(p.x), self.min.y.min(p.y)),
            max: Point::new(self.max.x.max(p.x), self.max.y.max(p.y)),
        }
    }

    /// Smallest rectangle holding both.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        self.including(other.min).including(other.max)
    }

    /// Width.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// Height.
    #[must_use]
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    /// Center point.
    #[must_use]
    pub fn center(&self) -> Point {
        (self.min + self.max) * 0.5
    }

    /// Whether `p` lies inside or on the edge.
    #[must_use]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    /// Whether `other` lies entirely inside or on the edge.
    #[must_use]
    pub fn contains_rect(&self, other: &Self) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    /// The four corners, starting at `min` and going round.
    #[must_use]
    pub fn corners(&self) -> [Point; 4] {
        [
            self.min,
            Point::new(self.max.x, self.min.y),
            self.max,
            Point::new(self.min.x, self.max.y),
        ]
    }

    /// The bounding box of this rectangle after a transform.
    #[must_use]
    pub fn transformed(&self, t: &Affine) -> Self {
        let [a, b, c, d] = self.corners().map(|p| t.apply(p));
        Self::from_corners(a, b).including(c).including(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounding_box_of_points() {
        let r = Rect::bounding([
            Point::new(3.0, -1.0),
            Point::new(-2.0, 4.0),
            Point::new(0.0, 0.0),
        ])
        .unwrap();
        assert_eq!(r.min, Point::new(-2.0, -1.0));
        assert_eq!(r.max, Point::new(3.0, 4.0));
        assert_eq!(r.width(), 5.0);
        assert_eq!(r.height(), 5.0);
        assert!(Rect::bounding(std::iter::empty()).is_none());
    }

    #[test]
    fn containment() {
        let bed = Rect::from_corners(Point::ZERO, Point::new(400.0, 300.0));
        assert!(bed.contains(Point::new(400.0, 300.0)));
        assert!(!bed.contains(Point::new(400.1, 10.0)));
        let job = Rect::from_corners(Point::new(10.0, 10.0), Point::new(50.0, 50.0));
        assert!(bed.contains_rect(&job));
        assert!(!job.contains_rect(&bed));
    }

    #[test]
    fn transformed_bounds_of_rotated_square() {
        let square = Rect::from_corners(Point::new(-1.0, -1.0), Point::new(1.0, 1.0));
        let r = square.transformed(&Affine::rotate_degrees(45.0));
        let half_diagonal = 2.0_f64.sqrt();
        assert!((r.width() - 2.0 * half_diagonal).abs() < 1e-12);
        assert!(r.center().approx_eq(Point::ZERO, 1e-12));
    }
}
