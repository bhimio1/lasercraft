//! Paths made of straight and cubic Bézier segments (`docs/spec/geometry.md`).
//!
//! A [`Path`] is one or more [`SubPath`]s; each starts at a point and
//! continues through segments, open or closed. Imported art, text, node
//! editing and boolean results are all paths.

use serde::{Deserialize, Serialize};

use crate::{Affine, Point, Rect};

/// Deepest recursion when flattening one cubic.
const MAX_DEPTH: u32 = 16;

/// One segment, from the previous end point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Vec<f64>", into = "Vec<f64>")]
pub enum Segment {
    /// Straight line to the point. Serialized as `[x, y]`.
    Line(Point),
    /// Cubic Bézier with two control points, then the end point. Serialized
    /// as `[c1x, c1y, c2x, c2y, x, y]`.
    Cubic(Point, Point, Point),
}

impl From<Segment> for Vec<f64> {
    fn from(s: Segment) -> Self {
        match s {
            Segment::Line(p) => vec![p.x, p.y],
            Segment::Cubic(a, b, p) => vec![a.x, a.y, b.x, b.y, p.x, p.y],
        }
    }
}

impl TryFrom<Vec<f64>> for Segment {
    type Error = String;

    fn try_from(v: Vec<f64>) -> Result<Self, String> {
        match v.as_slice() {
            [x, y] => Ok(Self::Line(Point::new(*x, *y))),
            [ax, ay, bx, by, x, y] => Ok(Self::Cubic(
                Point::new(*ax, *ay),
                Point::new(*bx, *by),
                Point::new(*x, *y),
            )),
            _ => Err(format!(
                "a segment has 2 (line) or 6 (cubic) numbers, not {}",
                v.len()
            )),
        }
    }
}

impl Segment {
    /// The segment's end point.
    #[must_use]
    pub fn end(&self) -> Point {
        match self {
            Self::Line(p) | Self::Cubic(_, _, p) => *p,
        }
    }

    /// Every point that defines the segment.
    fn points(&self) -> impl Iterator<Item = Point> {
        let (a, b, c) = match *self {
            Self::Line(p) => (p, p, p),
            Self::Cubic(a, b, c) => (a, b, c),
        };
        [a, b, c].into_iter()
    }

    fn transformed(&self, t: &Affine) -> Self {
        match self {
            Self::Line(p) => Self::Line(t.apply(*p)),
            Self::Cubic(a, b, c) => Self::Cubic(t.apply(*a), t.apply(*b), t.apply(*c)),
        }
    }
}

/// A connected run of segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubPath {
    /// Where the subpath starts.
    pub start: Point,
    /// Segments in order.
    pub segments: Vec<Segment>,
    /// Whether the end joins back to the start.
    pub closed: bool,
}

impl SubPath {
    /// A polyline as a subpath.
    #[must_use]
    pub fn from_points(points: &[Point], closed: bool) -> Option<Self> {
        let (first, rest) = points.split_first()?;
        Some(Self {
            start: *first,
            segments: rest.iter().map(|p| Segment::Line(*p)).collect(),
            closed,
        })
    }

    /// The points within `tolerance` mm of the true curve: start, then each
    /// segment's points. A closed subpath does not repeat its start.
    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> Vec<Point> {
        let mut out = vec![self.start];
        let mut from = self.start;
        for seg in &self.segments {
            match *seg {
                Segment::Line(p) => out.push(p),
                Segment::Cubic(a, b, p) => flatten_cubic(from, a, b, p, tolerance, 0, &mut out),
            }
            from = seg.end();
        }
        if self.closed && out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
        out
    }

    /// Exact bounding box.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        let mut r = Rect::from_corners(self.start, self.start);
        let mut from = self.start;
        for seg in &self.segments {
            match *seg {
                Segment::Line(p) => r = r.including(p),
                Segment::Cubic(a, b, p) => {
                    r = r.including(p);
                    for t in cubic_extrema(from, a, b, p) {
                        r = r.including(cubic_at(from, a, b, p, t));
                    }
                }
            }
            from = seg.end();
        }
        r
    }

    fn transformed(&self, t: &Affine) -> Self {
        Self {
            start: t.apply(self.start),
            segments: self.segments.iter().map(|s| s.transformed(t)).collect(),
            closed: self.closed,
        }
    }
}

/// One or more subpaths.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Path {
    /// The subpaths, in order.
    pub subpaths: Vec<SubPath>,
}

impl Path {
    /// A path of one subpath.
    #[must_use]
    pub fn single(subpath: SubPath) -> Self {
        Self {
            subpaths: vec![subpath],
        }
    }

    /// Whether every coordinate is finite and every subpath has a segment.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.subpaths.is_empty()
            && self.subpaths.iter().all(|s| {
                !s.segments.is_empty()
                    && s.start.is_finite()
                    && s.segments
                        .iter()
                        .all(|seg| seg.points().all(Point::is_finite))
            })
    }

    /// Every defining point (starts, ends and control points).
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.subpaths.iter().flat_map(|s| {
            std::iter::once(s.start).chain(s.segments.iter().flat_map(Segment::points))
        })
    }

    /// The path under a transform. Affine maps keep Béziers exact.
    #[must_use]
    pub fn transformed(&self, t: &Affine) -> Self {
        Self {
            subpaths: self.subpaths.iter().map(|s| s.transformed(t)).collect(),
        }
    }

    /// Each subpath flattened within `tolerance` mm, with its closed flag.
    #[must_use]
    pub fn flatten(&self, tolerance: f64) -> Vec<(Vec<Point>, bool)> {
        self.subpaths
            .iter()
            .map(|s| (s.flatten(tolerance), s.closed))
            .collect()
    }

    /// Exact bounding box, or `None` for an empty path.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        self.subpaths
            .iter()
            .map(SubPath::bounds)
            .reduce(Rect::union)
    }

    /// Whether every subpath is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.subpaths.iter().all(|s| s.closed)
    }
}

/// Point on a cubic at parameter `t`.
#[must_use]
pub fn cubic_at(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

/// Parameters in (0, 1) where the cubic's x or y derivative is zero.
fn cubic_extrema(p0: Point, p1: Point, p2: Point, p3: Point) -> Vec<f64> {
    let mut ts = Vec::new();
    for (a0, a1, a2, a3) in [(p0.x, p1.x, p2.x, p3.x), (p0.y, p1.y, p2.y, p3.y)] {
        // Derivative / 3 = a t² + b t + c.
        let a = -a0 + 3.0 * a1 - 3.0 * a2 + a3;
        let b = 2.0 * (a0 - 2.0 * a1 + a2);
        let c = a1 - a0;
        if a.abs() < 1e-12 {
            if b.abs() > 1e-12 {
                ts.push(-c / b);
            }
        } else {
            let disc = b * b - 4.0 * a * c;
            if disc >= 0.0 {
                let root = disc.sqrt();
                ts.push((-b + root) / (2.0 * a));
                ts.push((-b - root) / (2.0 * a));
            }
        }
    }
    ts.retain(|t| *t > 0.0 && *t < 1.0);
    ts
}

/// How far the control points stray from the chord: an upper bound on the
/// curve's distance from it.
fn flatness(p0: Point, p1: Point, p2: Point, p3: Point) -> f64 {
    crate::polyline::distance_to_segment(p1, p0, p3)
        .max(crate::polyline::distance_to_segment(p2, p0, p3))
}

/// Appends points of the cubic after `p0` (not `p0` itself), subdividing
/// until each piece is within `tolerance` of its chord.
fn flatten_cubic(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tolerance: f64,
    depth: u32,
    out: &mut Vec<Point>,
) {
    let usable = tolerance.is_finite() && tolerance > 0.0;
    if depth >= MAX_DEPTH || !usable || flatness(p0, p1, p2, p3) <= tolerance {
        out.push(p3);
        return;
    }
    // de Casteljau split at t = 0.5.
    let mid = |a: Point, b: Point| (a + b) * 0.5;
    let (p01, p12, p23) = (mid(p0, p1), mid(p1, p2), mid(p2, p3));
    let (p012, p123) = (mid(p01, p12), mid(p12, p23));
    let m = mid(p012, p123);
    flatten_cubic(p0, p01, p012, m, tolerance, depth + 1, out);
    flatten_cubic(m, p123, p23, p3, tolerance, depth + 1, out);
}

/// A circular arc of `radius` about `center` from `start_deg` sweeping
/// `sweep_deg` (positive is clockwise on screen), as cubic segments of at
/// most 90° each. Returns the start point and the segments.
#[must_use]
pub fn arc(center: Point, radius: f64, start_deg: f64, sweep_deg: f64) -> (Point, Vec<Segment>) {
    let at = |deg: f64| {
        let (s, c) = deg.to_radians().sin_cos();
        center + Point::new(c, s) * radius
    };
    // Whole pieces only; the count is small and positive.
    let pieces = (sweep_deg.abs() / 90.0).ceil().clamp(1.0, 64.0) as u32;
    let step = sweep_deg / f64::from(pieces);
    let k = 4.0 / 3.0 * (step.to_radians() / 4.0).tan() * radius;
    let mut segments = Vec::with_capacity(pieces as usize);
    for i in 0..pieces {
        let a = start_deg + step * f64::from(i);
        let b = a + step;
        let (sa, ca) = a.to_radians().sin_cos();
        let (sb, cb) = b.to_radians().sin_cos();
        let c1 = at(a) + Point::new(-sa, ca) * k;
        let c2 = at(b) - Point::new(-sb, cb) * k;
        segments.push(Segment::Cubic(c1, c2, at(b)));
    }
    (at(start_deg), segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(r: f64) -> Path {
        let (start, segments) = arc(Point::ZERO, r, 0.0, 360.0);
        Path::single(SubPath {
            start,
            segments,
            closed: true,
        })
    }

    #[test]
    fn segments_serialize_as_number_arrays() {
        let path = Path::single(SubPath {
            start: Point::new(0.0, 0.0),
            segments: vec![
                Segment::Line(Point::new(10.0, 0.0)),
                Segment::Cubic(
                    Point::new(12.0, 0.0),
                    Point::new(12.0, 5.0),
                    Point::new(10.0, 5.0),
                ),
            ],
            closed: true,
        });
        let json = serde_json::to_string(&path).unwrap();
        assert_eq!(
            json,
            r#"{"subpaths":[{"start":[0.0,0.0],"segments":[[10.0,0.0],[12.0,0.0,12.0,5.0,10.0,5.0]],"closed":true}]}"#
        );
        assert_eq!(serde_json::from_str::<Path>(&json).unwrap(), path);
        assert!(serde_json::from_str::<Segment>("[1.0, 2.0, 3.0]").is_err());
    }

    #[test]
    fn circle_bounds_are_exact_and_flattening_is_within_tolerance() {
        let c = circle(10.0);
        let b = c.bounds().unwrap();
        assert!(b.min.approx_eq(Point::new(-10.0, -10.0), 1e-9), "{b:?}");
        assert!(b.max.approx_eq(Point::new(10.0, 10.0), 1e-9), "{b:?}");
        let (points, closed) = &c.flatten(0.01)[0];
        assert!(closed);
        // Cubic arcs deviate from a true circle by ~0.03%, so allow that too.
        for p in points {
            assert!((p.length() - 10.0).abs() < 0.01 + 0.003, "{p:?}");
        }
        assert!(points.len() > 20);
    }

    #[test]
    fn transform_keeps_curves_exact() {
        let c = circle(5.0).transformed(&(Affine::translate(20.0, 0.0) * Affine::scale(2.0, 1.0)));
        let b = c.bounds().unwrap();
        assert!(b.min.approx_eq(Point::new(10.0, -5.0), 1e-9));
        assert!(b.max.approx_eq(Point::new(30.0, 5.0), 1e-9));
    }

    #[test]
    fn validity() {
        assert!(circle(1.0).is_valid());
        assert!(!Path::default().is_valid());
        let mut bad = circle(1.0);
        bad.subpaths[0].start = Point::new(f64::NAN, 0.0);
        assert!(!bad.is_valid());
        let empty = Path::single(SubPath {
            start: Point::ZERO,
            segments: Vec::new(),
            closed: false,
        });
        assert!(!empty.is_valid());
    }

    #[test]
    fn polyline_subpath_flattens_to_its_points() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(5.0, 5.0),
        ];
        let s = SubPath::from_points(&pts, true).unwrap();
        assert_eq!(s.flatten(0.1), pts);
        assert!(SubPath::from_points(&[], false).is_none());
    }
}
