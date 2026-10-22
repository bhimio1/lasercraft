//! `profile.*` and `project.set_device`: device profiles
//! (`docs/spec/devices.md`). Profile changes are user data: not undoable,
//! saved at once. `project.set_device` changes the project and is undoable.

use std::path::Path;

use lasercraft_project::Project;
use lasercraft_workshop::{DeviceProfile, WorkshopError};

use super::export::key_value;
use crate::args::Args;
use crate::command::ccmd;
use crate::{BATCH, Command, CommandError, Context, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        ccmd(
            "profile.add",
            "profile.add <name> [key=value ...]",
            "Add a device profile; keys as profile.set.",
            add,
        ),
        ccmd(
            "profile.set",
            "profile.set <name> <width|height|origin|port|baud|max_speed|acceleration|max_power|s_max|air> <value>",
            "Change one field of a device profile.",
            set,
        ),
        ccmd(
            "profile.rename",
            "profile.rename <old> <new>",
            "Rename a device profile.",
            rename,
        ),
        ccmd(
            "profile.duplicate",
            "profile.duplicate <name> <new>",
            "Copy a device profile under a new name.",
            duplicate,
        ),
        ccmd(
            "profile.delete",
            "profile.delete <name>",
            "Remove a device profile.",
            delete,
        ),
        ccmd(
            "profile.default",
            "profile.default <name|none>",
            "Set or clear the default device profile.",
            default,
        ),
        ccmd(
            "profile.import",
            "profile.import <path>",
            "Add the device profile in a .lcdevice file.",
            import,
        ),
        ccmd(
            "profile.export",
            "profile.export <name> <path>",
            "Write a device profile to a .lcdevice file.",
            export,
        ),
        ccmd(
            "profile.list",
            "profile.list",
            "List the device profiles.",
            list,
        ),
        ccmd(
            "project.set_device",
            "project.set_device <name|none> [keep_area]",
            "Make the project use a device profile and its work area.",
            set_device,
        ),
    ]
}

/// A workshop error as this command's error.
pub(super) fn ws<'a>(args: &'a Args<'_>) -> impl Fn(WorkshopError) -> CommandError + 'a {
    move |e| args.invalid(e.0)
}

/// The outcome of a workshop change, noting a failed save.
pub(super) fn saved(cx: &mut Context<'_>, message: String) -> Outcome {
    match cx.workshop.take_warning() {
        Some(w) => Outcome::done(format!("{message} (not saved: {w})")),
        None => Outcome::done(message),
    }
}

fn add(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() == 0 {
        return Err(args.usage_error());
    }
    let mut profile = DeviceProfile::new(args.str(0)?).map_err(ws(args))?;
    for i in 1..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        profile.set(key, value).map_err(ws(args))?;
    }
    let name = profile.name.clone();
    cx.workshop.add_profile(profile).map_err(ws(args))?;
    Ok(saved(cx, format!("added device profile \"{name}\"")))
}

fn set(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let (name, key, value) = (args.str(0)?, args.str(1)?, args.str(2)?);
    cx.workshop
        .set_profile(name, key, value)
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("device profile \"{name}\": {key} = {value}"),
    ))
}

fn rename(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    cx.workshop
        .rename_profile(args.str(0)?, args.str(1)?)
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("renamed device profile to \"{}\"", args.str(1)?.trim()),
    ))
}

fn duplicate(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    cx.workshop
        .duplicate_profile(args.str(0)?, args.str(1)?)
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("added device profile \"{}\"", args.str(1)?.trim()),
    ))
}

fn delete(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let removed = cx.workshop.delete_profile(args.str(0)?).map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("removed device profile \"{}\"", removed.name),
    ))
}

fn default(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let name = args.str(0)?;
    let name = (name != "none").then_some(name);
    cx.workshop.set_default(name).map_err(ws(args))?;
    let message = match cx.workshop.default_profile() {
        Some(n) => format!("default device profile \"{n}\""),
        None => "no default device profile".to_owned(),
    };
    Ok(saved(cx, message))
}

fn import(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let name = cx
        .workshop
        .import_profile(Path::new(args.str(0)?))
        .map_err(ws(args))?;
    Ok(saved(cx, format!("imported device profile \"{name}\"")))
}

fn export(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let path = args.str(1)?;
    cx.workshop
        .export_profile(args.str(0)?, Path::new(path))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!("wrote {path}")))
}

fn list(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let ws = &cx.workshop;
    if ws.profiles().is_empty() {
        return Ok(Outcome::done("no device profiles"));
    }
    let lines: Vec<String> = ws
        .profiles()
        .iter()
        .map(|p| {
            let mark = if ws.default_profile() == Some(p.name.as_str()) {
                " (default)"
            } else {
                ""
            };
            format!("{}{mark}: {}", p.name, p.summary())
        })
        .collect();
    Ok(Outcome::done(lines.join("\n")))
}

/// The invocation that puts the project's device and work area back.
fn restore(project: &Project) -> Invocation {
    let area = project.work_area();
    let device = project.device().map_or_else(
        || Invocation::new("project.set_device", ["none"]),
        |d| Invocation::new("project.set_device", [d, "keep_area"]),
    );
    Invocation::new(
        BATCH,
        [
            device.to_string(),
            Invocation::new(
                "project.set_work_area",
                [area.width_mm.to_string(), area.height_mm.to_string()],
            )
            .to_string(),
            Invocation::new("project.set_origin", [area.origin.as_str()]).to_string(),
        ],
    )
}

fn set_device(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 2)?;
    let name = args.str(0)?;
    let keep_area = match args.len() {
        1 => false,
        _ if args.str(1)? == "keep_area" => true,
        _ => return Err(args.usage_error()),
    };
    let inverse = restore(cx.project);
    if name == "none" {
        cx.project.set_device(None)?;
        return Ok(Outcome::undoable(
            "the project has no device profile",
            inverse,
        ));
    }
    if keep_area {
        cx.project.set_device(Some(name))?;
        return Ok(Outcome::undoable(
            format!("device profile \"{}\"", name.trim()),
            inverse,
        ));
    }
    let profile = cx
        .workshop
        .profile(name)
        .ok_or_else(|| args.invalid(format!("no device profile named \"{}\"", name.trim())))?;
    let area = profile.work_area();
    let profile_name = profile.name.clone();
    let mut next = cx.project.clone();
    next.set_device(Some(&profile_name))?;
    next.set_work_area_size(area.width_mm, area.height_mm)?;
    next.set_origin(area.origin);
    *cx.project = next;
    let mut message = format!(
        "device profile \"{profile_name}\": work area {} × {} mm, origin {}",
        area.width_mm,
        area.height_mm,
        area.origin.as_str().replace('-', " ")
    );
    if let Some(library) = cx.workshop.use_library_for(&profile_name) {
        message.push_str(&format!("; library \"{library}\" is active"));
    }
    Ok(Outcome::undoable(message, inverse))
}

#[cfg(test)]
mod tests {
    use lasercraft_project::Origin;

    use crate::{Registry, Session};

    fn session() -> Session {
        Session::new(
            lasercraft_project::Project::default(),
            Registry::with_builtin(),
        )
    }

    #[test]
    fn profiles_are_added_changed_and_listed() {
        let mut s = session();
        s.run_line("profile.add Diode width=300 height=200 origin=rear-left max_power=60")
            .unwrap();
        s.run_line("profile.add \"CO2 tube\"").unwrap();
        s.run_line("profile.set co2\\ tube width 600").unwrap_err();
        s.run_line("profile.set \"co2 tube\" width 600").unwrap();
        let list = s.run_line("profile.list").unwrap().message;
        assert!(
            list.contains("Diode (default): 300 × 200 mm, origin rear left"),
            "{list}"
        );
        assert!(list.contains("CO2 tube: 600 × 400 mm"), "{list}");
        assert!(s.run_line("profile.add diode").is_err(), "names are unique");
        assert!(s.run_line("profile.add Bad max_power=300").is_err());
        s.run_line("profile.rename Diode \"Shop diode\"").unwrap();
        s.run_line("profile.default \"CO2 tube\"").unwrap();
        assert_eq!(s.workshop().default_profile(), Some("CO2 tube"));
        s.run_line("profile.duplicate \"CO2 tube\" Spare").unwrap();
        s.run_line("profile.delete Spare").unwrap();
        assert_eq!(s.workshop().profiles().len(), 2);
        // Not part of undo history.
        assert!(s.run_line("edit.undo").is_err());
    }

    #[test]
    fn set_device_copies_the_work_area_and_undoes_exactly() {
        let mut s = session();
        s.run_line("profile.add Diode width=300 height=200 origin=rear-left")
            .unwrap();
        let before = s.project().clone();
        s.run_line("project.set_device diode").unwrap();
        let area = *s.project().work_area();
        assert_eq!(
            (
                s.project().device(),
                area.width_mm,
                area.height_mm,
                area.origin
            ),
            (Some("Diode"), 300.0, 200.0, Origin::RearLeft)
        );
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        s.run_line("edit.redo").unwrap();
        s.run_line("project.set_device none").unwrap();
        assert_eq!(s.project().device(), None);
        assert_eq!(
            s.project().work_area().width_mm,
            300.0,
            "none keeps the area"
        );
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().device(), Some("Diode"));
        assert!(s.run_line("project.set_device Nope").is_err());
    }
}
