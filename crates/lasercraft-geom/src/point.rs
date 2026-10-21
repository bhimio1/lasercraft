use std::ops::{Add, Mul, Neg, Sub};

/// A point or vector in document space, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// Horizontal position, growing to the right.
    pub x: f64,
    /// Vertical position, growing downwards.
    pub y: f64,
}

impl Point {
    /// The origin.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Builds a point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Length of the vector from the origin to this point.
    #[must_use]
    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    /// Distance to another point.
    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    /// Whether both coordinates are within `tolerance` of `other`'s.
    #[must_use]
    pub fn approx_eq(self, other: Self, tolerance: f64) -> bool {
        (self.x - other.x).abs() <= tolerance && (self.y - other.y).abs() <= tolerance
    }

    /// Whether both coordinates are finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Point {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Neg for Point {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_and_distance() {
        let a = Point::new(1.0, 2.0);
        let b = Point::new(4.0, 6.0);
        assert_eq!(b - a, Point::new(3.0, 4.0));
        assert_eq!(a + b, Point::new(5.0, 8.0));
        assert_eq!(a * 2.0, Point::new(2.0, 4.0));
        assert_eq!(-a, Point::new(-1.0, -2.0));
        assert!((a.distance(b) - 5.0).abs() < crate::EPSILON);
    }

    #[test]
    fn finiteness() {
        assert!(Point::new(1.0, 2.0).is_finite());
        assert!(!Point::new(f64::NAN, 2.0).is_finite());
        assert!(!Point::new(1.0, f64::INFINITY).is_finite());
    }
}
