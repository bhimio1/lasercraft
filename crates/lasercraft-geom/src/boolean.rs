//! Polygon booleans and offsets on flattened outlines, through `i_overlay`
//! (`docs/spec/geometry.md`). Inputs and outputs are closed contours; the
//! even-odd rule decides what is inside.

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::mesh::float::outline::offset::OutlineOffset;
use i_overlay::mesh::float::style::{LineJoin, OutlineStyle};

use crate::Point;

/// Closed contours (first point not repeated).
pub type Contours = Vec<Vec<Point>>;

/// A boolean operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOp {
    /// Area in either.
    Union,
    /// Area in the first but not the second.
    Subtract,
    /// Area in both.
    Intersect,
    /// Area in exactly one.
    Xor,
}

/// How offset corners are joined.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Join {
    /// Sharp corners.
    Miter,
    /// Rounded corners, flattened within this tolerance (mm).
    Round(f64),
    /// Corners cut straight across.
    Bevel,
}

fn to_raw(contours: &[Vec<Point>]) -> Vec<Vec<[f64; 2]>> {
    contours
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| c.iter().map(|p| [p.x, p.y]).collect())
        .collect()
}

fn from_shapes(shapes: Vec<Vec<Vec<[f64; 2]>>>) -> Contours {
    shapes
        .into_iter()
        .flatten()
        .map(|c| c.into_iter().map(|[x, y]| Point::new(x, y)).collect())
        .collect()
}

/// `a op b`, as closed contours (outer boundaries and holes alike).
#[must_use]
pub fn boolean(a: &[Vec<Point>], b: &[Vec<Point>], op: BoolOp) -> Contours {
    let rule = match op {
        BoolOp::Union => OverlayRule::Union,
        BoolOp::Subtract => OverlayRule::Difference,
        BoolOp::Intersect => OverlayRule::Intersect,
        BoolOp::Xor => OverlayRule::Xor,
    };
    let (a, b) = (to_raw(a), to_raw(b));
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    from_shapes(a.overlay(&b, rule, FillRule::EvenOdd))
}

/// The union of several shapes, each given as its own contours (even-odd
/// within a shape, so its holes stay holes): overlapping shapes become one
/// outline.
#[must_use]
pub fn union_all(shapes: &[Contours]) -> Contours {
    shapes
        .iter()
        .fold(Vec::new(), |acc, shape| boolean(&acc, shape, BoolOp::Union))
}

/// Grows (`distance` > 0) or shrinks (< 0) closed contours by `distance` mm.
#[must_use]
pub fn offset(contours: &[Vec<Point>], distance: f64, join: Join) -> Contours {
    let raw = to_raw(contours);
    if raw.is_empty() || !distance.is_finite() {
        return Vec::new();
    }
    if distance == 0.0 {
        return from_shapes(vec![raw]);
    }
    let join = match join {
        Join::Miter => LineJoin::Miter(0.01),
        Join::Round(tolerance) => LineJoin::Round(tolerance.max(1e-6)),
        Join::Bevel => LineJoin::Bevel,
    };
    let style = OutlineStyle::new(distance).line_join(join);
    from_shapes(raw.outline(&style))
}

/// Signed area of a closed contour (positive when counter-clockwise in y-up
/// terms, i.e. clockwise on screen).
#[must_use]
pub fn area(contour: &[Point]) -> f64 {
    crate::polyline::segments(contour, true)
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum::<f64>()
        / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, s: f64) -> Vec<Point> {
        vec![
            Point::new(x, y),
            Point::new(x + s, y),
            Point::new(x + s, y + s),
            Point::new(x, y + s),
        ]
    }

    fn total_area(c: &Contours) -> f64 {
        // Holes come back with the opposite winding, so signed areas add up.
        c.iter().map(|c| area(c)).sum::<f64>().abs()
    }

    #[test]
    fn booleans_of_overlapping_squares() {
        let a = vec![square(0.0, 0.0, 10.0)];
        let b = vec![square(5.0, 5.0, 10.0)];
        assert!((total_area(&boolean(&a, &b, BoolOp::Union)) - 175.0).abs() < 1e-6);
        assert!((total_area(&boolean(&a, &b, BoolOp::Subtract)) - 75.0).abs() < 1e-6);
        assert!((total_area(&boolean(&a, &b, BoolOp::Intersect)) - 25.0).abs() < 1e-6);
        assert!((total_area(&boolean(&a, &b, BoolOp::Xor)) - 150.0).abs() < 1e-6);
        assert!(boolean(&[], &[], BoolOp::Union).is_empty());
    }

    #[test]
    fn union_all_merges_overlaps() {
        let merged = union_all(&[vec![square(0.0, 0.0, 10.0)], vec![square(5.0, 0.0, 10.0)]]);
        assert_eq!(merged.len(), 1);
        assert!((total_area(&merged) - 150.0).abs() < 1e-6);
        // A shape's own hole survives the union.
        let ring = vec![square(0.0, 0.0, 10.0), square(3.0, 3.0, 4.0)];
        let merged = union_all(&[ring, vec![square(20.0, 0.0, 5.0)]]);
        assert!((total_area(&merged) - (100.0 - 16.0 + 25.0)).abs() < 1e-6);
    }

    #[test]
    fn offset_grows_and_shrinks() {
        let s = vec![square(0.0, 0.0, 10.0)];
        let inset = offset(&s, -1.0, Join::Miter);
        assert!((total_area(&inset) - 64.0).abs() < 1e-6, "{inset:?}");
        let grown = offset(&s, 1.0, Join::Miter);
        // Corners may be clipped by a hair; area is just under 12².
        let a = total_area(&grown);
        assert!(a > 143.9 && a <= 144.0 + 1e-6, "{a}");
        let round = offset(&s, 1.0, Join::Round(0.01));
        let r = total_area(&round);
        let expected = 100.0 + 40.0 + std::f64::consts::PI;
        assert!((r - expected).abs() < 0.05, "{r}");
        // Shrinking past the middle leaves nothing.
        assert!(offset(&s, -6.0, Join::Miter).is_empty());
        assert_eq!(offset(&s, 0.0, Join::Miter).len(), 1);
    }
}
