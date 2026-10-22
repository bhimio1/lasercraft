//! The curve half of Trace to vectors (`docs/spec/trace.md`, "Method"
//! steps 4–5 and the grouping): fewer nodes, curves at gentle bends, and
//! holes put with the outline around them.

use crate::path::{Path, Segment, SubPath};
use crate::polyline::contains;
use crate::{Point, Rect};

/// The midpoints of every unit edge of a loop along whole-number corners
/// (straight runs between the given turning corners): a pixel staircase
/// becomes a straight line through them. A corner between two runs of 3
/// or more is kept itself, so square corners stay exact.
#[must_use]
pub fn edge_midpoints(corners: &[Point]) -> Vec<Point> {
    let n = corners.len();
    let run = |i: usize| corners[i].distance(corners[(i + 1) % n]);
    let (mut points, mut fixed) = (Vec::new(), Vec::new());
    for i in 0..n {
        let (a, b) = (corners[i], corners[(i + 1) % n]);
        // On a smooth curve one of two neighbouring runs is short.
        if run(i) >= 2.5 && run((i + n - 1) % n) >= 2.5 {
            points.push(a);
            fixed.push(true);
        }
        let steps = a.distance(b).round().max(1.0) as usize;
        let d = Point::new((b.x - a.x) / steps as f64, (b.y - a.y) / steps as f64);
        for k in 0..steps {
            let t = k as f64 + 0.5;
            points.push(Point::new(a.x + d.x * t, a.y + d.y * t));
            fixed.push(false);
        }
    }
    smooth_free(&points, &fixed)
}

/// Each point not `fixed` becomes the mean of itself and up to two
/// neighbours on either side, not reaching past a fixed point.
fn smooth_free(points: &[Point], fixed: &[bool]) -> Vec<Point> {
    let m = points.len();
    if m < 8 {
        return points.to_vec();
    }
    (0..m)
        .map(|i| {
            if fixed[i] {
                return points[i];
            }
            let (mut x, mut y, mut count) = (points[i].x, points[i].y, 1.0);
            for step in [1, m - 1] {
                for k in 1..=2 {
                    let j = (i + step * k) % m;
                    if fixed[j] {
                        break;
                    }
                    x += points[j].x;
                    y += points[j].y;
                    count += 1.0;
                }
            }
            Point::new(x / count, y / count)
        })
        .collect()
}

/// The loop's nodes after Douglas–Peucker with `tolerance`.
#[must_use]
pub fn simplify_loop(points: &[Point], tolerance: f64) -> Vec<Point> {
    let n = points.len();
    if n < 4 {
        return points.to_vec();
    }
    // Split at the node farthest from the first, simplify both halves.
    let far = (1..n)
        .max_by(|a, b| {
            points[0]
                .distance(points[*a])
                .total_cmp(&points[0].distance(points[*b]))
        })
        .unwrap_or(n / 2);
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[far] = true;
    let ring: Vec<usize> = (0..=n).map(|i| i % n).collect();
    reduce(points, &ring[..=far], tolerance, &mut keep);
    reduce(points, &ring[far..], tolerance, &mut keep);
    let kept: Vec<Point> = points
        .iter()
        .zip(&keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect();
    // A loop smaller than the tolerance keeps its shape.
    if kept.len() < 3 {
        points.to_vec()
    } else {
        kept
    }
}

fn reduce(points: &[Point], run: &[usize], tolerance: f64, keep: &mut [bool]) {
    if run.len() < 3 {
        return;
    }
    let (a, b) = (points[run[0]], points[run[run.len() - 1]]);
    let (mut worst, mut at) = (0.0, 0);
    for (k, i) in run.iter().enumerate().take(run.len() - 1).skip(1) {
        let d = distance_to_segment(points[*i], a, b);
        if d > worst {
            worst = d;
            at = k;
        }
    }
    if worst > tolerance {
        keep[run[at]] = true;
        reduce(points, &run[..=at], tolerance, keep);
        reduce(points, &run[at..], tolerance, keep);
    }
}

fn distance_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.x * ab.x + ab.y * ab.y;
    if len2 == 0.0 {
        return p.distance(a);
    }
    let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len2).clamp(0.0, 1.0);
    p.distance(Point::new(a.x + ab.x * t, a.y + ab.y * t))
}

fn unit(v: Point) -> Point {
    let len = v.x.hypot(v.y);
    if len == 0.0 {
        v
    } else {
        Point::new(v.x / len, v.y / len)
    }
}

/// A closed loop as a subpath: nodes turning by less than `60° ×
/// smoothness` are smooth (cubic segments with handles a third of the
/// segment along the tangent), the others are corners.
#[must_use]
pub fn smooth_loop(nodes: &[Point], smoothness: f64) -> Option<SubPath> {
    let n = nodes.len();
    if n < 3 {
        return None;
    }
    let limit = (60.0 * smoothness.clamp(0.0, 1.34)).to_radians();
    let tangent = |i: usize| -> Option<Point> {
        let (prev, here, next) = (nodes[(i + n - 1) % n], nodes[i], nodes[(i + 1) % n]);
        let (a, b) = (unit(here - prev), unit(next - here));
        let turn = (a.x * b.x + a.y * b.y).clamp(-1.0, 1.0).acos();
        (turn < limit).then(|| unit(next - prev))
    };
    let tangents: Vec<Option<Point>> = (0..n).map(tangent).collect();
    let segments = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            let (a, b) = (nodes[i], nodes[j]);
            match (tangents[i], tangents[j]) {
                (None, None) => Segment::Line(b),
                (ta, tb) => {
                    let third = a.distance(b) / 3.0;
                    let along = unit(b - a);
                    let (ta, tb) = (ta.unwrap_or(along), tb.unwrap_or(along));
                    Segment::Cubic(
                        Point::new(a.x + ta.x * third, a.y + ta.y * third),
                        Point::new(b.x - tb.x * third, b.y - tb.y * third),
                        b,
                    )
                }
            }
        })
        .collect();
    Some(SubPath {
        start: nodes[0],
        segments,
        closed: true,
    })
}

/// Twice the signed area (y down): positive for outer outlines.
fn area(points: &[Point]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum()
}

/// Paths from traced loops (positive area outer, negative holes): each
/// outer loop with the holes directly inside it, simplified with
/// `tolerance` and smoothed. Returns the paths and their node count.
#[must_use]
pub fn paths_from_loops(
    loops: &[Vec<Point>],
    tolerance: f64,
    smoothness: f64,
) -> (Vec<Path>, usize) {
    let mut outers: Vec<(usize, f64, Rect)> = Vec::new();
    let mut holes = Vec::new();
    for (i, l) in loops.iter().enumerate() {
        let Some(bounds) = Rect::bounding(l.iter().copied()) else {
            continue;
        };
        let a = area(l);
        if a > 0.0 {
            outers.push((i, a, bounds));
        } else if a < 0.0 {
            holes.push(i);
        }
    }
    // Smallest outer containing the hole's first node holds it.
    let mut inside: Vec<Vec<usize>> = vec![Vec::new(); outers.len()];
    for h in holes {
        let p = loops[h][0];
        let probe = Point::new(p.x + 0.25, p.y + 0.25);
        let holder = outers
            .iter()
            .enumerate()
            .filter(|(_, (o, _, b))| b.contains(p) && *o != h && contains(&loops[*o], probe))
            .min_by(|a, b| a.1.1.total_cmp(&b.1.1))
            .map(|(k, _)| k);
        if let Some(k) = holder {
            inside[k].push(h);
        }
    }
    let mut nodes = 0;
    let paths = outers
        .iter()
        .zip(inside)
        .filter_map(|((o, _, _), hs)| {
            let subpaths: Vec<SubPath> = std::iter::once(*o)
                .chain(hs)
                .filter_map(|i| {
                    let kept = simplify_loop(&loops[i], tolerance);
                    let s = smooth_loop(&kept, smoothness)?;
                    nodes += s.segments.len();
                    Some(s)
                })
                .collect();
            (!subpaths.is_empty()).then_some(Path { subpaths })
        })
        .collect();
    (paths, nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(v: &[(f64, f64)]) -> Vec<Point> {
        v.iter().map(|(x, y)| Point::new(*x, *y)).collect()
    }

    #[test]
    fn staircases_straighten_and_corners_stay() {
        // A 45° staircase edge of a triangle.
        let mut tri = vec![Point::new(0.0, 0.0)];
        for k in 0..10 {
            let k = f64::from(k);
            tri.push(Point::new(k + 1.0, k));
            tri.push(Point::new(k + 1.0, k + 1.0));
        }
        tri.push(Point::new(0.0, 10.0));
        let kept = simplify_loop(&tri, 0.95);
        assert_eq!(kept.len(), 3, "{kept:?}");
        // Through the edge midpoints the staircase is straight already.
        let mid = edge_midpoints(&tri);
        assert_eq!(mid.len(), 41, "40 midpoints and the square corner");
        // The diagonal is one line; the sharp corner where it starts with a
        // one-pixel step keeps a chamfer a third of a pixel deep.
        assert_eq!(simplify_loop(&mid, 0.5).len(), 4);
        let square = pts(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let s = smooth_loop(&square, 1.33).unwrap();
        assert!(s.segments.iter().all(|seg| matches!(seg, Segment::Line(_))));
        assert_eq!(s.segments.last().unwrap().end(), s.start);
        assert!(smooth_loop(&square, 0.0).is_some());
    }

    #[test]
    fn gentle_bends_become_curves() {
        let ring: Vec<Point> = (0..16)
            .map(|k| {
                let a = f64::from(k) * std::f64::consts::TAU / 16.0;
                Point::new(10.0 * a.cos(), 10.0 * a.sin())
            })
            .collect();
        let curved = smooth_loop(&ring, 1.0).unwrap();
        assert!(
            curved
                .segments
                .iter()
                .all(|s| matches!(s, Segment::Cubic(..)))
        );
        let straight = smooth_loop(&ring, 0.0).unwrap();
        assert!(
            straight
                .segments
                .iter()
                .all(|s| matches!(s, Segment::Line(_)))
        );
    }

    #[test]
    fn holes_join_the_outline_around_them() {
        let outer = pts(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let hole = pts(&[(3.0, 3.0), (3.0, 6.0), (6.0, 6.0), (6.0, 3.0)]);
        let other = pts(&[(20.0, 0.0), (22.0, 0.0), (22.0, 2.0), (20.0, 2.0)]);
        let (paths, nodes) = paths_from_loops(&[outer, hole, other], 0.5, 1.0);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].subpaths.len(), 2);
        assert_eq!(nodes, 12);
    }
}
