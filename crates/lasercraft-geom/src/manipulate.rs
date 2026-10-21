//! Resize and rotate handles around a selection's bounds
//! (`docs/spec/canvas.md`).

use crate::{Point, Rect};

/// One of the eight scale handles on a bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    /// Top-left corner.
    TopLeft,
    /// Middle of the top edge.
    Top,
    /// Top-right corner.
    TopRight,
    /// Middle of the right edge.
    Right,
    /// Bottom-right corner.
    BottomRight,
    /// Middle of the bottom edge.
    Bottom,
    /// Bottom-left corner.
    BottomLeft,
    /// Middle of the left edge.
    Left,
}

impl Handle {
    /// Every handle, clockwise from the top-left corner.
    pub const ALL: [Self; 8] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Right,
        Self::BottomRight,
        Self::Bottom,
        Self::BottomLeft,
        Self::Left,
    ];

    /// Position relative to the box: 0 is min, 0.5 the middle, 1 is max.
    const fn fraction(self) -> (f64, f64) {
        match self {
            Self::TopLeft => (0.0, 0.0),
            Self::Top => (0.5, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::Right => (1.0, 0.5),
            Self::BottomRight => (1.0, 1.0),
            Self::Bottom => (0.5, 1.0),
            Self::BottomLeft => (0.0, 1.0),
            Self::Left => (0.0, 0.5),
        }
    }

    /// Where the handle sits on `bounds`.
    #[must_use]
    pub fn position(self, bounds: &Rect) -> Point {
        let (fx, fy) = self.fraction();
        Point::new(
            bounds.min.x + fx * bounds.width(),
            bounds.min.y + fy * bounds.height(),
        )
    }

    /// The handle opposite this one.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::TopLeft => Self::BottomRight,
            Self::Top => Self::Bottom,
            Self::TopRight => Self::BottomLeft,
            Self::Right => Self::Left,
            Self::BottomRight => Self::TopLeft,
            Self::Bottom => Self::Top,
            Self::BottomLeft => Self::TopRight,
            Self::Left => Self::Right,
        }
    }

    /// Whether this is a corner handle (scales both axes).
    #[must_use]
    pub const fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomRight | Self::BottomLeft
        )
    }
}

/// A scale about an anchor point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    /// Horizontal factor (negative mirrors).
    pub sx: f64,
    /// Vertical factor (negative mirrors).
    pub sy: f64,
    /// Fixed point of the scale.
    pub anchor: Point,
}

/// Factors below this are treated as a collapse and refused.
const MIN_FACTOR: f64 = 1e-6;

/// The scale produced by dragging `handle` of `bounds` to `pointer`.
///
/// Corner handles keep the aspect ratio when `keep_aspect` is set (the larger
/// of the two factors wins, each axis keeping its own sign); edge handles
/// scale one axis. `from_center` anchors at the center instead of the
/// opposite handle. An axis with zero extent keeps factor 1. Returns `None`
/// when the result would collapse the selection.
#[must_use]
pub fn scale_for_handle(
    bounds: &Rect,
    handle: Handle,
    pointer: Point,
    keep_aspect: bool,
    from_center: bool,
) -> Option<Scale> {
    let anchor = if from_center {
        bounds.center()
    } else {
        handle.opposite().position(bounds)
    };
    let grip = handle.position(bounds);
    let factor = |grip: f64, anchor: f64, pointer: f64| {
        let span = grip - anchor;
        if span.abs() <= f64::EPSILON {
            1.0
        } else {
            (pointer - anchor) / span
        }
    };
    let (fx, fy) = handle.fraction();
    let mut sx = if fx == 0.5 {
        1.0
    } else {
        factor(grip.x, anchor.x, pointer.x)
    };
    let mut sy = if fy == 0.5 {
        1.0
    } else {
        factor(grip.y, anchor.y, pointer.y)
    };
    if keep_aspect && handle.is_corner() {
        let magnitude = sx.abs().max(sy.abs());
        sx = magnitude.copysign(sx);
        sy = magnitude.copysign(sy);
    }
    let usable = |s: f64| s.is_finite() && s.abs() >= MIN_FACTOR;
    (usable(sx) && usable(sy)).then_some(Scale { sx, sy, anchor })
}

/// The angle in degrees (clockwise on screen) from the direction
/// `center → start` to the direction `center → current`, in (-180, 180].
#[must_use]
pub fn rotation_degrees(center: Point, start: Point, current: Point) -> f64 {
    let a = (start - center).y.atan2((start - center).x);
    let b = (current - center).y.atan2((current - center).x);
    let mut d = (b - a).to_degrees();
    while d <= -180.0 {
        d += 360.0;
    }
    while d > 180.0 {
        d -= 360.0;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rect {
        Rect::from_corners(Point::new(0.0, 0.0), Point::new(20.0, 10.0))
    }

    #[test]
    fn handles_sit_on_the_box_and_have_opposites() {
        let b = bounds();
        assert_eq!(Handle::Right.position(&b), Point::new(20.0, 5.0));
        assert_eq!(Handle::BottomLeft.position(&b), Point::new(0.0, 10.0));
        for h in Handle::ALL {
            assert_eq!(h.opposite().opposite(), h);
            let mid = (h.position(&b) + h.opposite().position(&b)) * 0.5;
            assert_eq!(mid, b.center());
        }
    }

    #[test]
    fn corner_drag_keeps_aspect_unless_released() {
        let b = bounds();
        let s =
            scale_for_handle(&b, Handle::BottomRight, Point::new(40.0, 12.0), true, false).unwrap();
        assert_eq!((s.sx, s.sy), (2.0, 2.0));
        assert_eq!(s.anchor, Point::new(0.0, 0.0));
        let free = scale_for_handle(
            &b,
            Handle::BottomRight,
            Point::new(40.0, 12.0),
            false,
            false,
        )
        .unwrap();
        assert_eq!((free.sx, free.sy), (2.0, 1.2));
    }

    #[test]
    fn edge_drag_scales_one_axis_and_center_anchor_doubles() {
        let b = bounds();
        let s = scale_for_handle(&b, Handle::Right, Point::new(30.0, 99.0), true, false).unwrap();
        assert_eq!((s.sx, s.sy), (1.5, 1.0));
        let c = scale_for_handle(&b, Handle::Right, Point::new(30.0, 0.0), false, true).unwrap();
        assert_eq!((c.sx, c.sy), (2.0, 1.0));
        assert_eq!(c.anchor, b.center());
    }

    #[test]
    fn dragging_past_the_anchor_mirrors_and_collapse_is_refused() {
        let b = bounds();
        let s = scale_for_handle(&b, Handle::Right, Point::new(-20.0, 5.0), false, false).unwrap();
        assert_eq!(s.sx, -1.0);
        assert!(scale_for_handle(&b, Handle::Right, Point::new(0.0, 5.0), false, false).is_none());
    }

    #[test]
    fn flat_bounds_keep_factor_one_on_the_flat_axis() {
        let line = Rect::from_corners(Point::new(0.0, 5.0), Point::new(10.0, 5.0));
        let s = scale_for_handle(
            &line,
            Handle::BottomRight,
            Point::new(20.0, 9.0),
            false,
            false,
        )
        .unwrap();
        assert_eq!((s.sx, s.sy), (2.0, 1.0));
    }

    #[test]
    fn rotation_is_signed_and_wraps() {
        let c = Point::ZERO;
        let r = rotation_degrees(c, Point::new(1.0, 0.0), Point::new(0.0, 1.0));
        assert!((r - 90.0).abs() < 1e-9);
        let back = rotation_degrees(c, Point::new(-1.0, 0.1), Point::new(-1.0, -0.1));
        assert!(back.abs() < 12.0, "{back}");
    }
}
