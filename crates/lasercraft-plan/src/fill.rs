//! Fill mode: even-odd scan lines with direction, overscan and crosshatch
//! (`docs/spec/planning.md`).

use lasercraft_geom::Point;

use crate::emit::{Emitter, Feed};
use crate::order::Outline;

/// Segments shorter than this are dropped, mm.
const MIN_SEGMENT: f64 = 1e-9;

/// The rotated frame of a scan angle: lines run along `u`, successive lines
/// step along `s`.
#[derive(Debug, Clone, Copy)]
struct Frame {
    u: Point,
    s: Point,
}

impl Frame {
    fn new(angle_deg: f64) -> Self {
        let (sin, cos) = angle_deg.to_radians().sin_cos();
        Self {
            u: Point::new(cos, sin),
            s: Point::new(sin, -cos),
        }
    }

    fn coords(&self, p: Point) -> (f64, f64) {
        (
            p.x * self.u.x + p.y * self.u.y,
            p.x * self.s.x + p.y * self.s.y,
        )
    }

    fn point(&self, u: f64, s: f64) -> Point {
        self.u * u + self.s * s
    }
}

/// One burn segment on a scan line, from `u0` to `u1` (ascending), with the
/// shapes whose edges bound it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Segment {
    pub u0: f64,
    pub u1: f64,
    pub shape0: u32,
    pub shape1: u32,
}

/// A scan line that has something to burn.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScanLine {
    pub s: f64,
    pub segments: Vec<Segment>,
}

struct Edge {
    a: (f64, f64),
    b: (f64, f64),
    s_min: f64,
    s_max: f64,
    shape: u32,
}

/// Scan lines through the closed outlines at `angle_deg`, `interval` apart,
/// filled even-odd across all of them. Lines without segments are left out.
pub(crate) fn scan(outlines: &[Outline], angle_deg: f64, interval: f64) -> Vec<ScanLine> {
    let frame = Frame::new(angle_deg);
    let mut edges = Vec::new();
    for o in outlines.iter().filter(|o| o.closed && o.points.len() >= 3) {
        let n = o.points.len();
        for i in 0..n {
            let a = frame.coords(o.points[i]);
            let b = frame.coords(o.points[(i + 1) % n]);
            edges.push(Edge {
                a,
                b,
                s_min: a.1.min(b.1),
                s_max: a.1.max(b.1),
                shape: o.shape,
            });
        }
    }
    let Some(s_lo) = edges.iter().map(|e| e.s_min).reduce(f64::min) else {
        return Vec::new();
    };
    let s_hi = edges.iter().map(|e| e.s_max).fold(f64::MIN, f64::max);
    // Stable sort keeps drawing order among equal starts.
    edges.sort_by(|x, y| x.s_min.total_cmp(&y.s_min));
    let mut lines = Vec::new();
    let mut active: Vec<usize> = Vec::new();
    let mut next = 0;
    let mut k = 0_u32;
    loop {
        let s = s_lo + interval * (f64::from(k) + 0.5);
        if s >= s_hi {
            break;
        }
        k += 1;
        while next < edges.len() && edges[next].s_min <= s {
            active.push(next);
            next += 1;
        }
        active.retain(|&i| edges[i].s_max >= s);
        let mut crossings: Vec<(f64, u32, usize)> = active
            .iter()
            .filter_map(|&i| {
                let e = &edges[i];
                // Half-open: a vertex on the line counts once.
                ((e.a.1 > s) != (e.b.1 > s)).then(|| {
                    let t = (s - e.a.1) / (e.b.1 - e.a.1);
                    (e.a.0 + t * (e.b.0 - e.a.0), e.shape, i)
                })
            })
            .collect();
        crossings.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.2.cmp(&y.2)));
        let segments: Vec<Segment> = crossings
            .as_chunks::<2>()
            .0
            .iter()
            .filter(|[a, b]| b.0 - a.0 > MIN_SEGMENT)
            .map(|[a, b]| Segment {
                u0: a.0,
                u1: b.0,
                shape0: a.1,
                shape1: b.1,
            })
            .collect();
        if !segments.is_empty() {
            lines.push(ScanLine { s, segments });
        }
    }
    lines
}

/// Fill settings for one layer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FillSettings {
    pub angle: f64,
    pub interval: f64,
    pub bidirectional: bool,
    pub crosshatch: bool,
    /// Run-on at each line end, mm (0 for none).
    pub overscan: f64,
}

/// Emits one set of scan lines.
fn emit_lines(
    lines: &[ScanLine],
    frame: Frame,
    settings: &FillSettings,
    feed: &Feed,
    out: &mut Emitter,
) {
    for (k, line) in lines.iter().enumerate() {
        let forward = !settings.bidirectional || k % 2 == 0;
        let dir = if forward { 1.0 } else { -1.0 };
        // (start u, end u, shape at start, shape at end) in burn direction.
        let segments: Vec<(f64, f64, u32, u32)> = if forward {
            line.segments
                .iter()
                .map(|g| (g.u0, g.u1, g.shape0, g.shape1))
                .collect()
        } else {
            line.segments
                .iter()
                .rev()
                .map(|g| (g.u1, g.u0, g.shape1, g.shape0))
                .collect()
        };
        let (Some(first), Some(last)) = (segments.first(), segments.last()) else {
            continue;
        };
        let at = |u: f64| frame.point(u, line.s);
        out.travel(at(first.0 - dir * settings.overscan), feed, Some(first.2));
        for (u0, u1, shape0, _) in &segments {
            out.feed(false, at(*u0), feed, *shape0);
            out.feed(true, at(*u1), feed, *shape0);
        }
        out.feed(false, at(last.1 + dir * settings.overscan), feed, last.3);
    }
}

/// Plans every pass of a fill layer.
pub(crate) fn plan_layer(
    outlines: &[Outline],
    settings: &FillSettings,
    passes: u32,
    feed: Feed,
    out: &mut Emitter,
) {
    let mut angles = vec![settings.angle];
    if settings.crosshatch {
        angles.push(settings.angle + 90.0);
    }
    let sets: Vec<(Frame, Vec<ScanLine>)> = angles
        .iter()
        .map(|a| (Frame::new(*a), scan(outlines, *a, settings.interval)))
        .collect();
    for pass in 1..=passes {
        let feed = Feed { pass, ..feed };
        for (frame, lines) in &sets {
            emit_lines(lines, *frame, settings, &feed, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(shape: u32, x: f64, y: f64, s: f64) -> Outline {
        Outline {
            shape,
            points: vec![
                Point::new(x, y),
                Point::new(x + s, y),
                Point::new(x + s, y + s),
                Point::new(x, y + s),
            ],
            closed: true,
        }
    }

    #[test]
    fn horizontal_lines_work_from_the_bottom_up() {
        let lines = scan(&[square(1, 0.0, 0.0, 10.0)], 0.0, 2.0);
        assert_eq!(lines.len(), 5);
        let frame = Frame::new(0.0);
        let first = frame.point(lines[0].segments[0].u0, lines[0].s);
        // Bottom of the square is y = 10 (y grows downwards).
        assert!(first.approx_eq(Point::new(0.0, 9.0), 1e-9), "{first:?}");
        let last = frame.point(lines[4].segments[0].u1, lines[4].s);
        assert!(last.approx_eq(Point::new(10.0, 1.0), 1e-9), "{last:?}");
    }

    #[test]
    fn ninety_degrees_works_left_to_right() {
        let lines = scan(&[square(1, 0.0, 0.0, 10.0)], 90.0, 5.0);
        let frame = Frame::new(90.0);
        let xs: Vec<f64> = lines.iter().map(|l| frame.point(0.0, l.s).x).collect();
        assert!(
            (xs[0] - 2.5).abs() < 1e-9 && (xs[1] - 7.5).abs() < 1e-9,
            "{xs:?}"
        );
    }

    #[test]
    fn nested_and_overlapping_shapes_fill_even_odd() {
        // A ring: the hole's span is left out.
        let ring = scan(
            &[square(1, 0.0, 0.0, 10.0), square(2, 3.0, 3.0, 4.0)],
            0.0,
            1.0,
        );
        let middle = ring.iter().find(|l| (l.s + 5.5).abs() < 1e-9).unwrap();
        let spans: Vec<(f64, f64)> = middle.segments.iter().map(|g| (g.u0, g.u1)).collect();
        assert_eq!(spans, [(0.0, 3.0), (7.0, 10.0)]);
        assert_eq!(
            (middle.segments[1].shape0, middle.segments[1].shape1),
            (2, 1)
        );
        // Overlap of two squares on one layer stays unburned.
        let overlap = scan(
            &[square(1, 0.0, 0.0, 10.0), square(2, 5.0, 0.0, 10.0)],
            0.0,
            1.0,
        );
        let spans: Vec<(f64, f64)> = overlap[0].segments.iter().map(|g| (g.u0, g.u1)).collect();
        assert_eq!(spans, [(0.0, 5.0), (10.0, 15.0)]);
    }

    #[test]
    fn open_outlines_and_empty_input_scan_nothing() {
        let open = Outline {
            closed: false,
            ..square(1, 0.0, 0.0, 10.0)
        };
        assert!(scan(&[open], 0.0, 1.0).is_empty());
        assert!(scan(&[], 0.0, 1.0).is_empty());
    }
}
