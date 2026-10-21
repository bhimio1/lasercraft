//! `group.*`: group and ungroup objects (`docs/spec/arrange.md`).

use lasercraft_project::Project;

use super::combine;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "group.create",
            "group.create [ids]",
            "Group the selected objects into one.",
            create,
        ),
        cmd(
            "group.ungroup",
            "group.ungroup [ids]",
            "Split grouped objects one level.",
            ungroup,
        ),
        cmd(
            "group.set",
            "group.set <id> [group ...]",
            "Set a shape's groups exactly, outermost first. Used by undo.",
            set,
        ),
    ]
}

fn set_invocation(id: u32, groups: &[u32]) -> Invocation {
    Invocation::new(
        "group.set",
        std::iter::once(id.to_string()).chain(groups.iter().map(ToString::to_string)),
    )
}

/// Sets new group chains, returning the inverse.
fn apply(project: &mut Project, chains: Vec<(u32, Vec<u32>)>) -> Result<Invocation, CommandError> {
    let mut inverses = Vec::with_capacity(chains.len());
    for (id, groups) in chains {
        let old = project
            .shape(id)
            .map(|s| s.groups().to_vec())
            .unwrap_or_default();
        project.set_shape_groups(id, groups)?;
        inverses.push(set_invocation(id, &old));
    }
    Ok(combine(inverses))
}

fn create(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let objects = project.objects_of(&ids)?;
    if objects.len() < 2 {
        return Err(args.invalid("select at least two objects to group"));
    }
    let group = project.allocate_id();
    let mut chains = Vec::new();
    for id in objects.iter().flatten() {
        let mut chain = vec![group];
        chain.extend(
            project
                .shape(*id)
                .map(|s| s.groups().to_vec())
                .unwrap_or_default(),
        );
        chains.push((*id, chain));
    }
    let members: Vec<u32> = chains.iter().map(|(id, _)| *id).collect();
    let inverse = apply(project, chains)?;
    project.set_selection(members)?;
    Ok(Outcome::undoable(
        format!("grouped {} objects as group {group}", objects.len()),
        inverse,
    ))
}

fn ungroup(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let ids = args.ids(0, project)?;
    let objects = project.objects_of(&ids)?;
    let mut chains = Vec::new();
    let mut count = 0;
    for object in &objects {
        let mut grouped = false;
        for id in object {
            if let Some(shape) = project.shape(*id)
                && let Some((_, rest)) = shape.groups().split_first()
            {
                chains.push((*id, rest.to_vec()));
                grouped = true;
            }
        }
        count += usize::from(grouped);
    }
    if chains.is_empty() {
        return Err(args.invalid("nothing to ungroup"));
    }
    let inverse = apply(project, chains)?;
    Ok(Outcome::undoable(
        format!("ungrouped {count} group(s)"),
        inverse,
    ))
}

fn set(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, usize::MAX)?;
    let id: u32 = args.parse(0, "shape id")?;
    if project.shape(id).is_none() {
        return Err(args.invalid(format!("shape {id} does not exist")));
    }
    let groups = (1..args.len())
        .map(|i| args.parse::<u32>(i, "group id"))
        .collect::<Result<Vec<_>, _>>()?;
    let inverse = apply(project, vec![(id, groups)])?;
    Ok(Outcome::undoable(
        format!("set groups of shape {id}"),
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
            s.run_line(line).unwrap();
        }
        s
    }

    #[test]
    fn grouping_selects_whole_groups_and_undoes() {
        let mut s = session(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 20 0 10 10",
            "shape.add_rect 40 0 10 10",
        ]);
        let before = s.project().clone();
        s.run_line("group.create 1 2").unwrap();
        assert_eq!(s.project().shape(1).unwrap().groups(), [4]);
        s.run_line("select.set 2").unwrap();
        assert_eq!(s.project().selection(), [1, 2]);
        // Nesting: the group and shape 3 into an outer group.
        s.run_line("group.create 1 3").unwrap();
        assert_eq!(s.project().shape(2).unwrap().groups(), [5, 4]);
        assert_eq!(s.project().shape(3).unwrap().groups(), [5]);
        s.run_line("group.ungroup 3").unwrap();
        assert_eq!(s.project().shape(1).unwrap().groups(), [4]);
        assert!(s.project().shape(3).unwrap().groups().is_empty());
        for _ in 0..3 {
            s.run_line("edit.undo").unwrap();
        }
        assert_eq!(s.project(), &before);
    }

    #[test]
    fn grouping_errors_leave_the_project_alone() {
        let mut s = session(&["shape.add_rect 0 0 10 10", "shape.add_rect 20 0 10 10"]);
        let before = s.project().clone();
        assert!(s.run_line("group.create 1").is_err());
        assert!(s.run_line("group.ungroup 1 2").is_err());
        assert!(s.run_line("group.create 1 9").is_err());
        assert!(s.run_line("group.set 9 1").is_err());
        assert!(s.run_line("group.set 1 0").is_err());
        assert!(s.run_line("group.set").is_err());
        assert_eq!(s.project(), &before);
    }

    #[test]
    fn copies_get_their_own_groups() {
        let mut s = session(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 20 0 10 10",
            "group.create 1 2",
        ]);
        s.run_line("shape.duplicate 1 2").unwrap();
        assert_eq!(s.project().shape(4).unwrap().groups(), [6]);
        assert_eq!(s.project().shape(5).unwrap().groups(), [6]);
        s.run_line("select.set 4").unwrap();
        assert_eq!(s.project().selection(), [4, 5]);
    }
}
