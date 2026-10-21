//! Snapping and drag constraints (`docs/spec/canvas.md`).

use crate::Point;

/// The nearest grid intersection for a grid of `spacing` millimetres.
#[must_use]
pub fn to_grid(p: Point, spacing: f64) -> Point {
    if !(spacing.is_finite() && spacing > 0.0) {
        return p;
    }
    Point::new(
        (p.x / spacing).round() * spacing,
        (p.y / spacing).round() * spacing,
    )
}

/// The target nearest to `p` within `radius`, if any. Ties keep the first.
#[must_use]
pub fn nearest(p: Point, targets: &[Point], radius: f64) -> Option<Point> {
    targets
        .iter()
        .map(|t| (t.distance(p), *t))
        .filter(|(d, _)| *d <= radius)
        .fold(None, |best: Option<(f64, Point)>, cur| match best {
            Some(b) if b.0 <= cur.0 => Some(b),
            _ => Some(cur),
        })
        .map(|(_, t)| t)
}

/// Snapping settings for one drag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snap {
    /// Grid spacing in mm, or `None` for no grid snapping.
    pub grid: Option<f64>,
    /// Object snap radius in mm (the screen radius divided by the zoom).
    pub radius: f64,
}

impl Snap {
    /// The point `p` snaps to, if any: an object target within the radius
    /// wins; otherwise the grid point, if it is within the radius.
    #[must_use]
    pub fn target(&self, p: Point, targets: &[Point]) -> Option<Point> {
        nearest(p, targets, self.radius).or_else(|| {
            let g = to_grid(p, self.grid?);
            (g.distance(p) <= self.radius).then_some(g)
        })
    }

    /// Snaps a point, or returns it unchanged when nothing is in range.
    #[must_use]
    pub fn point(&self, p: Point, targets: &[Point]) -> Point {
        self.target(p, targets).unwrap_or(p)
    }

    /// Snaps a move: of the moving `anchors` (points of the moving bounds,
    /// before the move), the one that lands closest to a snap point after
    /// moving by `delta` decides. Returns the delta that puts that anchor
    /// exactly on its snap point, or `delta` if none is in range.
    #[must_use]
    pub fn delta(&self, delta: Point, anchors: &[Point], targets: &[Point]) -> Point {
        anchors
            .iter()
            .filter_map(|a| {
                let moved = *a + delta;
                let t = self.target(moved, targets)?;
                Some((t.distance(moved), t - *a))
            })
            .fold(None, |best: Option<(f64, Point)>, cur| match best {
                Some(b) if b.0 <= cur.0 => Some(b),
                _ => Some(cur),
            })
            .map_or(delta, |(_, snapped)| snapped)
    }
}

/// Constrains a drag to horizontal, vertical or a 45° diagonal, whichever is
/// closest, keeping the projection of `delta` onto that direction.
#[must_use]
pub fn constrain_45(delta: Point) -> Point {
    if delta.length() == 0.0 {
        return delta;
    }
    let step = std::f64::consts::FRAC_PI_4;
    let angle = (delta.y.atan2(delta.x) / step).round() * step;
    let (sin, cos) = angle.sin_cos();
    let along = delta.x * cos + delta.y * sin;
    Point::new(along * cos, along * sin)
}

/// Rounds an angle in degrees to the nearest multiple of `step`.
#[must_use]
pub fn angle(degrees: f64, step: f64) -> f64 {
    if step > 0.0 {
        (degrees / step).round() * step
    } else {
        degrees
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_rounds_to_nearest_line() {
        assert_eq!(
            to_grid(Point::new(14.0, 26.0), 10.0),
            Point::new(10.0, 30.0)
        );
        assert_eq!(to_grid(Point::new(14.0, 26.0), 0.0), Point::new(14.0, 26.0));
    }

    #[test]
    fn object_targets_beat_the_grid() {
        let snap = Snap {
            grid: Some(10.0),
            radius: 2.0,
        };
        let targets = [Point::new(11.5, 20.0)];
        assert_eq!(
            snap.point(Point::new(10.5, 20.0), &targets),
            Point::new(11.5, 20.0)
        );
        assert_eq!(
            snap.point(Point::new(29.0, 41.0), &targets),
            Point::new(30.0, 40.0)
        );
        // Out of range of both.
        assert_eq!(
            snap.point(Point::new(25.0, 45.0), &targets),
            Point::new(25.0, 45.0)
        );
    }

    #[test]
    fn move_snaps_the_closest_anchor() {
        let snap = Snap {
            grid: Some(10.0),
            radius: 1.0,
        };
        // A box corner at (0.3, 0.3) moved by (9.5, 0): lands at (9.8, 0.3),
        // which snaps to (10, 0).
        let anchors = [Point::new(0.3, 0.3), Point::new(5.3, 5.3)];
        let d = snap.delta(Point::new(9.5, 0.0), &anchors, &[]);
        assert!(d.approx_eq(Point::new(9.7, -0.3), 1e-12), "{d:?}");
        // The anchor lands exactly on the grid line, with no rounding drift.
        let exact = snap.delta(Point::new(19.6, 0.2), &[Point::ZERO], &[]);
        assert_eq!(exact, Point::new(20.0, 0.0));
        // An anchor already exactly on the grid wins over nearby misses.
        let wide = Snap {
            grid: Some(10.0),
            radius: 5.5,
        };
        let anchors = [Point::new(100.0, 100.0), Point::new(125.0, 100.0)];
        assert_eq!(
            wide.delta(Point::new(100.0, 0.0), &anchors, &[]),
            Point::new(100.0, 0.0)
        );
        // Nothing in range: unchanged.
        let free = Snap {
            grid: None,
            radius: 1.0,
        };
        assert_eq!(
            free.delta(Point::new(3.0, 4.0), &anchors, &[]),
            Point::new(3.0, 4.0)
        );
    }

    #[test]
    fn constrain_picks_nearest_direction() {
        assert!(constrain_45(Point::new(10.0, 1.0)).approx_eq(Point::new(10.0, 0.0), 1e-9));
        assert!(constrain_45(Point::new(1.0, -10.0)).approx_eq(Point::new(0.0, -10.0), 1e-9));
        let d = constrain_45(Point::new(10.0, 9.0));
        assert!((d.x - d.y).abs() < 1e-9 && d.x > 9.0);
        assert_eq!(constrain_45(Point::ZERO), Point::ZERO);
    }

    #[test]
    fn angles_round_to_steps() {
        assert_eq!(angle(22.0, 15.0), 15.0);
        assert_eq!(angle(-23.0, 15.0), -30.0);
        assert_eq!(angle(22.0, 0.0), 22.0);
    }
}
