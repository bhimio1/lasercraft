//! Cut order within a layer: inner outlines first, then nearest neighbour
//! (`docs/spec/planning.md`, line mode).

use lasercraft_geom::{Point, Rect, polyline};

/// Most vertices sampled when testing whether one outline is inside another.
const SAMPLES: usize = 16;

/// An outline as the planner sees it: points, closed or open, its shape.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Outline {
    pub shape: u32,
    pub points: Vec<Point>,
    pub closed: bool,
}

impl Outline {
    pub fn bounds(&self) -> Rect {
        Rect::bounding(self.points.iter().copied())
            .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO))
    }
}

fn area(r: &Rect) -> f64 {
    r.width() * r.height()
}

/// Whether `a` lies inside the closed outline `b`: its box within `b`'s and
/// strictly smaller, and sampled vertices inside `b` (even-odd).
pub(crate) fn inside(a: &Outline, a_box: &Rect, b: &Outline, b_box: &Rect) -> bool {
    if !b.closed || !b_box.contains_rect(a_box) || area(a_box) >= area(b_box) {
        return false;
    }
    let n = a.points.len();
    let step = n.div_ceil(SAMPLES).max(1);
    a.points
        .iter()
        .step_by(step)
        .all(|p| polyline::contains(&b.points, *p))
}

/// For each outline, the indices of the closed outlines that contain it.
pub(crate) fn containers(outlines: &[Outline]) -> Vec<Vec<usize>> {
    let boxes: Vec<Rect> = outlines.iter().map(Outline::bounds).collect();
    (0..outlines.len())
        .map(|a| {
            (0..outlines.len())
                .filter(|&b| b != a && inside(&outlines[a], &boxes[a], &outlines[b], &boxes[b]))
                .collect()
        })
        .collect()
}

/// Where to enter an outline and which way to run it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    /// Index into the outlines.
    pub index: usize,
    /// Starting vertex (closed outlines; 0 for open ones).
    pub vertex: usize,
    /// Open outline run from its last point back to its first.
    pub reversed: bool,
}

/// The nearest way into an outline from `pos`, with its distance.
fn entry(outline: &Outline, index: usize, pos: Point) -> (f64, Entry) {
    if outline.closed {
        let mut best = (f64::INFINITY, 0);
        for (i, p) in outline.points.iter().enumerate() {
            let d = pos.distance(*p);
            if d < best.0 {
                best = (d, i);
            }
        }
        let e = Entry {
            index,
            vertex: best.1,
            reversed: false,
        };
        (best.0, e)
    } else {
        let first = outline
            .points
            .first()
            .map_or(f64::INFINITY, |p| pos.distance(*p));
        let last = outline
            .points
            .last()
            .map_or(f64::INFINITY, |p| pos.distance(*p));
        let reversed = last < first;
        let e = Entry {
            index,
            vertex: 0,
            reversed,
        };
        (first.min(last), e)
    }
}

/// Where the head ends after cutting `outline` entered as `e`.
pub(crate) fn exit(outline: &Outline, e: &Entry) -> Point {
    if outline.closed {
        outline.points[e.vertex]
    } else if e.reversed {
        outline.points[0]
    } else {
        outline.points[outline.points.len() - 1]
    }
}

/// Orders outlines from `start`: an outline becomes ready once everything
/// inside it is cut; the nearest ready one goes next.
pub(crate) fn order(outlines: &[Outline], start: Point) -> Vec<Entry> {
    let inside_of = containers(outlines);
    let n = outlines.len();
    // waiting[b]: outlines inside b not yet cut.
    let mut waiting = vec![0_usize; n];
    for containers in &inside_of {
        for &b in containers {
            waiting[b] += 1;
        }
    }
    let mut done = vec![false; n];
    let mut pos = start;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let mut best: Option<(f64, Entry)> = None;
        for i in (0..n).filter(|&i| !done[i] && waiting[i] == 0) {
            let (d, e) = entry(&outlines[i], i, pos);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, e));
            }
        }
        // The relation is acyclic (strictly smaller boxes), so something is
        // always ready.
        let Some((_, e)) = best else { break };
        done[e.index] = true;
        for &b in &inside_of[e.index] {
            waiting[b] -= 1;
        }
        pos = exit(&outlines[e.index], &e);
        out.push(e);
    }
    out
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

    fn line(shape: u32, a: (f64, f64), b: (f64, f64)) -> Outline {
        Outline {
            shape,
            points: vec![Point::new(a.0, a.1), Point::new(b.0, b.1)],
            closed: false,
        }
    }

    #[test]
    fn containment_needs_a_closed_strictly_larger_outline() {
        let outer = square(1, 0.0, 0.0, 10.0);
        let inner = square(2, 2.0, 2.0, 3.0);
        let dup = square(3, 0.0, 0.0, 10.0);
        let c = containers(&[outer, inner, dup, line(4, (1.0, 1.0), (9.0, 9.0))]);
        assert_eq!(
            c[0],
            Vec::<usize>::new(),
            "duplicates do not contain each other"
        );
        assert_eq!(c[1], vec![0, 2]);
        assert_eq!(c[3], vec![0, 2]);
        assert!(c[2].is_empty());
    }

    #[test]
    fn inner_outlines_are_cut_before_their_container() {
        // Starting right next to the big square, its hole still goes first.
        let outlines = [square(1, 0.0, 0.0, 100.0), square(2, 40.0, 40.0, 20.0)];
        let order = order(&outlines, Point::new(0.0, 0.0));
        assert_eq!(order.iter().map(|e| e.index).collect::<Vec<_>>(), [1, 0]);
        // Entered at the hole's nearest vertex (40, 40), then the outer
        // square at its vertex nearest to that.
        assert_eq!(order[0].vertex, 0);
        assert_eq!(order[1].vertex, 0);
    }

    #[test]
    fn nearest_neighbour_and_open_reversal() {
        let outlines = [
            square(1, 50.0, 0.0, 5.0),
            line(2, (30.0, 0.0), (10.0, 0.0)),
            square(3, 0.0, 0.0, 5.0),
        ];
        let order = order(&outlines, Point::new(0.0, 0.0));
        assert_eq!(order.iter().map(|e| e.index).collect::<Vec<_>>(), [2, 1, 0]);
        // From (0, 0) the line is entered at (10, 0), its last point.
        assert!(order[1].reversed);
        assert_eq!(exit(&outlines[1], &order[1]), Point::new(30.0, 0.0));
        // The far square is entered at its vertex nearest (30, 0).
        assert_eq!(order[2].vertex, 0);
    }
}
