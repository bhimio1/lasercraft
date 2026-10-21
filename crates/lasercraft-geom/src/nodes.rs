//! Node editing on paths (`docs/spec/nodes.md`).
//!
//! A subpath is edited in *node form*: its nodes (start and segment ends,
//! without repeating the start of a closed subpath) and one segment kind per
//! gap between nodes, including the closing segment of a closed subpath.
//! Every operation is pure: it returns a new [`Path`] or an error message.

use crate::path::cubic_at;
use crate::{Path, Point, Segment, SubPath};

/// The kind of segment leading to the next node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// Straight line.
    Line,
    /// Cubic Bézier with these two control points.
    Cubic(Point, Point),
}

/// A subpath as nodes and the segments between them.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeForm {
    /// Nodes in order; node 0 is the start.
    pub nodes: Vec<Point>,
    /// `segs[k]` runs from node k to node k + 1 (wrapping for closed).
    pub segs: Vec<Kind>,
    /// Whether the last node joins back to node 0.
    pub closed: bool,
}

impl NodeForm {
    /// Node form of a subpath.
    #[must_use]
    pub fn of(sub: &SubPath) -> Self {
        let mut nodes = vec![sub.start];
        let mut segs = Vec::with_capacity(sub.segments.len() + 1);
        for seg in &sub.segments {
            match *seg {
                Segment::Line(p) => {
                    segs.push(Kind::Line);
                    nodes.push(p);
                }
                Segment::Cubic(a, b, p) => {
                    segs.push(Kind::Cubic(a, b));
                    nodes.push(p);
                }
            }
        }
        if sub.closed {
            if nodes.len() > 1 && nodes.last() == Some(&sub.start) {
                nodes.pop();
            } else {
                segs.push(Kind::Line);
            }
        }
        Self {
            nodes,
            segs,
            closed: sub.closed,
        }
    }

    /// Back to a subpath; a straight closing segment stays implicit. `None`
    /// when nothing is left to draw.
    #[must_use]
    pub fn to_subpath(&self) -> Option<SubPath> {
        let n = self.nodes.len();
        let start = *self.nodes.first()?;
        let mut segments = Vec::with_capacity(self.segs.len());
        for (k, kind) in self.segs.iter().enumerate() {
            let end = self.nodes[(k + 1) % n];
            let last = k + 1 == self.segs.len();
            match *kind {
                Kind::Line if self.closed && last => {}
                Kind::Line => segments.push(Segment::Line(end)),
                Kind::Cubic(a, b) => segments.push(Segment::Cubic(a, b, end)),
            }
        }
        (!segments.is_empty()).then_some(SubPath {
            start,
            segments,
            closed: self.closed,
        })
    }

    /// The node a segment ends at.
    fn seg_end(&self, k: usize) -> Point {
        self.nodes[(k + 1) % self.nodes.len()]
    }

    /// The same outline traversed backwards.
    #[must_use]
    pub fn reversed(&self) -> Self {
        let mut nodes = self.nodes.clone();
        let mut segs: Vec<Kind> = self
            .segs
            .iter()
            .map(|k| match *k {
                Kind::Line => Kind::Line,
                Kind::Cubic(a, b) => Kind::Cubic(b, a),
            })
            .collect();
        if self.closed {
            // Keep node 0 first: nodes 0, n-1, …, 1; segment k of the result
            // is segment n-1-k of the original.
            nodes[1..].reverse();
            segs.reverse();
        } else {
            nodes.reverse();
            segs.reverse();
        }
        Self {
            nodes,
            segs,
            closed: self.closed,
        }
    }
}

/// Number of nodes in a subpath.
#[must_use]
pub fn node_count(sub: &SubPath) -> usize {
    NodeForm::of(sub).nodes.len()
}

/// Number of segments in a subpath, counting an implicit closing line.
#[must_use]
pub fn segment_count(sub: &SubPath) -> usize {
    NodeForm::of(sub).segs.len()
}

fn form(path: &Path, sub: usize) -> Result<NodeForm, String> {
    path.subpaths
        .get(sub)
        .map(NodeForm::of)
        .ok_or_else(|| format!("subpath {sub} does not exist"))
}

fn check_node(f: &NodeForm, node: usize) -> Result<(), String> {
    if node < f.nodes.len() {
        Ok(())
    } else {
        Err(format!("node {node} does not exist"))
    }
}

fn check_seg(f: &NodeForm, seg: usize) -> Result<(), String> {
    if seg < f.segs.len() {
        Ok(())
    } else {
        Err(format!("segment {seg} does not exist"))
    }
}

/// Replaces subpath `sub` by `parts` (dropping empty ones). Fails if the
/// path would be left empty.
fn replace(path: &Path, sub: usize, parts: &[NodeForm]) -> Result<Path, String> {
    let mut out = path.clone();
    let new: Vec<SubPath> = parts.iter().filter_map(NodeForm::to_subpath).collect();
    out.subpaths.splice(sub..=sub, new);
    if out.subpaths.is_empty() {
        return Err("that would remove the whole shape; delete the shape instead".to_owned());
    }
    Ok(out)
}

/// Moves a node to `to`; its handles move with it.
///
/// # Errors
///
/// Fails for a missing subpath or node.
pub fn move_node(path: &Path, sub: usize, node: usize, to: Point) -> Result<Path, String> {
    let mut f = form(path, sub)?;
    check_node(&f, node)?;
    let delta = to - f.nodes[node];
    f.nodes[node] = to;
    let n = f.nodes.len();
    let incoming = if node > 0 {
        Some(node - 1)
    } else if f.closed {
        Some(n - 1)
    } else {
        None
    };
    if let Some(k) = incoming
        && let Some(Kind::Cubic(_, b)) = f.segs.get_mut(k)
    {
        *b = *b + delta;
    }
    if let Some(Kind::Cubic(a, _)) = f.segs.get_mut(node) {
        *a = *a + delta;
    }
    replace(path, sub, &[f])
}

/// Moves handle `which` (1 or 2) of a cubic segment to `to`.
///
/// # Errors
///
/// Fails for a missing subpath or segment, a straight segment, or a handle
/// number other than 1 or 2.
pub fn set_handle(
    path: &Path,
    sub: usize,
    seg: usize,
    which: u8,
    to: Point,
) -> Result<Path, String> {
    let mut f = form(path, sub)?;
    check_seg(&f, seg)?;
    match (&mut f.segs[seg], which) {
        (Kind::Cubic(a, _), 1) => *a = to,
        (Kind::Cubic(_, b), 2) => *b = to,
        (Kind::Cubic(..), _) => return Err("a handle is 1 or 2".to_owned()),
        (Kind::Line, _) => return Err(format!("segment {seg} is not a curve")),
    }
    replace(path, sub, &[f])
}

/// One segment standing in for two that met at a removed node.
fn merged(first: Kind, second: Kind, from: Point, to: Point) -> Kind {
    match (first, second) {
        (Kind::Line, Kind::Line) => Kind::Line,
        _ => {
            let a = match first {
                Kind::Cubic(a, _) => a,
                Kind::Line => from,
            };
            let b = match second {
                Kind::Cubic(_, b) => b,
                Kind::Line => to,
            };
            Kind::Cubic(a, b)
        }
    }
}

/// Removes a node, joining its two segments.
///
/// # Errors
///
/// Fails for a missing subpath or node, or when nothing would be left.
pub fn delete_node(path: &Path, sub: usize, node: usize) -> Result<Path, String> {
    let mut f = form(path, sub)?;
    check_node(&f, node)?;
    let n = f.nodes.len();
    if f.closed {
        if n == 1 {
            return replace(path, sub, &[]);
        }
        let prev = (node + n - 1) % n;
        let from = f.nodes[prev];
        let to = f.seg_end(node);
        let joined = merged(f.segs[prev], f.segs[node], from, to);
        f.segs[prev] = joined;
        f.segs.remove(node);
        f.nodes.remove(node);
        if f.nodes.len() == 1 && f.segs[0] == Kind::Line {
            return replace(path, sub, &[]);
        }
    } else if node == 0 {
        f.nodes.remove(0);
        f.segs.remove(0);
    } else if node + 1 == n {
        f.nodes.pop();
        f.segs.pop();
    } else {
        let joined = merged(
            f.segs[node - 1],
            f.segs[node],
            f.nodes[node - 1],
            f.nodes[node + 1],
        );
        f.segs[node - 1] = joined;
        f.segs.remove(node);
        f.nodes.remove(node);
    }
    if f.segs.is_empty() {
        return replace(path, sub, &[]);
    }
    replace(path, sub, &[f])
}

/// Splits a cubic at `t` (de Casteljau): the control points of both halves
/// and the point between them.
#[must_use]
pub fn split_cubic(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    t: f64,
) -> ((Point, Point), Point, (Point, Point)) {
    let lerp = |a: Point, b: Point| a + (b - a) * t;
    let (p01, p12, p23) = (lerp(p0, p1), lerp(p1, p2), lerp(p2, p3));
    let (p012, p123) = (lerp(p01, p12), lerp(p12, p23));
    let m = lerp(p012, p123);
    ((p01, p012), m, (p123, p23))
}

/// Inserts a node in a segment at parameter `t` without changing the
/// outline.
///
/// # Errors
///
/// Fails for a missing subpath or segment or `t` outside (0, 1).
pub fn insert_node(path: &Path, sub: usize, seg: usize, t: f64) -> Result<Path, String> {
    if !(t > 0.0 && t < 1.0) {
        return Err(format!("t must be between 0 and 1, got {t}"));
    }
    let mut f = form(path, sub)?;
    check_seg(&f, seg)?;
    let from = f.nodes[seg];
    let to = f.seg_end(seg);
    let (first, mid, second) = match f.segs[seg] {
        Kind::Line => (Kind::Line, from + (to - from) * t, Kind::Line),
        Kind::Cubic(a, b) => {
            let (h1, m, h2) = split_cubic(from, a, b, to, t);
            (Kind::Cubic(h1.0, h1.1), m, Kind::Cubic(h2.0, h2.1))
        }
    };
    f.segs[seg] = first;
    f.segs.insert(seg + 1, second);
    f.nodes.insert(seg + 1, mid);
    replace(path, sub, &[f])
}

/// Makes a segment straight.
///
/// # Errors
///
/// Fails for a missing subpath or segment.
pub fn to_line(path: &Path, sub: usize, seg: usize) -> Result<Path, String> {
    let mut f = form(path, sub)?;
    check_seg(&f, seg)?;
    f.segs[seg] = Kind::Line;
    replace(path, sub, &[f])
}

/// Makes a straight segment a cubic with handles at a third and two thirds.
///
/// # Errors
///
/// Fails for a missing subpath or segment, or one that is already a curve.
pub fn to_curve(path: &Path, sub: usize, seg: usize) -> Result<Path, String> {
    let mut f = form(path, sub)?;
    check_seg(&f, seg)?;
    if matches!(f.segs[seg], Kind::Cubic(..)) {
        return Err(format!("segment {seg} is already a curve"));
    }
    let (from, to) = (f.nodes[seg], f.seg_end(seg));
    let third = (to - from) * (1.0 / 3.0);
    f.segs[seg] = Kind::Cubic(from + third, to - third);
    replace(path, sub, &[f])
}

/// Opens the path at a node: a closed subpath opens there, an open one is
/// split in two.
///
/// # Errors
///
/// Fails for a missing subpath or node, or an end node of an open subpath.
pub fn break_at(path: &Path, sub: usize, node: usize) -> Result<Path, String> {
    let f = form(path, sub)?;
    check_node(&f, node)?;
    if f.closed {
        let mut nodes: Vec<Point> = f.nodes[node..]
            .iter()
            .chain(&f.nodes[..node])
            .copied()
            .collect();
        nodes.push(f.nodes[node]);
        let segs = f.segs[node..]
            .iter()
            .chain(&f.segs[..node])
            .copied()
            .collect();
        return replace(
            path,
            sub,
            &[NodeForm {
                nodes,
                segs,
                closed: false,
            }],
        );
    }
    if node == 0 || node + 1 == f.nodes.len() {
        return Err(format!("node {node} is already an end of the path"));
    }
    let a = NodeForm {
        nodes: f.nodes[..=node].to_vec(),
        segs: f.segs[..node].to_vec(),
        closed: false,
    };
    let b = NodeForm {
        nodes: f.nodes[node..].to_vec(),
        segs: f.segs[node..].to_vec(),
        closed: false,
    };
    replace(path, sub, &[a, b])
}

/// Removes one segment, opening or splitting the subpath there.
///
/// # Errors
///
/// Fails for a missing subpath or segment, or when nothing would be left.
pub fn delete_segment(path: &Path, sub: usize, seg: usize) -> Result<Path, String> {
    let f = form(path, sub)?;
    check_seg(&f, seg)?;
    if f.closed {
        let n = f.nodes.len();
        let nodes: Vec<Point> = (1..=n).map(|i| f.nodes[(seg + i) % n]).collect();
        let segs: Vec<Kind> = (1..n).map(|i| f.segs[(seg + i) % n]).collect();
        return replace(
            path,
            sub,
            &[NodeForm {
                nodes,
                segs,
                closed: false,
            }],
        );
    }
    let a = NodeForm {
        nodes: f.nodes[..=seg].to_vec(),
        segs: f.segs[..seg].to_vec(),
        closed: false,
    };
    let b = NodeForm {
        nodes: f.nodes[seg + 1..].to_vec(),
        segs: f.segs[seg + 1..].to_vec(),
        closed: false,
    };
    replace(path, sub, &[a, b])
}

/// Closes open subpaths whose ends are within `tolerance`; returns the new
/// path and how many were closed.
#[must_use]
pub fn close_open(path: &Path, tolerance: f64) -> (Path, usize) {
    let mut out = path.clone();
    let mut count = 0;
    for sub in &mut out.subpaths {
        let end = sub.segments.last().map_or(sub.start, Segment::end);
        if !sub.closed && end.distance(sub.start) <= tolerance {
            sub.closed = true;
            count += 1;
        }
    }
    (out, count)
}

/// Joins open subpaths whose ends lie within `tolerance` into chains, and
/// closes chains whose ends meet. Closed subpaths pass through unchanged.
/// Deterministic: chains grow from the first unused subpath, taking the
/// first match in order.
#[must_use]
pub fn join(subpaths: &[SubPath], tolerance: f64) -> Vec<SubPath> {
    let mut out = Vec::new();
    let mut open: Vec<Option<NodeForm>> = Vec::new();
    for sub in subpaths {
        if sub.closed {
            out.push(sub.clone());
        } else {
            open.push(Some(NodeForm::of(sub)));
        }
    }
    let near = |a: Point, b: Point| a.distance(b) <= tolerance;
    for i in 0..open.len() {
        let Some(mut chain) = open[i].take() else {
            continue;
        };
        loop {
            let tail = *chain.nodes.last().unwrap_or(&Point::ZERO);
            let head = chain.nodes[0];
            let found = open.iter().position(|o| {
                o.as_ref().is_some_and(|o| {
                    let (s, e) = (o.nodes[0], *o.nodes.last().unwrap_or(&o.nodes[0]));
                    near(tail, s) || near(tail, e) || near(head, e) || near(head, s)
                })
            });
            let Some(j) = found else { break };
            let Some(other) = open[j].take() else { break };
            let (s, e) = (
                other.nodes[0],
                *other.nodes.last().unwrap_or(&other.nodes[0]),
            );
            if near(tail, s) {
                append(&mut chain, other);
            } else if near(tail, e) {
                append(&mut chain, other.reversed());
            } else if near(head, e) {
                let mut other = other;
                append(&mut other, chain);
                chain = other;
            } else {
                let mut other = other.reversed();
                append(&mut other, chain);
                chain = other;
            }
        }
        let n = chain.nodes.len();
        if n > 2 && near(chain.nodes[n - 1], chain.nodes[0]) {
            // The last node merges into the first; its segment closes.
            chain.nodes.pop();
            chain.closed = true;
        }
        out.extend(chain.to_subpath());
    }
    out
}

/// Appends `b` to `a`; `b`'s first node merges into `a`'s last.
fn append(a: &mut NodeForm, b: NodeForm) {
    a.nodes.extend(b.nodes.into_iter().skip(1));
    a.segs.extend(b.segs);
}

/// Every segment of a path as its own open one-segment subpath.
#[must_use]
pub fn segments_apart(path: &Path) -> Vec<SubPath> {
    let mut out = Vec::new();
    for sub in &path.subpaths {
        let f = NodeForm::of(sub);
        for (k, kind) in f.segs.iter().enumerate() {
            let end = f.seg_end(k);
            out.push(SubPath {
                start: f.nodes[k],
                segments: vec![match *kind {
                    Kind::Line => Segment::Line(end),
                    Kind::Cubic(a, b) => Segment::Cubic(a, b, end),
                }],
                closed: false,
            });
        }
    }
    out
}

/// A node's position: subpath, node number, point.
pub type NodeRef = (usize, usize, Point);

/// Every node of a path.
#[must_use]
pub fn nodes(path: &Path) -> Vec<NodeRef> {
    path.subpaths
        .iter()
        .enumerate()
        .flat_map(|(s, sub)| {
            NodeForm::of(sub)
                .nodes
                .into_iter()
                .enumerate()
                .map(move |(k, p)| (s, k, p))
        })
        .collect()
}

/// A handle: subpath, segment, handle number (1 or 2), the handle and the
/// node it belongs to.
pub type HandleRef = (usize, usize, u8, Point, Point);

/// Every handle of a path's cubic segments.
#[must_use]
pub fn handles(path: &Path) -> Vec<HandleRef> {
    let mut out = Vec::new();
    for (s, sub) in path.subpaths.iter().enumerate() {
        let f = NodeForm::of(sub);
        for (k, kind) in f.segs.iter().enumerate() {
            if let Kind::Cubic(a, b) = *kind {
                out.push((s, k, 1, a, f.nodes[k]));
                out.push((s, k, 2, b, f.seg_end(k)));
            }
        }
    }
    out
}

/// The segment nearest to `p`: subpath, segment, parameter and distance.
/// Curves are sampled at 64 points, which is plenty for picking.
#[must_use]
pub fn nearest_segment(path: &Path, p: Point) -> Option<(usize, usize, f64, f64)> {
    const SAMPLES: u32 = 64;
    let mut best: Option<(usize, usize, f64, f64)> = None;
    for (s, sub) in path.subpaths.iter().enumerate() {
        let f = NodeForm::of(sub);
        for (k, kind) in f.segs.iter().enumerate() {
            let (from, to) = (f.nodes[k], f.seg_end(k));
            for i in 1..SAMPLES {
                let t = f64::from(i) / f64::from(SAMPLES);
                let q = match *kind {
                    Kind::Line => from + (to - from) * t,
                    Kind::Cubic(a, b) => cubic_at(from, a, b, to, t),
                };
                let d = q.distance(p);
                if best.is_none_or(|b| d < b.3) {
                    best = Some((s, k, t, d));
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64) -> Point {
        Point::new(x, y)
    }

    fn square() -> Path {
        Path::single(
            SubPath::from_points(
                &[pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 10.0), pt(0.0, 10.0)],
                true,
            )
            .unwrap(),
        )
    }

    fn open_line() -> Path {
        Path::single(
            SubPath::from_points(
                &[pt(0.0, 0.0), pt(10.0, 0.0), pt(20.0, 0.0), pt(30.0, 0.0)],
                false,
            )
            .unwrap(),
        )
    }

    #[test]
    fn node_form_round_trips_and_counts_the_closing_segment() {
        let sq = square();
        let f = NodeForm::of(&sq.subpaths[0]);
        assert_eq!((f.nodes.len(), f.segs.len()), (4, 4));
        assert_eq!(f.to_subpath().unwrap(), sq.subpaths[0]);
        let (start, segments) = crate::path::arc(Point::ZERO, 5.0, 0.0, 360.0);
        let mut segments = segments;
        if let Some(Segment::Cubic(_, _, end)) = segments.last_mut() {
            *end = start;
        }
        let circle = SubPath {
            start,
            segments,
            closed: true,
        };
        let f = NodeForm::of(&circle);
        assert_eq!((f.nodes.len(), f.segs.len()), (4, 4));
        assert_eq!(f.to_subpath().unwrap(), circle);
        assert_eq!(segment_count(&open_line().subpaths[0]), 3);
        assert_eq!(node_count(&open_line().subpaths[0]), 4);
    }

    #[test]
    fn moving_nodes_and_handles() {
        let p = move_node(&square(), 0, 2, pt(12.0, 12.0)).unwrap();
        assert_eq!(p.subpaths[0].segments[1], Segment::Line(pt(12.0, 12.0)));
        assert!(move_node(&square(), 1, 0, pt(0.0, 0.0)).is_err());
        assert!(move_node(&square(), 0, 4, pt(0.0, 0.0)).is_err());
        let curved = to_curve(&square(), 0, 0).unwrap();
        // Moving node 0 drags handle 1 of segment 0 along.
        let moved = move_node(&curved, 0, 0, pt(1.0, 1.0)).unwrap();
        let Segment::Cubic(a, _, _) = moved.subpaths[0].segments[0] else {
            panic!("curve expected");
        };
        assert!(a.approx_eq(pt(10.0 / 3.0 + 1.0, 1.0), 1e-12));
        let h = set_handle(&curved, 0, 0, 2, pt(5.0, -5.0)).unwrap();
        assert!(matches!(h.subpaths[0].segments[0], Segment::Cubic(_, b, _) if b == pt(5.0, -5.0)));
        assert!(set_handle(&curved, 0, 1, 1, pt(0.0, 0.0)).is_err());
        assert!(set_handle(&curved, 0, 0, 3, pt(0.0, 0.0)).is_err());
    }

    #[test]
    fn insert_keeps_the_outline_and_delete_merges() {
        let p = insert_node(&square(), 0, 3, 0.5).unwrap();
        assert_eq!(node_count(&p.subpaths[0]), 5);
        assert_eq!(nodes(&p)[4].2, pt(0.0, 5.0));
        let back = delete_node(&p, 0, 4).unwrap();
        assert_eq!(back, square());
        assert!(insert_node(&square(), 0, 0, 1.0).is_err());
        // Splitting a curve keeps its shape exactly at the split point.
        let curved = to_curve(&square(), 0, 0).unwrap();
        let split = insert_node(&curved, 0, 0, 0.25).unwrap();
        let mid = nodes(&split)[1].2;
        assert!(mid.approx_eq(pt(2.5, 0.0), 1e-12));
        // Deleting the node again merges into one curve.
        let merged = delete_node(&split, 0, 1).unwrap();
        assert_eq!(segment_count(&merged.subpaths[0]), 4);
        // Ends of an open path drop their segment.
        let p = delete_node(&open_line(), 0, 0).unwrap();
        assert_eq!(p.subpaths[0].start, pt(10.0, 0.0));
        let p = delete_node(&open_line(), 0, 3).unwrap();
        assert_eq!(node_count(&p.subpaths[0]), 3);
        // The last segment of the only subpath cannot go.
        let two = Path::single(SubPath::from_points(&[pt(0.0, 0.0), pt(1.0, 0.0)], false).unwrap());
        assert!(delete_node(&two, 0, 0).is_err());
    }

    #[test]
    fn lines_and_curves_convert() {
        let c = to_curve(&square(), 0, 3).unwrap();
        // The closing segment became an explicit curve back to the start.
        assert_eq!(c.subpaths[0].segments.len(), 4);
        assert!(to_curve(&c, 0, 3).is_err());
        assert_eq!(to_line(&c, 0, 3).unwrap(), square());
    }

    #[test]
    fn break_and_delete_segment() {
        let b = break_at(&square(), 0, 2).unwrap();
        let s = &b.subpaths[0];
        assert!(!s.closed);
        assert_eq!(s.start, pt(10.0, 10.0));
        assert_eq!(s.segments.last().unwrap().end(), pt(10.0, 10.0));
        assert_eq!(s.segments.len(), 4);
        let b = break_at(&open_line(), 0, 1).unwrap();
        assert_eq!(b.subpaths.len(), 2);
        assert!(break_at(&open_line(), 0, 0).is_err());
        let d = delete_segment(&square(), 0, 0).unwrap();
        assert_eq!(d.subpaths[0].start, pt(10.0, 0.0));
        assert_eq!(d.subpaths[0].segments.len(), 3);
        assert_eq!(d.subpaths[0].segments[2], Segment::Line(pt(0.0, 0.0)));
        let d = delete_segment(&open_line(), 0, 1).unwrap();
        assert_eq!(d.subpaths.len(), 2);
        let d = delete_segment(&open_line(), 0, 0).unwrap();
        assert_eq!(d.subpaths.len(), 1);
    }

    #[test]
    fn joining_chains_reverses_and_closes() {
        let pieces = [
            SubPath::from_points(&[pt(0.0, 0.0), pt(10.0, 0.0)], false).unwrap(),
            SubPath::from_points(&[pt(10.0, 10.0), pt(10.0, 0.0)], false).unwrap(),
            SubPath::from_points(&[pt(0.0, 10.0), pt(0.0, 0.01)], false).unwrap(),
            SubPath::from_points(&[pt(10.0, 10.0), pt(0.0, 10.0)], false).unwrap(),
            SubPath::from_points(&[pt(50.0, 0.0), pt(60.0, 0.0)], false).unwrap(),
        ];
        let joined = join(&pieces, 0.05);
        assert_eq!(joined.len(), 2);
        assert!(joined[0].closed);
        assert_eq!(node_count(&joined[0]), 4);
        assert!(!joined[1].closed);
        let (closed, n) = close_open(&Path::single(pieces[0].clone()), 0.5);
        assert_eq!(n, 0);
        assert!(!closed.subpaths[0].closed);
    }

    #[test]
    fn apart_and_picking() {
        assert_eq!(segments_apart(&square()).len(), 4);
        let c = to_curve(&square(), 0, 0).unwrap();
        assert_eq!(handles(&c).len(), 2);
        let (s, k, t, d) = nearest_segment(&square(), pt(5.0, 10.2)).unwrap();
        assert_eq!((s, k), (0, 2));
        assert!((t - 0.5).abs() < 0.02 && (d - 0.2).abs() < 1e-9);
        let reversed = NodeForm::of(&c.subpaths[0]).reversed().reversed();
        assert_eq!(reversed, NodeForm::of(&c.subpaths[0]));
    }
}
