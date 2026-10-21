//! Queries on polylines: distances, containment and box intersection.

use crate::{Point, Rect};

/// Distance from `p` to the segment `a`–`b`.
#[must_use]
pub fn distance_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    if len2 <= f64::EPSILON {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// The segments of a polyline, including the closing one when `closed`.
pub fn segments(points: &[Point], closed: bool) -> impl Iterator<Item = (Point, Point)> + '_ {
    let closing = (closed && points.len() > 2)
        .then(|| (points.last().copied(), points.first().copied()))
        .and_then(|(a, b)| Some((a?, b?)));
    points.windows(2).map(|w| (w[0], w[1])).chain(closing)
}

/// Smallest distance from `p` to the polyline's outline.
#[must_use]
pub fn distance_to_polyline(points: &[Point], closed: bool, p: Point) -> f64 {
    if let [only] = points {
        return p.distance(*only);
    }
    segments(points, closed)
        .map(|(a, b)| distance_to_segment(p, a, b))
        .fold(f64::INFINITY, f64::min)
}

/// Whether `p` is inside the closed polygon `points` (even-odd rule).
#[must_use]
pub fn contains(points: &[Point], p: Point) -> bool {
    let mut inside = false;
    for (a, b) in segments(points, true) {
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                inside = !inside;
            }
        }
    }
    inside
}

fn cross(o: Point, a: Point, b: Point) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Whether segments `a1`–`a2` and `b1`–`b2` touch or cross.
#[must_use]
pub fn segments_intersect(a1: Point, a2: Point, b1: Point, b2: Point) -> bool {
    let d1 = cross(b1, b2, a1);
    let d2 = cross(b1, b2, a2);
    let d3 = cross(a1, a2, b1);
    let d4 = cross(a1, a2, b2);
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    let on =
        |o: Point, a: Point, p: Point, d: f64| d == 0.0 && Rect::from_corners(o, a).contains(p);
    on(b1, b2, a1, d1) || on(b1, b2, a2, d2) || on(a1, a2, b1, d3) || on(a1, a2, b2, d4)
}

/// Whether the polyline's outline crosses or lies inside `rect`, or (when
/// closed) the shape contains the rectangle.
#[must_use]
pub fn touches_rect(points: &[Point], closed: bool, rect: &Rect) -> bool {
    if points.iter().any(|p| rect.contains(*p)) {
        return true;
    }
    let corners = rect.corners();
    let edges = [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ];
    let crosses = segments(points, closed)
        .any(|(a, b)| edges.iter().any(|(c, d)| segments_intersect(a, b, *c, *d)));
    crosses || (closed && contains(points, rect.center()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]
    }

    #[test]
    fn segment_distance_clamps_to_endpoints() {
        let (a, b) = (Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        assert_eq!(distance_to_segment(Point::new(5.0, 3.0), a, b), 3.0);
        assert_eq!(distance_to_segment(Point::new(-4.0, 3.0), a, b), 5.0);
        assert_eq!(
            distance_to_segment(Point::new(1.0, 1.0), a, a),
            2.0_f64.sqrt()
        );
    }

    #[test]
    fn closing_segment_counts_only_when_closed() {
        let p = Point::new(0.0, 5.0);
        assert_eq!(distance_to_polyline(&square(), true, p), 0.0);
        assert_eq!(distance_to_polyline(&square(), false, p), 5.0);
        let open = &square()[..3];
        assert_eq!(
            distance_to_polyline(open, false, Point::new(0.0, 10.0)),
            10.0
        );
        assert_eq!(segments(&square(), true).count(), 4);
        assert_eq!(segments(&square(), false).count(), 3);
    }

    #[test]
    fn even_odd_containment() {
        assert!(contains(&square(), Point::new(5.0, 5.0)));
        assert!(!contains(&square(), Point::new(15.0, 5.0)));
        assert!(!contains(&square(), Point::new(5.0, -1.0)));
    }

    #[test]
    fn segment_intersection_including_touching() {
        let p = Point::new;
        assert!(segments_intersect(
            p(0.0, 0.0),
            p(2.0, 2.0),
            p(0.0, 2.0),
            p(2.0, 0.0)
        ));
        assert!(!segments_intersect(
            p(0.0, 0.0),
            p(1.0, 1.0),
            p(2.0, 2.0),
            p(3.0, 0.0)
        ));
        assert!(segments_intersect(
            p(0.0, 0.0),
            p(2.0, 0.0),
            p(2.0, 0.0),
            p(2.0, 5.0)
        ));
        assert!(!segments_intersect(
            p(0.0, 0.0),
            p(1.0, 0.0),
            p(2.0, 0.0),
            p(3.0, 0.0)
        ));
    }

    #[test]
    fn rect_touching() {
        let r = |x0, y0, x1, y1| Rect::from_corners(Point::new(x0, y0), Point::new(x1, y1));
        // A box crossing one edge.
        assert!(touches_rect(&square(), true, &r(8.0, 4.0, 12.0, 6.0)));
        // A box inside a closed shape touches it, but not an open one.
        assert!(touches_rect(&square(), true, &r(4.0, 4.0, 6.0, 6.0)));
        assert!(!touches_rect(&square(), false, &r(4.0, 4.0, 6.0, 6.0)));
        // Far away.
        assert!(!touches_rect(&square(), true, &r(20.0, 20.0, 30.0, 30.0)));
    }
}
