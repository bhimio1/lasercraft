use std::ops::Mul;

use crate::Point;

/// A 2D affine transform in SVG's `matrix(a b c d e f)` layout:
///
/// ```text
/// x' = a·x + c·y + e
/// y' = b·x + d·y + f
/// ```
///
/// `t1 * t2` applies `t2` first, then `t1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    /// Coefficients `[a, b, c, d, e, f]`.
    pub m: [f64; 6],
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    /// The transform that changes nothing.
    pub const IDENTITY: Self = Self {
        m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// Moves by `(dx, dy)`.
    #[must_use]
    pub const fn translate(dx: f64, dy: f64) -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 1.0, dx, dy],
        }
    }

    /// Scales about the origin.
    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            m: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    /// Rotates about the origin by `degrees`. With y growing downwards a
    /// positive angle turns clockwise on screen, as in SVG.
    #[must_use]
    pub fn rotate_degrees(degrees: f64) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        Self {
            m: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    /// Rotates by `degrees` about `center`.
    #[must_use]
    pub fn rotate_about(degrees: f64, center: Point) -> Self {
        Self::translate(center.x, center.y)
            * Self::rotate_degrees(degrees)
            * Self::translate(-center.x, -center.y)
    }

    /// Applies the transform to a point.
    #[must_use]
    pub fn apply(&self, p: Point) -> Point {
        let [a, b, c, d, e, f] = self.m;
        Point::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    /// The determinant of the linear part. Zero means the transform collapses
    /// shapes to a line or a point.
    #[must_use]
    pub fn determinant(&self) -> f64 {
        let [a, b, c, d, ..] = self.m;
        a * d - b * c
    }

    /// The inverse transform, or `None` if the determinant is (nearly) zero.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < crate::EPSILON || !det.is_finite() {
            return None;
        }
        let [a, b, c, d, e, f] = self.m;
        let inv = 1.0 / det;
        Some(Self {
            m: [
                d * inv,
                -b * inv,
                -c * inv,
                a * inv,
                (c * f - d * e) * inv,
                (b * e - a * f) * inv,
            ],
        })
    }

    /// Whether every coefficient is within `tolerance` of `other`'s.
    #[must_use]
    pub fn approx_eq(&self, other: &Self, tolerance: f64) -> bool {
        self.m
            .iter()
            .zip(other.m.iter())
            .all(|(x, y)| (x - y).abs() <= tolerance)
    }
}

impl Mul for Affine {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let [a1, b1, c1, d1, e1, f1] = self.m;
        let [a2, b2, c2, d2, e2, f2] = rhs.m;
        Self {
            m: [
                a1 * a2 + c1 * b2,
                b1 * a2 + d1 * b2,
                a1 * c2 + c1 * d2,
                b1 * c2 + d1 * d2,
                a1 * e2 + c1 * f2 + e1,
                b1 * e2 + d1 * f2 + f1,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-12;

    #[test]
    fn composition_applies_right_operand_first() {
        let t = Affine::translate(10.0, 0.0) * Affine::scale(2.0, 2.0);
        assert!(
            t.apply(Point::new(1.0, 1.0))
                .approx_eq(Point::new(12.0, 2.0), TOL)
        );
    }

    #[test]
    fn rotation_turns_x_axis_towards_y() {
        let p = Affine::rotate_degrees(90.0).apply(Point::new(1.0, 0.0));
        assert!(p.approx_eq(Point::new(0.0, 1.0), TOL));
    }

    #[test]
    fn rotate_about_keeps_center_fixed() {
        let c = Point::new(5.0, 5.0);
        let t = Affine::rotate_about(37.0, c);
        assert!(t.apply(c).approx_eq(c, TOL));
    }

    #[test]
    fn inverse_undoes_transform() {
        let t =
            Affine::translate(3.0, -4.0) * Affine::rotate_degrees(30.0) * Affine::scale(2.0, 0.5);
        let inv = t.inverse().unwrap();
        assert!((inv * t).approx_eq(&Affine::IDENTITY, TOL));
        let p = Point::new(7.0, 11.0);
        assert!(inv.apply(t.apply(p)).approx_eq(p, TOL));
    }

    #[test]
    fn degenerate_transform_has_no_inverse() {
        assert!(Affine::scale(0.0, 1.0).inverse().is_none());
    }
}
