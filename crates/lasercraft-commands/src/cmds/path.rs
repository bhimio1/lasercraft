//! `path.union`, `path.subtract`, `path.intersect`, `path.weld`,
//! `path.offset` (`docs/spec/arrange.md`).

use lasercraft_geom::Path;
use lasercraft_geom::boolean::{self, BoolOp, Contours, Join};
use lasercraft_project::Project;

use super::objects::{bottom_layer, ids_and_anchor, objects_with_anchor, replace_with};
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Outcome};

/// Curves are flattened within this distance (mm) before booleans.
pub const TOLERANCE_MM: f64 = 0.01;

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "path.union",
            "path.union [ids]",
            "Merge two objects into one outline.",
            |p, a| two(p, a, BoolOp::Union),
        ),
        cmd(
            "path.subtract",
            "path.subtract [ids]",
            "Cut the last-selected object out of the other one.",
            |p, a| two(p, a, BoolOp::Subtract),
        ),
        cmd(
            "path.intersect",
            "path.intersect [ids]",
            "Keep only the area two objects share.",
            |p, a| two(p, a, BoolOp::Intersect),
        ),
        cmd(
            "path.weld",
            "path.weld [ids]",
            "Merge any number of shapes into one outline.",
            weld,
        ),
        cmd(
            "path.offset",
            "path.offset <distance> [round|miter|bevel] [ids]",
            "Add an outline grown (positive) or shrunk (negative) by a distance.",
            offset,
        ),
    ]
}

/// The closed contours of shapes, even-odd across all of them.
fn contours(
    project: &Project,
    ids: &[u32],
    args: &Args<'_>,
    what: &str,
) -> Result<Contours, CommandError> {
    let mut out = Vec::new();
    for id in ids {
        let shape = project
            .shape(*id)
            .ok_or_else(|| args.invalid(format!("shape {id} does not exist")))?;
        if !shape.geometry().is_closed() {
            return Err(args.invalid(format!("{what} need closed shapes; shape {id} is open")));
        }
        out.extend(shape.closed_contours(TOLERANCE_MM));
    }
    Ok(out)
}

fn result_path(contours: &Contours, args: &Args<'_>) -> Result<Path, CommandError> {
    Path::from_contours(contours).ok_or_else(|| args.invalid("the result is empty"))
}

fn two(project: &mut Project, args: &Args<'_>, op: BoolOp) -> Result<Outcome, CommandError> {
    let (ids, mut anchor) = ids_and_anchor(args, 0, project)?;
    if op == BoolOp::Subtract && args.len() == 0 {
        // The object selected second is the cutter; of two picked together,
        // the one drawn on top.
        anchor = project.second_selected();
    }
    let (objects, anchor) = objects_with_anchor(project, &ids, anchor)?;
    if objects.len() != 2 {
        return Err(args.invalid(format!("select exactly two objects, not {}", objects.len())));
    }
    // For subtract, the anchor is the cutter.
    let (keep, cut) = if anchor == 1 { (0, 1) } else { (1, 0) };
    let a = contours(project, &objects[keep], args, "booleans")?;
    let b = contours(project, &objects[cut], args, "booleans")?;
    let result = boolean::boolean(&a, &b, op);
    let path = result_path(&result, args)?;
    let all: Vec<u32> = objects.concat();
    let layer = if op == BoolOp::Subtract {
        bottom_layer(project, &objects[keep])
    } else {
        bottom_layer(project, &all)
    };
    let (new, inverse) = replace_with(project, &all, vec![(path, layer)])?;
    let verb = match op {
        BoolOp::Union => "united",
        BoolOp::Subtract => "subtracted",
        BoolOp::Intersect => "intersected",
        BoolOp::Xor => "combined",
    };
    Ok(Outcome::undoable(
        format!("{verb} {} shape(s) into shape {}", all.len(), new[0]),
        inverse,
    ))
}

/// Each shape even-odd on its own, then all merged.
fn welded(
    project: &Project,
    ids: &[u32],
    args: &Args<'_>,
    what: &str,
) -> Result<Contours, CommandError> {
    let shapes = ids
        .iter()
        .map(|id| contours(project, &[*id], args, what))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(boolean::union_all(&shapes))
}

fn weld(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let path = result_path(&welded(project, &ids, args, "booleans")?, args)?;
    let layer = bottom_layer(project, &ids);
    let (new, inverse) = replace_with(project, &ids, vec![(path, layer)])?;
    Ok(Outcome::undoable(
        format!("welded {} shape(s) into shape {}", ids.len(), new[0]),
        inverse,
    ))
}

fn offset(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let distance = args.f64(0, "distance")?;
    if distance.abs() > lasercraft_project::MAX_COORD_MM {
        return Err(args.invalid("distance is too large"));
    }
    let (join, from) = match args.str(1).ok() {
        Some("round") => (Join::Round(TOLERANCE_MM), 2),
        Some("miter") => (Join::Miter, 2),
        Some("bevel") => (Join::Bevel, 2),
        _ => (Join::Round(TOLERANCE_MM), 1),
    };
    let ids = args.ids(from, project)?;
    let merged = welded(project, &ids, args, "offset")?;
    let grown = boolean::offset(&merged, distance, join);
    let path = result_path(&grown, args)?;
    let layer = bottom_layer(project, &ids);
    let (new, inverse) = replace_with(project, &[], vec![(path, layer)])?;
    Ok(Outcome::undoable(
        format!(
            "offset {} shape(s) by {distance} mm into shape {}",
            ids.len(),
            new[0]
        ),
        inverse,
    ))
}

#[cfg(test)]
mod tests {
    use lasercraft_project::Project;

    use crate::{Registry, Session};

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap_or_else(|e| panic!("{line}: {e}"));
        }
        s
    }

    fn area(s: &Session, id: u32) -> f64 {
        let shape = s.project().shape(id).unwrap();
        shape
            .closed_contours(0.01)
            .iter()
            .map(|c| lasercraft_geom::boolean::area(c))
            .sum::<f64>()
            .abs()
    }

    const SQUARES: [&str; 2] = ["shape.add_rect 0 0 10 10 @2", "shape.add_rect 5 5 10 10 @4"];

    #[test]
    fn two_object_booleans_replace_their_inputs() {
        for (line, expected, layer) in [
            ("path.union 1 2", 175.0, 2),
            ("path.intersect 1 2", 25.0, 2),
            ("path.subtract 1 2", 75.0, 2),
            ("path.subtract 2 1", 75.0, 4),
        ] {
            let mut s = session(&SQUARES);
            let before = s.project().clone();
            s.run_line(line).unwrap();
            assert_eq!(s.project().order(), [3], "{line}");
            assert!(
                (area(&s, 3) - expected).abs() < 1e-6,
                "{line}: {}",
                area(&s, 3)
            );
            assert_eq!(s.project().shape(3).unwrap().layer(), layer, "{line}");
            assert_eq!(s.project().selection(), [3]);
            s.run_line("edit.undo").unwrap();
            assert_eq!(s.project(), &before, "{line}");
        }
    }

    #[test]
    fn subtract_uses_the_last_selected_object_as_the_cutter() {
        let mut s = session(&SQUARES);
        s.run_line("select.set 2").unwrap();
        s.run_line("select.add 1").unwrap();
        s.run_line("path.subtract").unwrap();
        assert_eq!(s.project().shape(3).unwrap().layer(), 4, "1 was cut from 2");
    }

    #[test]
    fn subtract_on_a_box_selection_cuts_the_top_shape_from_the_bottom_one() {
        // A plate with a hole drawn after it, both picked at once.
        let mut s = session(&[
            "shape.add_rect 0 0 60 40",
            "shape.add_ellipse 30 20 10 10",
            "select.in_rect -1 -1 61 41 contained",
        ]);
        s.run_line("path.subtract").unwrap();
        let plate = s.project().shapes().last().unwrap();
        assert_eq!(plate.closed_contours(0.01).len(), 2, "plate with a hole");
        assert_eq!(plate.bounds().width(), 60.0);
    }

    #[test]
    fn groups_count_as_one_even_odd_object() {
        let mut s = session(&[
            "shape.add_rect 0 0 20 20",
            "shape.add_rect 5 5 10 10",
            "group.create 1 2",
            "shape.add_rect 0 0 20 10",
        ]);
        s.run_line("path.intersect 1 4").unwrap();
        // The ring's top half: 200 minus the hole's top half (50).
        assert!((area(&s, 5) - 150.0).abs() < 1e-6, "{}", area(&s, 5));
    }

    #[test]
    fn weld_merges_and_swallows_inner_shapes() {
        let mut s = session(&[
            "shape.add_rect 0 0 20 20",
            "shape.add_rect 5 5 10 10",
            "shape.add_rect 15 0 10 10",
        ]);
        s.run_line("path.weld 1 2 3").unwrap();
        assert!((area(&s, 4) - 450.0).abs() < 1e-6, "{}", area(&s, 4));
    }

    #[test]
    fn offset_adds_an_outline_and_keeps_the_inputs() {
        let mut s = session(&["shape.add_rect 0 0 10 10"]);
        s.run_line("path.offset -1 miter").unwrap();
        assert_eq!(s.project().order(), [1, 2]);
        assert!((area(&s, 2) - 64.0).abs() < 1e-6);
        s.run_line("path.offset 2 1").unwrap();
        let b = s.project().shape(3).unwrap().bounds();
        assert!((b.width() - 14.0).abs() < 1e-6);
        assert!(s.run_line("path.offset -6 1").is_err(), "nothing left");
        assert!(s.run_line("path.offset x 1").is_err());
    }

    #[test]
    fn boolean_errors_leave_the_project_alone() {
        let mut s = session(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 50 0 10 10",
            "shape.add_line 0 0 5 5",
            "shape.add_rect 80 0 10 10",
        ]);
        let before = s.project().clone();
        for bad in [
            "path.union 1",
            "path.union 1 2 4",
            "path.union 1 3",
            "path.intersect 1 2",
            "path.weld 3",
            "path.offset 1 3",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert_eq!(s.project(), &before);
    }
}
