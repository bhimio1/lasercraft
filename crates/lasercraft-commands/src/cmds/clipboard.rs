//! `edit.copy`, `edit.cut`, `edit.paste*` (`docs/spec/shapes.md`). The
//! clipboard is per project and not saved.

use lasercraft_geom::{Point, Rect};
use lasercraft_project::{Project, Shape};

use super::shape::{add_copies, remove};
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "edit.copy",
            "edit.copy [ids]",
            "Copy shapes to the clipboard.",
            copy,
        ),
        cmd(
            "edit.cut",
            "edit.cut [ids]",
            "Copy shapes to the clipboard, then delete them.",
            cut,
        ),
        cmd(
            "edit.paste",
            "edit.paste [<x> <y>]",
            "Paste the clipboard, centered on (x, y) if given.",
            paste,
        ),
        cmd(
            "edit.paste_in_place",
            "edit.paste_in_place",
            "Paste the clipboard at its original position.",
            paste_in_place,
        ),
    ]
}

fn copied(project: &Project, ids: &[u32]) -> Vec<Shape> {
    ids.iter()
        .filter_map(|id| project.shape(*id).cloned())
        .collect()
}

fn copy(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    project.set_clipboard(copied(project, &ids));
    Ok(Outcome::done(format!("copied {} shape(s)", ids.len())))
}

fn cut(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    project.set_clipboard(copied(project, &ids));
    let inverse = remove(project, &ids)?;
    Ok(Outcome::undoable(
        format!("cut {} shape(s)", ids.len()),
        inverse,
    ))
}

fn clipboard(project: &Project, args: &Args<'_>) -> Result<Vec<Shape>, CommandError> {
    let shapes = project.clipboard().to_vec();
    if shapes.is_empty() {
        return Err(args.invalid("the clipboard is empty"));
    }
    Ok(shapes)
}

fn paste(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() == 0 {
        return paste_in_place(project, args);
    }
    args.expect(2, 2)?;
    let target = args.point(0)?;
    let shapes = clipboard(project, args)?;
    let center = shapes
        .iter()
        .map(Shape::bounds)
        .reduce(Rect::union)
        .map_or(Point::ZERO, |b| b.center());
    add_copies(project, &shapes, target - center, "pasted")
}

fn paste_in_place(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let shapes = clipboard(project, args)?;
    add_copies(project, &shapes, Point::ZERO, "pasted")
}

#[cfg(test)]
mod tests {
    use crate::{Registry, Session};

    use super::*;

    fn session() -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        s.run_line("shape.add_rect 0 0 10 10").unwrap();
        s
    }

    #[test]
    fn copy_then_paste_at_a_point_and_in_place() {
        let mut s = session();
        assert!(s.run_line("edit.paste").is_err(), "empty clipboard");
        s.run_line("edit.copy").unwrap();
        assert_eq!(s.undo_depth(), 1, "copy is not an edit");
        s.run_line("edit.paste 100 100").unwrap();
        let pasted = s.project().shape(2).unwrap().bounds();
        assert_eq!(pasted.center(), Point::new(100.0, 100.0));
        assert_eq!(s.project().selection(), [2]);
        s.run_line("edit.paste_in_place").unwrap();
        assert_eq!(s.project().shape(3).unwrap().bounds().min, Point::ZERO);
        s.run_line("edit.undo").unwrap();
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().order(), [1]);
    }

    #[test]
    fn cut_deletes_and_undo_restores() {
        let mut s = session();
        let before = s.project().clone();
        s.run_line("edit.cut 1").unwrap();
        assert!(s.project().shapes().is_empty());
        s.run_line("edit.paste_in_place").unwrap();
        assert_eq!(s.project().order(), [2], "pasted copies get new ids");
        s.run_line("edit.undo").unwrap();
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        assert!(s.run_line("edit.paste 1").is_err());
    }
}
