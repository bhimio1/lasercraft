//! `shape.to_path`, `shape.set_geometry`, `node.*` and the path clean-up
//! commands `path.close`, `path.join`, `path.break_apart`
//! (`docs/spec/nodes.md`).

use lasercraft_geom::{Path, Point, nodes};
use lasercraft_project::{Geometry, Project};

use super::combine;
use super::objects::{bottom_layer, doc_path, replace_with};
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

/// `path.close` closes ends within this distance (mm).
pub const CLOSE_TOLERANCE_MM: f64 = 0.5;
/// `path.join` joins ends within this distance (mm).
pub const JOIN_TOLERANCE_MM: f64 = 0.05;

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "shape.to_path",
            "shape.to_path [ids]",
            "Convert rectangles, ellipses and polylines to editable paths.",
            to_path,
        ),
        cmd(
            "shape.set_geometry",
            "shape.set_geometry <id> <json>",
            "Replace a shape's geometry exactly. Used by undo.",
            set_geometry,
        ),
        cmd(
            "node.move",
            "node.move <id> <sub> <node> <x> <y>",
            "Move a path node (and its handles) to a point.",
            |p, a| edit(p, a, 3, Edit::Move),
        ),
        cmd(
            "node.set_handle",
            "node.set_handle <id> <sub> <seg> <1|2> <x> <y>",
            "Move one handle of a curve segment to a point.",
            |p, a| edit(p, a, 4, Edit::Handle),
        ),
        cmd(
            "node.delete",
            "node.delete <id> <sub> <node>",
            "Delete a path node, joining its segments.",
            |p, a| edit(p, a, 3, Edit::Delete),
        ),
        cmd(
            "node.insert",
            "node.insert <id> <sub> <seg> [t]",
            "Insert a node into a segment (default: its middle).",
            |p, a| edit(p, a, 3, Edit::Insert),
        ),
        cmd(
            "node.to_line",
            "node.to_line <id> <sub> <seg>",
            "Make a path segment straight.",
            |p, a| edit(p, a, 3, Edit::ToLine),
        ),
        cmd(
            "node.to_curve",
            "node.to_curve <id> <sub> <seg>",
            "Make a straight path segment a curve.",
            |p, a| edit(p, a, 3, Edit::ToCurve),
        ),
        cmd(
            "node.break",
            "node.break <id> <sub> <node>",
            "Open a path at a node.",
            |p, a| edit(p, a, 3, Edit::Break),
        ),
        cmd(
            "node.delete_segment",
            "node.delete_segment <id> <sub> <seg>",
            "Delete one segment, opening or splitting the path.",
            |p, a| edit(p, a, 3, Edit::DeleteSegment),
        ),
        cmd(
            "path.close",
            "path.close [ids]",
            "Close open paths whose ends are within 0.5 mm.",
            close,
        ),
        cmd(
            "path.join",
            "path.join [ids]",
            "Join open paths whose ends meet into one shape.",
            join,
        ),
        cmd(
            "path.break_apart",
            "path.break_apart [ids]",
            "Split shapes into one shape per line or curve.",
            break_apart,
        ),
    ]
}

fn geometry_invocation(id: u32, g: &Geometry) -> Invocation {
    Invocation::new(
        "shape.set_geometry",
        [id.to_string(), serde_json::to_string(g).unwrap_or_default()],
    )
}

/// Replaces geometries, all validated first; returns the combined inverse.
fn set_geometries(
    project: &mut Project,
    changes: Vec<(u32, Geometry)>,
) -> Result<Invocation, CommandError> {
    for (_, g) in &changes {
        g.validate()?;
    }
    let mut inverses = Vec::with_capacity(changes.len());
    for (id, g) in changes {
        let old = project
            .shape(id)
            .map(|s| s.geometry().clone())
            .ok_or_else(|| CommandError::InvalidArgument {
                command: "shape.set_geometry".to_owned(),
                reason: format!("shape {id} does not exist"),
            })?;
        project.set_shape_geometry(id, g)?;
        inverses.push(geometry_invocation(id, &old));
    }
    Ok(combine(inverses))
}

fn to_path(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let changes: Vec<(u32, Geometry)> = ids
        .iter()
        .filter_map(|id| project.shape(*id))
        .filter(|s| !matches!(s.geometry(), Geometry::Path(_)))
        .map(|s| (s.id(), Geometry::Path(s.geometry().to_path())))
        .collect();
    if changes.is_empty() {
        return Err(args.invalid("nothing to convert: the shapes are already paths"));
    }
    let count = changes.len();
    let inverse = set_geometries(project, changes)?;
    Ok(Outcome::undoable(
        format!("converted {count} shape(s) to paths"),
        inverse,
    ))
}

fn set_geometry(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let id: u32 = args.parse(0, "shape id")?;
    if project.shape(id).is_none() {
        return Err(args.invalid(format!("shape {id} does not exist")));
    }
    let g: Geometry = serde_json::from_str(args.str(1)?)
        .map_err(|e| args.invalid(format!("not a geometry: {e}")))?;
    let inverse = set_geometries(project, vec![(id, g)])?;
    Ok(Outcome::undoable(
        format!("set geometry of shape {id}"),
        inverse,
    ))
}

#[derive(Clone, Copy)]
enum Edit {
    Move,
    Handle,
    Delete,
    Insert,
    ToLine,
    ToCurve,
    Break,
    DeleteSegment,
}

/// Runs one node edit on shape `<id>`: `fixed` arguments are id, subpath
/// and node/segment numbers; the rest depend on the edit.
fn edit(
    project: &mut Project,
    args: &Args<'_>,
    fixed: usize,
    what: Edit,
) -> Result<Outcome, CommandError> {
    let extra = match what {
        Edit::Move => (2, 2),
        Edit::Handle => (2, 2),
        Edit::Insert => (0, 1),
        _ => (0, 0),
    };
    args.expect(fixed + extra.0, fixed + extra.1)?;
    let id: u32 = args.parse(0, "shape id")?;
    let sub: usize = args.parse(1, "subpath number")?;
    let at: usize = args.parse(2, "node or segment number")?;
    let shape = project
        .shape(id)
        .ok_or_else(|| args.invalid(format!("shape {id} does not exist")))?;
    let Geometry::Path(path) = shape.geometry() else {
        return Err(args.invalid(format!(
            "shape {id} is a {}; convert the shape to a path first",
            shape.geometry().kind()
        )));
    };
    // Document positions map into the shape's own coordinates.
    let inverse = shape.transform().inverse();
    let local = |p: Point| inverse.map_or(p, |t| t.apply(p));
    let result: Result<Path, String> = match what {
        Edit::Move => nodes::move_node(path, sub, at, local(args.point(3)?)),
        Edit::Handle => {
            let which: u8 = args.parse(3, "handle number")?;
            nodes::set_handle(path, sub, at, which, local(args.point(4)?))
        }
        Edit::Delete => nodes::delete_node(path, sub, at),
        Edit::Insert => {
            let t = if args.len() > 3 {
                args.f64(3, "t")?
            } else {
                0.5
            };
            nodes::insert_node(path, sub, at, t)
        }
        Edit::ToLine => nodes::to_line(path, sub, at),
        Edit::ToCurve => nodes::to_curve(path, sub, at),
        Edit::Break => nodes::break_at(path, sub, at),
        Edit::DeleteSegment => nodes::delete_segment(path, sub, at),
    };
    let new = result.map_err(|reason| args.invalid(reason))?;
    let inverse = set_geometries(project, vec![(id, Geometry::Path(new))])?;
    Ok(Outcome::undoable(
        format!("edited nodes of shape {id}"),
        inverse,
    ))
}

fn close(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let mut changes = Vec::new();
    let mut count = 0;
    for shape in ids.iter().filter_map(|id| project.shape(*id)) {
        let new = match shape.geometry() {
            Geometry::Path(path) => {
                let (closed, n) = nodes::close_open(path, CLOSE_TOLERANCE_MM);
                count += n;
                (n > 0).then_some(Geometry::Path(closed))
            }
            Geometry::Polyline {
                points,
                closed: false,
            } if points.len() >= 3 => {
                let (first, last) = (points[0], points[points.len() - 1]);
                (first.distance(last) <= CLOSE_TOLERANCE_MM).then(|| {
                    count += 1;
                    let mut points = points.clone();
                    if first == last {
                        points.pop();
                    }
                    Geometry::Polyline {
                        points,
                        closed: true,
                    }
                })
            }
            _ => None,
        };
        changes.extend(new.map(|g| (shape.id(), g)));
    }
    if changes.is_empty() {
        return Err(args.invalid("no open path has its ends within 0.5 mm"));
    }
    let inverse = set_geometries(project, changes)?;
    Ok(Outcome::undoable(
        format!("closed {count} path(s)"),
        inverse,
    ))
}

fn join(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let subpaths: Vec<_> = ids
        .iter()
        .filter_map(|id| project.shape(*id))
        .flat_map(|s| doc_path(s).subpaths)
        .collect();
    let joined = nodes::join(&subpaths, JOIN_TOLERANCE_MM);
    if joined.len() == subpaths.len() {
        return Err(args.invalid("no open ends meet within 0.05 mm"));
    }
    let layer = bottom_layer(project, &ids);
    let path = Path { subpaths: joined };
    let (new, inverse) = replace_with(project, &ids, vec![(path, layer)])?;
    Ok(Outcome::undoable(
        format!(
            "joined {} subpath(s) into {} in shape {}",
            subpaths.len(),
            project.shape(new[0]).map_or(0, |s| match s.geometry() {
                Geometry::Path(p) => p.subpaths.len(),
                _ => 0,
            }),
            new[0]
        ),
        inverse,
    ))
}

fn break_apart(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    for shape in ids.iter().filter_map(|id| project.shape(*id)) {
        let pieces = nodes::segments_apart(&doc_path(shape));
        if pieces.len() > 1 {
            inputs.push(shape.id());
            outputs.extend(pieces.into_iter().map(|p| (Path::single(p), shape.layer())));
        }
    }
    if inputs.is_empty() {
        return Err(args.invalid("nothing to break apart: every shape is a single segment"));
    }
    let (new, inverse) = replace_with(project, &inputs, outputs)?;
    Ok(Outcome::undoable(
        format!("broke {} shape(s) into {} pieces", inputs.len(), new.len()),
        inverse,
    ))
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::{Point, Segment};
    use lasercraft_project::{Geometry, Project};

    use crate::{Registry, Session};

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap_or_else(|e| panic!("{line}: {e}"));
        }
        s
    }

    fn path(s: &Session, id: u32) -> lasercraft_geom::Path {
        match s.project().shape(id).unwrap().geometry() {
            Geometry::Path(p) => p.clone(),
            other => panic!("not a path: {other:?}"),
        }
    }

    #[test]
    fn to_path_keeps_the_outline_and_undoes() {
        let mut s = session(&["shape.add_rect 10 10 20 10", "shape.add_ellipse 50 50 5 5"]);
        let before = s.project().clone();
        let b = s.project().shape(2).unwrap().bounds();
        s.run_line("shape.to_path 1 2").unwrap();
        assert_eq!(path(&s, 1).subpaths[0].segments.len(), 3);
        let after = s.project().shape(2).unwrap().bounds();
        assert!(after.min.approx_eq(b.min, 1e-9) && after.max.approx_eq(b.max, 1e-9));
        assert!(s.run_line("shape.to_path 1").is_err(), "already a path");
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
    }

    #[test]
    fn node_edits_use_document_coordinates() {
        let mut s = session(&["shape.add_rect 10 10 20 10", "shape.to_path"]);
        // Node 2 is the bottom-right corner (30, 20) in document space.
        s.run_line("node.move 1 0 2 40 25").unwrap();
        assert_eq!(
            path(&s, 1).subpaths[0].segments[1],
            Segment::Line(Point::new(30.0, 15.0))
        );
        s.run_line("node.insert 1 0 0").unwrap();
        assert_eq!(
            path(&s, 1).subpaths[0].segments[0],
            Segment::Line(Point::new(10.0, 0.0))
        );
        s.run_line("node.to_curve 1 0 0").unwrap();
        s.run_line("node.set_handle 1 0 0 1 12 5").unwrap();
        let Segment::Cubic(a, _, _) = path(&s, 1).subpaths[0].segments[0] else {
            panic!("curve expected");
        };
        assert_eq!(a, Point::new(2.0, -5.0));
        s.run_line("node.to_line 1 0 0").unwrap();
        s.run_line("node.delete 1 0 1").unwrap();
        s.run_line("node.break 1 0 0").unwrap();
        assert!(!path(&s, 1).is_closed());
        s.run_line("node.delete_segment 1 0 0").unwrap();
        assert_eq!(s.undo_depth(), 10);
        for _ in 0..8 {
            s.run_line("edit.undo").unwrap();
        }
        assert_eq!(path(&s, 1).subpaths[0].segments.len(), 3);
    }

    #[test]
    fn node_errors_leave_the_shape_alone() {
        let mut s = session(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 0 0 5 5",
            "shape.to_path 2",
        ]);
        let before = s.project().clone();
        for bad in [
            "node.move 1 0 0 1 1",
            "node.move 2 1 0 1 1",
            "node.move 2 0 9 1 1",
            "node.move 2 0 0 1",
            "node.set_handle 2 0 0 1 1 1",
            "node.set_handle 2 0 0 x 1 1",
            "node.insert 2 0 0 1.5",
            "node.break 9 0 0",
            "node.to_curve 2 0 7",
            "shape.set_geometry 2 {}",
            "shape.set_geometry 9 {}",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert_eq!(s.project(), &before);
        assert_eq!(s.undo_depth(), 3);
    }

    #[test]
    fn close_join_and_break_apart() {
        let mut s = session(&[
            "shape.add_polyline 0 0 10 0 10 10 0 10 0 0.3",
            "shape.add_line 50 0 60 0",
            "shape.add_line 60 0 60 10",
        ]);
        s.run_line("path.close 1").unwrap();
        assert!(s.project().shape(1).unwrap().geometry().is_closed());
        assert!(s.run_line("path.close 2").is_err());
        let before = s.project().clone();
        s.run_line("path.join 2 3").unwrap();
        assert_eq!(s.project().order(), [1, 4]);
        assert_eq!(path(&s, 4).subpaths.len(), 1);
        assert!(s.run_line("path.join 1").is_err());
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        s.run_line("path.break_apart 1").unwrap();
        assert_eq!(s.project().order(), [2, 3, 4, 5, 6, 7, 8]);
        assert!(s.run_line("path.break_apart 2").is_err());
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
    }
}
