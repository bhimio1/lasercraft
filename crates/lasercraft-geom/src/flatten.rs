//! Turning curves into polylines within a distance tolerance.

use std::f64::consts::PI;

use crate::Point;

/// Fewest and most segments used for a full ellipse.
const MIN_SEGMENTS: usize = 8;
const MAX_SEGMENTS: usize = 4096;

/// Points on the ellipse centered on the origin with radii `rx`, `ry`,
/// starting at angle 0 and going clockwise on screen (y down). The polygon
/// through them (closed, first point not repeated) stays within `tolerance`
/// of the true ellipse.
#[must_use]
pub fn ellipse_points(rx: f64, ry: f64, tolerance: f64) -> Vec<Point> {
    let n = ellipse_segments(rx.abs().max(ry.abs()), tolerance);
    (0..n)
        .map(|i| {
            let (sin, cos) = (2.0 * PI * i as f64 / n as f64).sin_cos();
            Point::new(rx * cos, ry * sin)
        })
        .collect()
}

/// Segments needed so that chords of a circle of radius `r` deviate from it by
/// at most `tolerance` (sagitta `r(1 - cos(θ/2))`).
fn ellipse_segments(r: f64, tolerance: f64) -> usize {
    if !(r.is_finite() && tolerance.is_finite()) || r <= 0.0 || tolerance <= 0.0 {
        return MIN_SEGMENTS;
    }
    if tolerance >= r {
        return MIN_SEGMENTS;
    }
    let half_angle = (1.0 - tolerance / r).acos();
    let n = (PI / half_angle).ceil();
    // Clamped above, so the cast cannot overflow.
    (n.min(MAX_SEGMENTS as f64) as usize).max(MIN_SEGMENTS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyline::distance_to_segment;

    #[test]
    fn circle_chords_stay_within_tolerance() {
        for (r, tol) in [(10.0, 0.01), (100.0, 0.05), (1.0, 0.001), (500.0, 0.1)] {
            let pts = ellipse_points(r, r, tol);
            let n = pts.len();
            for i in 0..n {
                let (a, b) = (pts[i], pts[(i + 1) % n]);
                let mid_angle = 2.0 * PI * (i as f64 + 0.5) / n as f64;
                let on_circle = Point::new(r * mid_angle.cos(), r * mid_angle.sin());
                assert!(
                    distance_to_segment(on_circle, a, b) <= tol * 1.0001,
                    "r {r} tol {tol}"
                );
            }
        }
    }

    #[test]
    fn segment_count_is_bounded() {
        assert_eq!(ellipse_points(1.0, 1.0, 5.0).len(), MIN_SEGMENTS);
        assert_eq!(ellipse_points(1e9, 1e9, 1e-9).len(), MAX_SEGMENTS);
        assert_eq!(ellipse_points(1.0, 1.0, 0.0).len(), MIN_SEGMENTS);
    }

    #[test]
    fn starts_at_angle_zero_and_turns_towards_positive_y() {
        let pts = ellipse_points(20.0, 10.0, 0.1);
        assert!(pts[0].approx_eq(Point::new(20.0, 0.0), 1e-12));
        assert!(pts[1].y > 0.0);
    }
}
