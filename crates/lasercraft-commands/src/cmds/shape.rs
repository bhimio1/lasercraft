//! `shape.*`: create, delete, transform and reorder shapes
//! (`docs/spec/shapes.md`).

use lasercraft_geom::{Affine, Point};
use lasercraft_project::{Geometry, Project, Shape, validate_transform};

use super::{combine, id_args};
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "shape.add_rect",
            "shape.add_rect <x> <y> <width> <height> [@color]",
            "Add a rectangle with its top-left corner at (x, y).",
            add_rect,
        ),
        cmd(
            "shape.add_ellipse",
            "shape.add_ellipse <cx> <cy> <rx> <ry> [@color]",
            "Add an ellipse centered on (cx, cy).",
            add_ellipse,
        ),
        cmd(
            "shape.add_line",
            "shape.add_line <x1> <y1> <x2> <y2> [@color]",
            "Add a straight line.",
            add_line,
        ),
        cmd(
            "shape.add_polyline",
            "shape.add_polyline [closed] <x1> <y1> <x2> <y2> ... [@color]",
            "Add a polyline through two or more points.",
            add_polyline,
        ),
        cmd(
            "shape.insert",
            "shape.insert <index> <json>",
            "Insert a shape, keeping its id, at a drawing-order position.",
            insert,
        ),
        cmd(
            "shape.delete",
            "shape.delete [ids]",
            "Delete shapes (default: the selection).",
            delete,
        ),
        cmd(
            "shape.duplicate",
            "shape.duplicate [ids]",
            "Copy shapes in place, on top, and select the copies.",
            duplicate,
        ),
        cmd(
            "shape.move",
            "shape.move <dx> <dy> [ids]",
            "Move shapes by (dx, dy).",
            move_by,
        ),
        cmd(
            "shape.scale",
            "shape.scale <sx> <sy> [ids]",
            "Scale shapes about the center of their combined bounds.",
            scale,
        ),
        cmd(
            "shape.scale_about",
            "shape.scale_about <sx> <sy> <cx> <cy> [ids]",
            "Scale shapes about a point.",
            scale_about,
        ),
        cmd(
            "shape.rotate",
            "shape.rotate <degrees> [ids]",
            "Rotate shapes clockwise about the center of their combined bounds.",
            rotate,
        ),
        cmd(
            "shape.rotate_about",
            "shape.rotate_about <degrees> <cx> <cy> [ids]",
            "Rotate shapes clockwise about a point.",
            rotate_about,
        ),
        cmd(
            "shape.flip",
            "shape.flip <horizontal|vertical> [ids]",
            "Mirror shapes about the center of their combined bounds.",
            flip,
        ),
        cmd(
            "shape.set_transform",
            "shape.set_transform <id> <a> <b> <c> <d> <e> <f>",
            "Set a shape's transform matrix exactly.",
            set_transform,
        ),
        cmd(
            "shape.raise",
            "shape.raise [ids]",
            "Move shapes one step up in drawing order.",
            |p, a| reorder(p, a, Step::Raise),
        ),
        cmd(
            "shape.lower",
            "shape.lower [ids]",
            "Move shapes one step down in drawing order.",
            |p, a| reorder(p, a, Step::Lower),
        ),
        cmd(
            "shape.to_front",
            "shape.to_front [ids]",
            "Move shapes to the top of the drawing order.",
            |p, a| reorder(p, a, Step::Front),
        ),
        cmd(
            "shape.to_back",
            "shape.to_back [ids]",
            "Move shapes to the bottom of the drawing order.",
            |p, a| reorder(p, a, Step::Back),
        ),
        cmd(
            "shape.set_order",
            "shape.set_order <id> ...",
            "Set the drawing order; list every shape exactly once.",
            set_order,
        ),
    ]
}

/// Adds a shape, selects it, and returns an outcome whose inverse deletes it.
pub(crate) fn add(
    project: &mut Project,
    args: &Args<'_>,
    geometry: Geometry,
    transform: Affine,
) -> Result<Outcome, CommandError> {
    let kind = geometry.kind();
    let color = args.color_or_default()?;
    let id = project.add_shape_on(geometry, transform, color)?;
    project.set_selection([id])?;
    Ok(Outcome::undoable(
        format!("added {kind} {id}"),
        Invocation::new("shape.delete", [id.to_string()]),
    ))
}

fn add_rect(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 4)?;
    let corner = args.point(0)?;
    let (width, height) = (args.f64(2, "width")?, args.f64(3, "height")?);
    add(
        project,
        args,
        Geometry::Rect { width, height },
        Affine::translate(corner.x, corner.y),
    )
}

fn add_ellipse(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 4)?;
    let center = args.point(0)?;
    let (rx, ry) = (args.f64(2, "x radius")?, args.f64(3, "y radius")?);
    add(
        project,
        args,
        Geometry::Ellipse { rx, ry },
        Affine::translate(center.x, center.y),
    )
}

fn add_line(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 4)?;
    let points = vec![args.point(0)?, args.point(2)?];
    add(
        project,
        args,
        Geometry::Polyline {
            points,
            closed: false,
        },
        Affine::IDENTITY,
    )
}

fn add_polyline(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let closed = args.str(0)? == "closed";
    let first = usize::from(closed);
    let count = args.len() - first;
    if count < 4 || !count.is_multiple_of(2) {
        return Err(args.usage_error());
    }
    let points = (first..args.len())
        .step_by(2)
        .map(|i| args.point(i))
        .collect::<Result<Vec<Point>, _>>()?;
    add(
        project,
        args,
        Geometry::Polyline { points, closed },
        Affine::IDENTITY,
    )
}

fn insert(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let index: usize = args.parse(0, "index")?;
    let shape: Shape = serde_json::from_str(args.str(1)?)
        .map_err(|e| args.invalid(format!("not a shape: {e}")))?;
    let id = shape.id();
    project.insert_shape(index, shape)?;
    Ok(Outcome::undoable(
        format!("inserted shape {id}"),
        Invocation::new("shape.delete", [id.to_string()]),
    ))
}

/// Removes shapes and returns the inverse that re-inserts them exactly.
pub(crate) fn remove(project: &mut Project, ids: &[u32]) -> Result<Invocation, CommandError> {
    let mut removed = Vec::with_capacity(ids.len());
    // Remove from the top down so earlier indices stay valid.
    let mut by_index: Vec<(usize, u32)> = ids
        .iter()
        .filter_map(|id| project.index_of(*id).map(|i| (i, *id)))
        .collect();
    by_index.sort_unstable();
    for (_, id) in by_index.iter().rev() {
        removed.push(project.remove_shape(*id)?);
    }
    // Re-insert from the bottom up, each at its original index.
    let inverses = removed
        .iter()
        .rev()
        .map(|(index, shape)| {
            let json = serde_json::to_string(shape).unwrap_or_default();
            Invocation::new("shape.insert", [index.to_string(), json])
        })
        .collect();
    Ok(combine(inverses))
}

fn delete(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let inverse = remove(project, &ids)?;
    Ok(Outcome::undoable(
        format!("deleted {} shape(s)", ids.len()),
        inverse,
    ))
}

/// Adds copies of `shapes` on top, offset by `delta`, selects them, and
/// returns the outcome whose inverse deletes them.
pub(crate) fn add_copies(
    project: &mut Project,
    shapes: &[Shape],
    delta: Point,
    verb: &str,
) -> Result<Outcome, CommandError> {
    let moved = Affine::translate(delta.x, delta.y);
    let new_ids = place_copies(project, shapes, |t| moved * *t)?;
    project.set_selection(new_ids.iter().copied())?;
    Ok(Outcome::undoable(
        format!("{verb} {} shape(s)", new_ids.len()),
        Invocation::new("shape.delete", id_args(&new_ids)),
    ))
}

/// Adds copies of `shapes` on top with transforms `change(old)`, all or
/// nothing. Copies get fresh ids and fresh group ids (`docs/spec/arrange.md`).
pub(crate) fn place_copies(
    project: &mut Project,
    shapes: &[Shape],
    change: impl Fn(&Affine) -> Affine,
) -> Result<Vec<u32>, CommandError> {
    for shape in shapes {
        validate_transform(&change(shape.transform()))?;
    }
    let mut new_ids = Vec::with_capacity(shapes.len());
    for shape in shapes {
        let id = project.add_copy(shape);
        project.set_shape_transform(id, change(shape.transform()))?;
        new_ids.push(id);
    }
    project.regroup(&new_ids);
    Ok(new_ids)
}

fn duplicate(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let shapes: Vec<Shape> = ids
        .iter()
        .filter_map(|id| project.shape(*id).cloned())
        .collect();
    add_copies(project, &shapes, Point::ZERO, "duplicated")
}

/// Applies `change` to each shape's transform, all or nothing, with an exact
/// inverse.
pub(crate) fn transform_shapes(
    project: &mut Project,
    ids: &[u32],
    message: String,
    change: impl Fn(&Affine) -> Affine,
) -> Result<Outcome, CommandError> {
    let mut updates = Vec::with_capacity(ids.len());
    for id in ids {
        let old = *project
            .shape(*id)
            .ok_or_else(|| CommandError::InvalidArgument {
                command: "shape".to_owned(),
                reason: format!("shape {id} does not exist"),
            })?
            .transform();
        let new = change(&old);
        validate_transform(&new)?;
        updates.push((*id, old, new));
    }
    let mut inverses = Vec::with_capacity(updates.len());
    for (id, old, new) in updates {
        project.set_shape_transform(id, new)?;
        inverses.push(set_transform_invocation(id, &old));
    }
    Ok(Outcome::undoable(message, combine(inverses)))
}

pub(crate) fn set_transform_invocation(id: u32, t: &Affine) -> Invocation {
    Invocation::new(
        "shape.set_transform",
        std::iter::once(id.to_string()).chain(t.m.iter().map(ToString::to_string)),
    )
}

fn center_of(project: &Project, ids: &[u32]) -> Result<Point, CommandError> {
    Ok(project.bounds_of(ids)?.map_or(Point::ZERO, |b| b.center()))
}

fn scale_factors(args: &Args<'_>) -> Result<(f64, f64), CommandError> {
    let (sx, sy) = (args.f64(0, "x scale")?, args.f64(1, "y scale")?);
    if sx == 0.0 || sy == 0.0 {
        return Err(args.invalid("scale factors must not be zero"));
    }
    Ok((sx, sy))
}

fn move_by(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let delta = args.point(0)?;
    let ids = args.ids(2, project)?;
    let t = Affine::translate(delta.x, delta.y);
    transform_shapes(
        project,
        &ids,
        format!("moved {} shape(s) by ({}, {})", ids.len(), delta.x, delta.y),
        |old| t * *old,
    )
}

fn scale_about_point(
    project: &mut Project,
    ids: &[u32],
    (sx, sy): (f64, f64),
    c: Point,
) -> Result<Outcome, CommandError> {
    let t = Affine::translate(c.x, c.y) * Affine::scale(sx, sy) * Affine::translate(-c.x, -c.y);
    transform_shapes(
        project,
        ids,
        format!("scaled {} shape(s) by ({sx}, {sy})", ids.len()),
        |old| t * *old,
    )
}

fn scale(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let factors = scale_factors(args)?;
    let ids = args.ids(2, project)?;
    let c = center_of(project, &ids)?;
    scale_about_point(project, &ids, factors, c)
}

fn scale_about(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let factors = scale_factors(args)?;
    let c = args.point(2)?;
    let ids = args.ids(4, project)?;
    scale_about_point(project, &ids, factors, c)
}

fn rotate_about_point(
    project: &mut Project,
    ids: &[u32],
    degrees: f64,
    c: Point,
) -> Result<Outcome, CommandError> {
    let t = Affine::rotate_about(degrees, c);
    transform_shapes(
        project,
        ids,
        format!("rotated {} shape(s) by {degrees}°", ids.len()),
        |old| t * *old,
    )
}

fn rotate(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let degrees = args.f64(0, "angle")?;
    let ids = args.ids(1, project)?;
    let c = center_of(project, &ids)?;
    rotate_about_point(project, &ids, degrees, c)
}

fn rotate_about(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let degrees = args.f64(0, "angle")?;
    let c = args.point(1)?;
    let ids = args.ids(3, project)?;
    rotate_about_point(project, &ids, degrees, c)
}

fn flip(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let factors = match args.str(0)? {
        "horizontal" | "h" => (-1.0, 1.0),
        "vertical" | "v" => (1.0, -1.0),
        other => {
            return Err(args.invalid(format!("`{other}` is not horizontal or vertical")));
        }
    };
    let ids = args.ids(1, project)?;
    let c = center_of(project, &ids)?;
    scale_about_point(project, &ids, factors, c)
}

fn set_transform(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(7, 7)?;
    let id: u32 = args.parse(0, "shape id")?;
    if project.shape(id).is_none() {
        return Err(args.invalid(format!("shape {id} does not exist")));
    }
    let ids = [id];
    let mut m = [0.0; 6];
    for (i, v) in m.iter_mut().enumerate() {
        *v = args.f64(i + 1, "matrix value")?;
    }
    let t = Affine { m };
    transform_shapes(
        project,
        &ids,
        format!("set transform of shape {}", ids[0]),
        |_| t,
    )
}

#[derive(Clone, Copy)]
enum Step {
    Raise,
    Lower,
    Front,
    Back,
}

/// The drawing order after applying `step` to the shapes in `chosen`.
fn reordered(order: &[u32], chosen: &[u32], step: Step) -> Vec<u32> {
    let is_chosen = |id: &u32| chosen.contains(id);
    let mut order = order.to_vec();
    match step {
        Step::Raise => {
            for i in (0..order.len().saturating_sub(1)).rev() {
                if is_chosen(&order[i]) && !is_chosen(&order[i + 1]) {
                    order.swap(i, i + 1);
                }
            }
        }
        Step::Lower => {
            for i in 1..order.len() {
                if is_chosen(&order[i]) && !is_chosen(&order[i - 1]) {
                    order.swap(i, i - 1);
                }
            }
        }
        Step::Front | Step::Back => {
            let (mut picked, rest): (Vec<u32>, Vec<u32>) =
                order.iter().partition(|id| is_chosen(id));
            order = if matches!(step, Step::Front) {
                rest.into_iter().chain(picked).collect()
            } else {
                picked.extend(rest);
                picked
            };
        }
    }
    order
}

fn apply_order(project: &mut Project, order: &[u32]) -> Result<Outcome, CommandError> {
    let old = project.order();
    if old == order {
        return Ok(Outcome::done("drawing order unchanged"));
    }
    project.set_order(order)?;
    Ok(Outcome::undoable(
        "drawing order changed",
        Invocation::new("shape.set_order", id_args(&old)),
    ))
}

fn reorder(project: &mut Project, args: &Args<'_>, step: Step) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let order = reordered(&project.order(), &ids, step);
    apply_order(project, &order)
}

fn set_order(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let order = (0..args.len())
        .map(|i| args.parse::<u32>(i, "shape id"))
        .collect::<Result<Vec<_>, _>>()?;
    apply_order(project, &order)
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::Rect;

    use super::*;
    use crate::{Registry, Session};

    fn session() -> Session {
        Session::new(Project::default(), Registry::with_builtin())
    }

    fn bounds(s: &Session, id: u32) -> Rect {
        s.project().shape(id).unwrap().bounds()
    }

    fn assert_rect(r: Rect, min: (f64, f64), max: (f64, f64)) {
        let ok = r.min.approx_eq(Point::new(min.0, min.1), 1e-9)
            && r.max.approx_eq(Point::new(max.0, max.1), 1e-9);
        assert!(ok, "{r:?} != {min:?}..{max:?}");
    }

    #[test]
    fn adding_shapes_selects_them_and_undo_deletes() {
        let mut s = session();
        s.run_line("shape.add_rect 10 20 30 40").unwrap();
        s.run_line("shape.add_ellipse 100 100 10 5").unwrap();
        s.run_line("shape.add_line 0 0 10 10").unwrap();
        s.run_line("shape.add_polyline closed 0 0 10 0 5 8")
            .unwrap();
        assert_eq!(s.project().order(), [1, 2, 3, 4]);
        assert_eq!(s.project().selection(), [4]);
        assert_rect(bounds(&s, 1), (10.0, 20.0), (40.0, 60.0));
        assert_rect(bounds(&s, 2), (90.0, 95.0), (110.0, 105.0));
        assert_eq!(s.project().shape(4).unwrap().geometry().kind(), "polygon");
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().order(), [1, 2, 3]);
        s.run_line("edit.redo").unwrap();
        assert_eq!(s.project().order(), [1, 2, 3, 4], "redo reuses the id");
    }

    #[test]
    fn bad_shapes_are_refused() {
        let mut s = session();
        for bad in [
            "shape.add_rect 0 0 0 10",
            "shape.add_rect 0 0 10",
            "shape.add_ellipse 0 0 -1 1",
            "shape.add_line 0 0 nan 1",
            "shape.add_polyline 0 0",
            "shape.add_polyline closed 0 0 1",
            "shape.add_polyline 0 0 1 1 2",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert!(s.project().shapes().is_empty());
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn delete_and_undo_restore_order_and_ids() {
        let mut s = session();
        for x in [0, 20, 40, 60] {
            s.run_line(&format!("shape.add_rect {x} 0 10 10")).unwrap();
        }
        let before = s.project().clone();
        s.run_line("shape.delete 2 4 1").unwrap();
        assert_eq!(s.project().order(), [3]);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        s.run_line("select.clear").unwrap();
        let err = s.run_line("shape.delete").unwrap_err();
        assert!(err.to_string().contains("nothing selected"));
        assert!(s.run_line("shape.delete 99").is_err());
    }

    #[test]
    fn move_scale_rotate_flip_and_undo_exactly() {
        let mut s = session();
        s.run_line("shape.add_rect 0 0 20 10").unwrap();
        let original = s.project().clone();
        s.run_line("shape.move 5 -5").unwrap();
        assert_rect(bounds(&s, 1), (5.0, -5.0), (25.0, 5.0));
        s.run_line("shape.scale 2 3").unwrap();
        assert_rect(bounds(&s, 1), (-5.0, -15.0), (35.0, 15.0));
        s.run_line("shape.rotate 90").unwrap();
        assert_rect(bounds(&s, 1), (-0.0, -20.0), (30.0, 20.0));
        s.run_line("shape.flip horizontal").unwrap();
        assert_rect(bounds(&s, 1), (0.0, -20.0), (30.0, 20.0));
        s.run_line("shape.scale_about 0.5 0.5 0 0").unwrap();
        s.run_line("shape.rotate_about 0.1 7 7").unwrap();
        for _ in 0..6 {
            s.run_line("edit.undo").unwrap();
        }
        // Undo restores the exact matrix, not an approximation.
        assert_eq!(s.project(), &original);
        assert!(s.run_line("shape.scale 0 1").is_err());
        assert!(s.run_line("shape.flip sideways").is_err());
    }

    #[test]
    fn multi_shape_transforms_are_one_undo_step() {
        let mut s = session();
        s.run_line("shape.add_rect 0 0 10 10").unwrap();
        s.run_line("shape.add_rect 30 0 10 10").unwrap();
        s.run_line("select.all").unwrap();
        s.run_line("shape.rotate 90").unwrap();
        // Rotated about the center of the combined bounds (20, 5).
        assert_rect(bounds(&s, 1), (15.0, -15.0), (25.0, -5.0));
        assert_eq!(s.undo_depth(), 3);
        s.run_line("edit.undo").unwrap();
        assert_rect(bounds(&s, 1), (0.0, 0.0), (10.0, 10.0));
        assert_rect(bounds(&s, 2), (30.0, 0.0), (40.0, 10.0));
    }

    #[test]
    fn duplicate_copies_on_top_and_selects_copies() {
        let mut s = session();
        s.run_line("shape.add_rect 0 0 10 10").unwrap();
        s.run_line("shape.add_rect 30 0 10 10").unwrap();
        s.run_line("shape.duplicate 1").unwrap();
        assert_eq!(s.project().order(), [1, 2, 3]);
        assert_eq!(s.project().selection(), [3]);
        assert_eq!(bounds(&s, 3), bounds(&s, 1));
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().order(), [1, 2]);
    }

    #[test]
    fn set_transform_is_validated() {
        let mut s = session();
        s.run_line("shape.add_rect 0 0 10 10").unwrap();
        s.run_line("shape.set_transform 1 2 0 0 2 1 1").unwrap();
        assert_rect(bounds(&s, 1), (1.0, 1.0), (21.0, 21.0));
        assert!(s.run_line("shape.set_transform 1 0 0 0 0 1 1").is_err());
        assert!(s.run_line("shape.set_transform 9 1 0 0 1 0 0").is_err());
        assert!(s.run_line("shape.set_transform 1 1 0 0 1").is_err());
    }

    #[test]
    fn drawing_order_steps() {
        let order = [1, 2, 3, 4, 5];
        assert_eq!(reordered(&order, &[2, 3], Step::Raise), [1, 4, 2, 3, 5]);
        assert_eq!(reordered(&order, &[5], Step::Raise), order);
        assert_eq!(reordered(&order, &[2, 4], Step::Lower), [2, 1, 4, 3, 5]);
        assert_eq!(reordered(&order, &[1, 3], Step::Front), [2, 4, 5, 1, 3]);
        assert_eq!(reordered(&order, &[2, 5], Step::Back), [2, 5, 1, 3, 4]);
    }

    #[test]
    fn order_commands_undo_and_skip_no_ops() {
        let mut s = session();
        for _ in 0..3 {
            s.run_line("shape.add_rect 0 0 10 10").unwrap();
        }
        s.run_line("shape.to_back 3").unwrap();
        assert_eq!(s.project().order(), [3, 1, 2]);
        let depth = s.undo_depth();
        s.run_line("shape.to_back 3").unwrap();
        assert_eq!(s.undo_depth(), depth, "no-op is not recorded");
        s.run_line("shape.raise 3").unwrap();
        s.run_line("shape.lower 2").unwrap();
        s.run_line("shape.to_front 1").unwrap();
        s.run_line("edit.undo").unwrap();
        s.run_line("edit.undo").unwrap();
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().order(), [3, 1, 2]);
        assert!(s.run_line("shape.set_order 1 2").is_err());
        s.run_line("shape.set_order 2 1 3").unwrap();
        assert_eq!(s.project().order(), [2, 1, 3]);
    }

    #[test]
    fn insert_rejects_bad_json_and_duplicates() {
        let mut s = session();
        s.run_line("shape.add_rect 0 0 10 10").unwrap();
        let json = serde_json::to_string(s.project().shape(1).unwrap()).unwrap();
        assert!(
            s.run(&Invocation::new("shape.insert", ["0", &json]))
                .is_err()
        );
        assert!(
            s.run(&Invocation::new("shape.insert", ["0", "{}"]))
                .is_err()
        );
        let moved = json.replace("\"id\":1", "\"id\":7");
        s.run(&Invocation::new("shape.insert", ["0", &moved]))
            .unwrap();
        assert_eq!(s.project().order(), [7, 1]);
        s.run_line("shape.add_rect 0 0 1 1").unwrap();
        assert_eq!(
            s.project().order(),
            [7, 1, 8],
            "ids continue after inserted ones"
        );
    }
}
