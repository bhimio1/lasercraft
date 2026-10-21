//! `arrange.*`: align, distribute, move together, move on the bed, grid
//! and circular arrays (`docs/spec/arrange.md`).

use lasercraft_geom::{Affine, Point, Rect};
use lasercraft_project::{Project, Shape};

use super::id_args;
use super::objects::{ids_and_anchor, move_shapes, objects_with_anchor};
use super::shape::place_copies;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

/// Most shapes one array may create.
pub const MAX_COPIES: usize = 10_000;

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "arrange.align",
            "arrange.align <left|right|top|bottom|center-x|center-y|center> [ids]",
            "Line objects up with the last-selected one.",
            align,
        ),
        cmd(
            "arrange.distribute",
            "arrange.distribute <h-spaced|h-centered|v-spaced|v-centered> [ids]",
            "Space objects evenly between the outermost two.",
            distribute,
        ),
        cmd(
            "arrange.move_together",
            "arrange.move_together <horizontal|vertical> [ids]",
            "Pack objects edge to edge around the last-selected one.",
            move_together,
        ),
        cmd(
            "arrange.move_to_bed",
            "arrange.move_to_bed <center|top-left|top|top-right|left|right|bottom-left|bottom|bottom-right> [ids]",
            "Move the selection to the work area's center, a corner or an edge.",
            move_to_bed,
        ),
        cmd(
            "arrange.grid",
            "arrange.grid <columns> <rows> <x_gap> <y_gap> [ids]",
            "Copy the selection into a grid with gaps between copies.",
            grid,
        ),
        cmd(
            "arrange.circular",
            "arrange.circular <count> <cx> <cy> <sweep_deg> <rotate|upright> [ids]",
            "Copy the selection around a center.",
            circular,
        ),
    ]
}

fn bounds(project: &Project, ids: &[u32]) -> Result<Rect, CommandError> {
    Ok(project
        .bounds_of(ids)?
        .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO)))
}

/// Moves each object by its delta as one step.
fn apply_moves(
    project: &mut Project,
    objects: &[Vec<u32>],
    deltas: &[Point],
    message: String,
) -> Result<Outcome, CommandError> {
    let moves: Vec<(u32, Point)> = objects
        .iter()
        .zip(deltas)
        .flat_map(|(o, d)| o.iter().map(move |id| (*id, *d)))
        .collect();
    Ok(match move_shapes(project, &moves)? {
        Some(inverse) => Outcome::undoable(message, inverse),
        None => Outcome::done("nothing to move"),
    })
}

fn align(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let how = args.str(0)?;
    if ![
        "left", "right", "top", "bottom", "center-x", "center-y", "center",
    ]
    .contains(&how)
    {
        return Err(args.invalid(format!("`{how}` is not a way to align")));
    }
    let (ids, anchor) = ids_and_anchor(args, 1, project)?;
    let (objects, anchor) = objects_with_anchor(project, &ids, anchor)?;
    if objects.len() < 2 {
        return Err(args.invalid("select at least two objects to align"));
    }
    let to = bounds(project, &objects[anchor])?;
    let mut deltas = Vec::with_capacity(objects.len());
    for object in &objects {
        let b = bounds(project, object)?;
        let (c, tc) = (b.center(), to.center());
        deltas.push(match how {
            "left" => Point::new(to.min.x - b.min.x, 0.0),
            "right" => Point::new(to.max.x - b.max.x, 0.0),
            "top" => Point::new(0.0, to.min.y - b.min.y),
            "bottom" => Point::new(0.0, to.max.y - b.max.y),
            "center-x" => Point::new(tc.x - c.x, 0.0),
            "center-y" => Point::new(0.0, tc.y - c.y),
            _ => tc - c,
        });
    }
    let n = objects.len();
    apply_moves(
        project,
        &objects,
        &deltas,
        format!("aligned {n} objects {how}"),
    )
}

/// Along one axis: (min, max) of a box.
fn span(b: &Rect, horizontal: bool) -> (f64, f64) {
    if horizontal {
        (b.min.x, b.max.x)
    } else {
        (b.min.y, b.max.y)
    }
}

fn along(d: f64, horizontal: bool) -> Point {
    if horizontal {
        Point::new(d, 0.0)
    } else {
        Point::new(0.0, d)
    }
}

/// Objects with their bounds, sorted by center along the axis (stable, so
/// ties keep drawing order).
fn sorted(
    project: &Project,
    objects: Vec<Vec<u32>>,
    horizontal: bool,
) -> Result<Vec<(Vec<u32>, Rect)>, CommandError> {
    let mut with: Vec<(Vec<u32>, Rect)> = objects
        .into_iter()
        .map(|o| bounds(project, &o).map(|b| (o, b)))
        .collect::<Result<_, _>>()?;
    with.sort_by(|a, b| {
        let (a0, a1) = span(&a.1, horizontal);
        let (b0, b1) = span(&b.1, horizontal);
        (a0 + a1).total_cmp(&(b0 + b1))
    });
    Ok(with)
}

fn distribute(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let how = args.str(0)?;
    let (horizontal, centered) = match how {
        "h-spaced" => (true, false),
        "h-centered" => (true, true),
        "v-spaced" => (false, false),
        "v-centered" => (false, true),
        other => return Err(args.invalid(format!("`{other}` is not a way to distribute"))),
    };
    let ids = args.ids(1, project)?;
    let objects = project.objects_of(&ids)?;
    if objects.len() < 3 {
        return Err(args.invalid("select at least three objects to distribute"));
    }
    let items = sorted(project, objects, horizontal)?;
    let n = items.len();
    let (first, last) = (
        span(&items[0].1, horizontal),
        span(&items[n - 1].1, horizontal),
    );
    let steps = (n - 1) as f64;
    let mut deltas = Vec::with_capacity(n);
    if centered {
        let (c0, c1) = ((first.0 + first.1) / 2.0, (last.0 + last.1) / 2.0);
        for (k, (_, b)) in items.iter().enumerate() {
            let (lo, hi) = span(b, horizontal);
            let target = c0 + (c1 - c0) * k as f64 / steps;
            let d = if k == 0 || k == n - 1 {
                0.0
            } else {
                target - (lo + hi) / 2.0
            };
            deltas.push(along(d, horizontal));
        }
    } else {
        let sizes: f64 = items
            .iter()
            .map(|(_, b)| span(b, horizontal))
            .map(|(a, b)| b - a)
            .sum();
        let gap = (last.1 - first.0 - sizes) / steps;
        let mut pos = first.0;
        for (k, (_, b)) in items.iter().enumerate() {
            let (lo, hi) = span(b, horizontal);
            let d = if k == 0 || k == n - 1 { 0.0 } else { pos - lo };
            deltas.push(along(d, horizontal));
            pos += hi - lo + gap;
        }
    }
    let objects: Vec<Vec<u32>> = items.into_iter().map(|(o, _)| o).collect();
    apply_moves(
        project,
        &objects,
        &deltas,
        format!("distributed {n} objects {how}"),
    )
}

fn move_together(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let horizontal = match args.str(0)? {
        "horizontal" | "h" => true,
        "vertical" | "v" => false,
        other => return Err(args.invalid(format!("`{other}` is not horizontal or vertical"))),
    };
    let (ids, anchor) = ids_and_anchor(args, 1, project)?;
    let (objects, anchor) = objects_with_anchor(project, &ids, anchor)?;
    if objects.len() < 2 {
        return Err(args.invalid("select at least two objects to move together"));
    }
    let anchor_object = objects[anchor].clone();
    let items = sorted(project, objects, horizontal)?;
    let a = items
        .iter()
        .position(|(o, _)| *o == anchor_object)
        .unwrap_or(0);
    let mut deltas = vec![Point::ZERO; items.len()];
    let (lo, hi) = span(&items[a].1, horizontal);
    let mut pos = hi;
    for k in a + 1..items.len() {
        let (l, h) = span(&items[k].1, horizontal);
        deltas[k] = along(pos - l, horizontal);
        pos += h - l;
    }
    let mut pos = lo;
    for k in (0..a).rev() {
        let (l, h) = span(&items[k].1, horizontal);
        deltas[k] = along(pos - h, horizontal);
        pos -= h - l;
    }
    let n = items.len();
    let objects: Vec<Vec<u32>> = items.into_iter().map(|(o, _)| o).collect();
    apply_moves(
        project,
        &objects,
        &deltas,
        format!("moved {n} objects together"),
    )
}

fn move_to_bed(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let place = args.str(0)?;
    let ids = args.ids(1, project)?;
    let b = bounds(project, &ids)?;
    let area = project.work_area();
    let (w, h) = (area.width_mm, area.height_mm);
    let (cx, cy) = ((w - b.width()) / 2.0, (h - b.height()) / 2.0);
    let (right, bottom) = (w - b.width(), h - b.height());
    let min = match place {
        "center" => Point::new(cx, cy),
        "top-left" => Point::new(0.0, 0.0),
        "top" => Point::new(cx, 0.0),
        "top-right" => Point::new(right, 0.0),
        "left" => Point::new(0.0, cy),
        "right" => Point::new(right, cy),
        "bottom-left" => Point::new(0.0, bottom),
        "bottom" => Point::new(cx, bottom),
        "bottom-right" => Point::new(right, bottom),
        other => return Err(args.invalid(format!("`{other}` is not a place on the bed"))),
    };
    let objects = vec![ids.clone()];
    apply_moves(
        project,
        &objects,
        &[min - b.min],
        format!("moved {} shape(s) to the bed's {place}", ids.len()),
    )
}

fn count(args: &Args<'_>, i: usize, what: &str) -> Result<usize, CommandError> {
    let n: usize = args.parse(i, what)?;
    if n == 0 || n > MAX_COPIES {
        return Err(args.invalid(format!("{what} must be 1–{MAX_COPIES}")));
    }
    Ok(n)
}

/// Copies the block `shapes` once per transform, as one step.
fn copies(
    project: &mut Project,
    args: &Args<'_>,
    ids: &[u32],
    placements: &[Affine],
    what: &str,
) -> Result<Outcome, CommandError> {
    if placements.len().saturating_mul(ids.len()) > MAX_COPIES {
        return Err(args.invalid(format!("an array makes at most {MAX_COPIES} copies")));
    }
    let shapes: Vec<Shape> = ids
        .iter()
        .filter_map(|id| project.shape(*id).cloned())
        .collect();
    let mut new_ids = Vec::new();
    for t in placements {
        match place_copies(project, &shapes, |old| *t * *old) {
            Ok(made) => new_ids.extend(made),
            Err(err) => {
                for id in &new_ids {
                    let _ = project.remove_shape(*id);
                }
                return Err(err);
            }
        }
    }
    project.set_selection(ids.iter().chain(&new_ids).copied())?;
    Ok(Outcome::undoable(
        format!(
            "{what}: {} cop(ies) of {} shape(s)",
            placements.len(),
            ids.len()
        ),
        Invocation::new("shape.delete", id_args(&new_ids)),
    ))
}

fn grid(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let columns = count(args, 0, "columns")?;
    let rows = count(args, 1, "rows")?;
    let gap = Point::new(args.f64(2, "x gap")?, args.f64(3, "y gap")?);
    if columns * rows < 2 {
        return Err(args.invalid("a grid needs at least two cells"));
    }
    let ids = args.ids(4, project)?;
    let b = bounds(project, &ids)?;
    let step = Point::new(b.width() + gap.x, b.height() + gap.y);
    let mut placements = Vec::new();
    for r in 0..rows {
        for c in 0..columns {
            if (r, c) != (0, 0) {
                placements.push(Affine::translate(step.x * c as f64, step.y * r as f64));
            }
        }
    }
    copies(project, args, &ids, &placements, "grid")
}

fn circular(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let n = count(args, 0, "count")?;
    if n < 2 {
        return Err(args.invalid("a circular array needs a count of at least 2"));
    }
    let center = args.point(1)?;
    let sweep = args.f64(3, "sweep")?;
    let rotate = match args.str(4)? {
        "rotate" => true,
        "upright" => false,
        other => return Err(args.invalid(format!("`{other}` is not rotate or upright"))),
    };
    let ids = args.ids(5, project)?;
    let step = if sweep.abs() == 360.0 {
        sweep / n as f64
    } else {
        sweep / (n - 1) as f64
    };
    let block_center = bounds(project, &ids)?.center();
    let placements: Vec<Affine> = (1..n)
        .map(|k| {
            let turn = Affine::rotate_about(step * k as f64, center);
            if rotate {
                turn
            } else {
                let d = turn.apply(block_center) - block_center;
                Affine::translate(d.x, d.y)
            }
        })
        .collect();
    copies(project, args, &ids, &placements, "circular array")
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::{Point, Rect};
    use lasercraft_project::Project;

    use crate::{Registry, Session};

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap_or_else(|e| panic!("{line}: {e}"));
        }
        s
    }

    fn b(s: &Session, id: u32) -> Rect {
        s.project().shape(id).unwrap().bounds()
    }

    fn near(p: Point, x: f64, y: f64) -> bool {
        p.approx_eq(Point::new(x, y), 1e-9)
    }

    const THREE: [&str; 3] = [
        "shape.add_rect 0 0 10 10",
        "shape.add_rect 30 20 20 5",
        "shape.add_rect 100 50 4 4",
    ];

    #[test]
    fn align_follows_the_anchor() {
        let mut s = session(&THREE);
        let before = s.project().clone();
        // Explicit ids: the last one is the anchor.
        s.run_line("arrange.align left 1 3 2").unwrap();
        assert_eq!((b(&s, 1).min.x, b(&s, 3).min.x), (30.0, 30.0));
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        // Selection: the last-added object.
        s.run_line("select.set 3").unwrap();
        s.run_line("select.add 1").unwrap();
        s.run_line("arrange.align bottom").unwrap();
        assert_eq!(b(&s, 3).max.y, 10.0);
        s.run_line("arrange.align center 1 2").unwrap();
        assert!(near(b(&s, 1).center(), 40.0, 22.5));
        for how in ["right", "top", "center-x", "center-y"] {
            s.run_line(&format!("arrange.align {how} 1 2 3")).unwrap();
        }
        assert!(s.run_line("arrange.align sideways 1 2").is_err());
        assert!(s.run_line("arrange.align left 1").is_err());
    }

    #[test]
    fn distribute_and_move_together() {
        let mut s = session(&THREE);
        s.run_line("arrange.distribute h-centered 1 2 3").unwrap();
        assert!((b(&s, 2).center().x - 53.5).abs() < 1e-9);
        s.run_line("edit.undo").unwrap();
        s.run_line("arrange.distribute h-spaced 1 2 3").unwrap();
        // Gaps: (104 - 0 - 34) / 2 = 35.
        assert_eq!(b(&s, 2).min.x, 45.0);
        s.run_line("arrange.distribute v-centered 1 2 3").unwrap();
        s.run_line("arrange.distribute v-spaced 1 2 3").unwrap();
        assert!(s.run_line("arrange.distribute h-spaced 1 2").is_err());
        let mut s = session(&THREE);
        s.run_line("arrange.move_together horizontal 1 3 2")
            .unwrap();
        assert_eq!((b(&s, 1).max.x, b(&s, 2).min.x), (30.0, 30.0));
        assert_eq!(b(&s, 3).min.x, 50.0);
        assert_eq!(b(&s, 3).min.y, 50.0, "other coordinate kept");
        s.run_line("arrange.move_together v 1 2").unwrap();
        assert!(s.run_line("arrange.move_together diagonal 1 2").is_err());
    }

    #[test]
    fn move_to_bed_places_the_block() {
        let mut s = session(&["shape.add_rect 0 0 10 10", "shape.add_rect 20 0 10 10"]);
        s.run_line("arrange.move_to_bed center 1 2").unwrap();
        assert!(near(b(&s, 1).min, 185.0, 195.0));
        s.run_line("arrange.move_to_bed bottom-right 1 2").unwrap();
        assert!(near(b(&s, 2).max, 400.0, 400.0));
        let depth = s.undo_depth();
        s.run_line("arrange.move_to_bed bottom-right 1 2").unwrap();
        assert_eq!(s.undo_depth(), depth, "already there: not recorded");
        for place in [
            "top-left",
            "top",
            "top-right",
            "left",
            "right",
            "bottom-left",
            "bottom",
        ] {
            s.run_line(&format!("arrange.move_to_bed {place} 1 2"))
                .unwrap();
        }
        assert!(s.run_line("arrange.move_to_bed moon 1").is_err());
    }

    #[test]
    fn grid_array_copies_with_gaps() {
        let mut s = session(&["shape.add_rect 0 0 10 5"]);
        let before = s.project().clone();
        s.run_line("arrange.grid 3 2 2 1 1").unwrap();
        assert_eq!(s.project().shapes().len(), 6);
        assert!(near(b(&s, 4).min, 0.0, 6.0));
        assert!(near(b(&s, 6).min, 24.0, 6.0));
        assert_eq!(s.project().selection().len(), 6);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        for bad in [
            "arrange.grid 1 1 0 0 1",
            "arrange.grid 0 2 0 0 1",
            "arrange.grid 200 200 0 0 1",
            "arrange.grid 2 2 x 0 1",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert_eq!(s.project(), &before);
    }

    #[test]
    fn circular_array_rotates_or_keeps_upright() {
        let mut s = session(&["shape.add_rect 45 0 10 10"]);
        s.run_line("arrange.circular 4 50 50 360 rotate 1").unwrap();
        assert!(near(b(&s, 3).center(), 50.0, 95.0));
        s.run_line("edit.undo").unwrap();
        s.run_line("arrange.circular 3 50 50 90 upright 1").unwrap();
        // Last copy at +90° (clockwise on screen): from above the center to its right.
        assert!(near(b(&s, 3).center(), 95.0, 50.0));
        let t = s.project().shape(3).unwrap().transform().m;
        assert_eq!((t[0], t[1]), (1.0, 0.0), "upright copies are only moved");
        assert!(s.run_line("arrange.circular 1 0 0 360 rotate 1").is_err());
        assert!(s.run_line("arrange.circular 3 0 0 360 spin 1").is_err());
    }
}
