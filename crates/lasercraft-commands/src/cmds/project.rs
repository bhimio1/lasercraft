//! `project.*`: create, load, save and describe the document.

use std::path::Path;

use lasercraft_project::{Origin, Project};

use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "project.new",
            "project.new [<width_mm> <height_mm>]",
            "Start an empty project, optionally with a work area size. Clears undo history.",
            new,
        ),
        cmd(
            "project.load",
            "project.load <path>",
            "Open a .lcproj file. Clears undo history.",
            load,
        ),
        cmd(
            "project.save",
            "project.save <path>",
            "Write the project to a .lcproj file.",
            save,
        ),
        cmd(
            "project.rename",
            "project.rename <name>",
            "Rename the project.",
            rename,
        ),
        cmd(
            "project.set_notes",
            "project.set_notes <text>",
            "Replace the project notes (quote text with spaces).",
            set_notes,
        ),
        cmd(
            "project.set_work_area",
            "project.set_work_area <width_mm> <height_mm>",
            "Set the size of the machine's work area.",
            set_work_area,
        ),
        cmd(
            "project.set_origin",
            "project.set_origin <front-left|front-right|rear-left|rear-right>",
            "Set the bed corner that is the machine origin.",
            set_origin,
        ),
    ]
}

fn new(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let fresh = match args.len() {
        0 => Project::default(),
        2 => Project::with_work_area(args.f64(0, "width")?, args.f64(1, "height")?)?,
        _ => return Err(args.usage_error()),
    };
    *project = fresh;
    Ok(Outcome::reset("new project"))
}

fn load(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let path = args.str(0)?;
    *project = Project::load(Path::new(path))?;
    Ok(Outcome::reset(format!("loaded {path}")))
}

fn save(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let path = args.str(0)?;
    project.save(Path::new(path))?;
    Ok(Outcome::done(format!("saved {path}")))
}

fn rename(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let name = args.rest(0)?;
    let old = project.name().to_owned();
    project.set_name(&name)?;
    Ok(Outcome::undoable(
        format!("renamed to {}", project.name()),
        Invocation::new("project.rename", [old]),
    ))
}

fn set_notes(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let old = project.notes().to_owned();
    project.set_notes(args.str(0)?)?;
    Ok(Outcome::undoable(
        "notes updated",
        Invocation::new("project.set_notes", [old]),
    ))
}

fn set_work_area(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let (width, height) = (args.f64(0, "width")?, args.f64(1, "height")?);
    let old = *project.work_area();
    project.set_work_area_size(width, height)?;
    Ok(Outcome::undoable(
        format!("work area {width} × {height} mm"),
        Invocation::new(
            "project.set_work_area",
            [old.width_mm.to_string(), old.height_mm.to_string()],
        ),
    ))
}

fn set_origin(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let origin: Origin = args.str(0)?.parse()?;
    let old = project.work_area().origin;
    project.set_origin(origin);
    Ok(Outcome::undoable(
        format!("origin {origin}"),
        Invocation::new("project.set_origin", [old.as_str()]),
    ))
}

#[cfg(test)]
mod tests {
    use lasercraft_project::ProjectError;

    use crate::{Registry, Session};

    use super::*;

    fn session() -> Session {
        Session::new(Project::default(), Registry::with_builtin())
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lasercraft-cmd-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn new_resets_project_and_accepts_a_size() {
        let mut s = session();
        s.run_line("project.rename Old").unwrap();
        s.run_line("project.new 600 300").unwrap();
        assert_eq!(s.project().name(), "Untitled");
        assert_eq!(s.project().work_area().width_mm, 600.0);
        assert_eq!(s.undo_depth(), 0);
        assert!(s.run_line("project.new 600").is_err());
        assert!(s.run_line("project.new 0 300").is_err());
        assert_eq!(s.project().work_area().width_mm, 600.0);
    }

    #[test]
    fn rename_joins_words_and_undoes() {
        let mut s = session();
        s.run_line("project.rename Birch coasters").unwrap();
        assert_eq!(s.project().name(), "Birch coasters");
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().name(), "Untitled");
        assert!(matches!(
            s.run_line("project.rename"),
            Err(CommandError::Usage { .. })
        ));
        assert!(matches!(
            s.run_line(r#"project.rename "  ""#),
            Err(CommandError::Project(ProjectError::InvalidValue { .. }))
        ));
    }

    #[test]
    fn notes_keep_newlines_and_undo() {
        let mut s = session();
        s.run_line(r#"project.set_notes "3 mm birch""#).unwrap();
        s.run(&Invocation::new("project.set_notes", ["a\nb"]))
            .unwrap();
        assert_eq!(s.project().notes(), "a\nb");
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().notes(), "3 mm birch");
        assert!(s.run_line("project.set_notes a b").is_err());
    }

    #[test]
    fn work_area_is_validated_and_undoes_exactly() {
        let mut s = session();
        s.run_line("project.set_work_area 600.25 300.1").unwrap();
        assert_eq!(s.project().work_area().height_mm, 300.1);
        s.run_line("project.set_work_area 100 100").unwrap();
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().work_area().width_mm, 600.25);
        assert_eq!(s.project().work_area().height_mm, 300.1);
        for bad in ["0 100", "100 nan", "100 inf", "abc 100", "100"] {
            assert!(
                s.run_line(&format!("project.set_work_area {bad}")).is_err(),
                "{bad}"
            );
        }
        assert_eq!(s.project().work_area().width_mm, 600.25);
    }

    #[test]
    fn origin_parses_and_undoes() {
        let mut s = session();
        s.run_line("project.set_origin rear-right").unwrap();
        assert_eq!(s.project().work_area().origin, Origin::RearRight);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().work_area().origin, Origin::FrontLeft);
        assert!(s.run_line("project.set_origin centre").is_err());
    }

    #[test]
    fn save_then_load_restores_project_and_clears_history() {
        let dir = temp_dir("save");
        let path = dir.join("job with spaces.lcproj");
        let mut s = session();
        s.run_line("project.rename Saved job").unwrap();
        s.run(&Invocation::new(
            "project.save",
            [path.display().to_string()],
        ))
        .unwrap();
        // Saving is not an edit.
        assert_eq!(s.undo_depth(), 1);
        s.run_line("project.new").unwrap();
        s.run(&Invocation::new(
            "project.load",
            [path.display().to_string()],
        ))
        .unwrap();
        assert_eq!(s.project().name(), "Saved job");
        assert_eq!(s.undo_depth(), 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn failed_load_leaves_project_alone() {
        let dir = temp_dir("badload");
        let path = dir.join("bad.lcproj");
        std::fs::write(&path, "{\"format\": \"something-else\"}").unwrap();
        let mut s = session();
        s.run_line("project.rename Keep").unwrap();
        assert!(
            s.run(&Invocation::new(
                "project.load",
                [path.display().to_string()]
            ))
            .is_err()
        );
        assert!(s.run_line("project.load /nonexistent/x.lcproj").is_err());
        assert_eq!(s.project().name(), "Keep");
        assert_eq!(s.undo_depth(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
