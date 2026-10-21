//! `select.*`: change the selection. Not undoable (`docs/spec/shapes.md`).

use std::collections::BTreeSet;

use lasercraft_geom::Rect;
use lasercraft_project::{BoxMode, Project};

use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "select.set",
            "select.set <id> ...",
            "Select exactly these shapes.",
            |p, a| change(p, a, Combine::Replace),
        ),
        cmd(
            "select.add",
            "select.add <id> ...",
            "Add shapes to the selection.",
            |p, a| change(p, a, Combine::Add),
        ),
        cmd(
            "select.remove",
            "select.remove <id> ...",
            "Remove shapes from the selection.",
            |p, a| change(p, a, Combine::Remove),
        ),
        cmd(
            "select.toggle",
            "select.toggle <id> ...",
            "Toggle shapes in or out of the selection.",
            |p, a| change(p, a, Combine::Toggle),
        ),
        cmd("select.clear", "select.clear", "Select nothing.", clear),
        cmd("select.all", "select.all", "Select every shape.", all),
        cmd(
            "select.invert",
            "select.invert",
            "Select exactly the shapes that are not selected.",
            invert,
        ),
        cmd(
            "select.at",
            "select.at <x> <y> <tolerance> [replace|add|toggle]",
            "Select the topmost shape at a point.",
            at,
        ),
        cmd(
            "select.in_rect",
            "select.in_rect <x0> <y0> <x1> <y1> <contained|touching> [replace|add|toggle]",
            "Select shapes inside or touching a box.",
            in_rect,
        ),
        cmd(
            "select.next",
            "select.next",
            "Select the next shape in drawing order.",
            |p, a| cycle(p, a, true),
        ),
        cmd(
            "select.previous",
            "select.previous",
            "Select the previous shape in drawing order.",
            |p, a| cycle(p, a, false),
        ),
    ]
}

#[derive(Clone, Copy)]
enum Combine {
    Replace,
    Add,
    Remove,
    Toggle,
}

impl Combine {
    fn parse(args: &Args<'_>, i: usize) -> Result<Self, CommandError> {
        if args.len() <= i {
            return Ok(Self::Replace);
        }
        match args.str(i)? {
            "replace" => Ok(Self::Replace),
            "add" => Ok(Self::Add),
            "toggle" => Ok(Self::Toggle),
            other => Err(args.invalid(format!("`{other}` is not replace, add or toggle"))),
        }
    }

    fn apply(self, current: Vec<u32>, ids: &[u32]) -> BTreeSet<u32> {
        let mut set: BTreeSet<u32> = current.into_iter().collect();
        match self {
            Self::Replace => return ids.iter().copied().collect(),
            Self::Add => set.extend(ids),
            Self::Remove => ids.iter().for_each(|id| {
                set.remove(id);
            }),
            Self::Toggle => ids.iter().for_each(|id| {
                if !set.remove(id) {
                    set.insert(*id);
                }
            }),
        }
        set
    }
}

fn select(project: &mut Project, set: BTreeSet<u32>) -> Result<Outcome, CommandError> {
    let count = set.len();
    project.set_selection(set)?;
    Ok(Outcome::done(format!("{count} selected")))
}

fn change(project: &mut Project, args: &Args<'_>, how: Combine) -> Result<Outcome, CommandError> {
    let ids = args.required_ids(0, project)?;
    let set = how.apply(project.selection(), &ids);
    select(project, set)
}

fn clear(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    select(project, BTreeSet::new())
}

fn all(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let set = project.order().into_iter().collect();
    select(project, set)
}

fn invert(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let set = project
        .order()
        .into_iter()
        .filter(|id| !project.is_selected(*id))
        .collect();
    select(project, set)
}

fn tolerance(args: &Args<'_>, i: usize) -> Result<f64, CommandError> {
    let t = args.f64(i, "tolerance")?;
    if t < 0.0 {
        return Err(args.invalid("tolerance must not be negative"));
    }
    Ok(t)
}

fn at(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 4)?;
    let p = args.point(0)?;
    let tol = tolerance(args, 2)?;
    let how = Combine::parse(args, 3)?;
    let hit: Vec<u32> = project.shape_at(p, tol).into_iter().collect();
    let set = how.apply(project.selection(), &hit);
    select(project, set)
}

fn in_rect(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(5, 6)?;
    let rect = Rect::from_corners(args.point(0)?, args.point(2)?);
    let mode = match args.str(4)? {
        "contained" => BoxMode::Contained,
        "touching" => BoxMode::Touching,
        other => return Err(args.invalid(format!("`{other}` is not contained or touching"))),
    };
    let how = Combine::parse(args, 5)?;
    let ids = project.shapes_in_rect(&rect, mode, 0.01);
    let set = how.apply(project.selection(), &ids);
    select(project, set)
}

fn cycle(project: &mut Project, args: &Args<'_>, forward: bool) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let order = project.order();
    let Some(last) = order.len().checked_sub(1) else {
        return select(project, BTreeSet::new());
    };
    let current = match project.selection().as_slice() {
        [one] => project.index_of(*one),
        _ => None,
    };
    let next = match (current, forward) {
        (None, true) => 0,
        (None, false) => last,
        (Some(i), true) => (i + 1) % order.len(),
        (Some(i), false) => i.checked_sub(1).unwrap_or(last),
    };
    select(project, BTreeSet::from([order[next]]))
}

#[cfg(test)]
mod tests {
    use crate::{Registry, Session};

    use super::*;

    fn session_with_three() -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for x in [0, 20, 40] {
            s.run_line(&format!("shape.add_rect {x} 0 10 10")).unwrap();
        }
        s
    }

    fn selected(s: &Session) -> Vec<u32> {
        s.project().selection()
    }

    #[test]
    fn set_add_remove_toggle_clear_all_invert() {
        let mut s = session_with_three();
        s.run_line("select.set 1").unwrap();
        s.run_line("select.add 3").unwrap();
        assert_eq!(selected(&s), [1, 3]);
        s.run_line("select.toggle 3 2").unwrap();
        assert_eq!(selected(&s), [1, 2]);
        s.run_line("select.remove 1").unwrap();
        assert_eq!(selected(&s), [2]);
        s.run_line("select.invert").unwrap();
        assert_eq!(selected(&s), [1, 3]);
        s.run_line("select.all").unwrap();
        assert_eq!(selected(&s), [1, 2, 3]);
        s.run_line("select.clear").unwrap();
        assert!(selected(&s).is_empty());
        assert!(s.run_line("select.set 9").is_err());
        assert!(s.run_line("select.set").is_err());
        // Selection is never undo history.
        assert_eq!(s.undo_depth(), 3);
    }

    #[test]
    fn point_selection_modes() {
        let mut s = session_with_three();
        s.run_line("select.at 25 5 0.1").unwrap();
        assert_eq!(selected(&s), [2]);
        s.run_line("select.at 5 5 0.1 add").unwrap();
        assert_eq!(selected(&s), [1, 2]);
        s.run_line("select.at 5 5 0.1 toggle").unwrap();
        assert_eq!(selected(&s), [2]);
        s.run_line("select.at 15 5 0.1 add").unwrap();
        assert_eq!(selected(&s), [2], "a miss with add keeps the selection");
        s.run_line("select.at 15 5 0.1").unwrap();
        assert!(selected(&s).is_empty(), "a miss with replace clears");
        assert!(s.run_line("select.at 1 1 -1").is_err());
        assert!(s.run_line("select.at 1 1 1 sideways").is_err());
    }

    #[test]
    fn box_selection_left_and_right() {
        let mut s = session_with_three();
        s.run_line("select.in_rect -1 -1 35 11 contained").unwrap();
        assert_eq!(selected(&s), [1, 2]);
        s.run_line("select.in_rect 35 -1 -1 11 touching").unwrap();
        assert_eq!(selected(&s), [1, 2]);
        s.run_line("select.in_rect 15 -1 45 5 touching").unwrap();
        assert_eq!(selected(&s), [2, 3]);
        s.run_line("select.in_rect 0 0 1 1 contained add").unwrap();
        assert_eq!(selected(&s), [2, 3]);
        assert!(s.run_line("select.in_rect 0 0 1 1 inside").is_err());
    }

    #[test]
    fn cycling_wraps_round() {
        let mut s = session_with_three();
        s.run_line("select.clear").unwrap();
        s.run_line("select.next").unwrap();
        assert_eq!(selected(&s), [1]);
        s.run_line("select.previous").unwrap();
        assert_eq!(selected(&s), [3]);
        s.run_line("select.next").unwrap();
        assert_eq!(selected(&s), [1]);
        let mut empty = Session::new(Project::default(), Registry::with_builtin());
        empty.run_line("select.next").unwrap();
    }
}
