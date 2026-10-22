//! `library.*` and `material.*`: material libraries
//! (`docs/spec/materials.md`). Library changes are user data and not
//! undoable; the project part of a command (layer settings, follow marks)
//! is undoable and restored exactly by its inverse.

use std::path::Path;

use lasercraft_project::{Color, Layer, Project};
use lasercraft_workshop::{Key, Material, Workshop};

use super::combine;
use super::profile::{saved, ws};
use crate::args::Args;
use crate::command::ccmd;
use crate::{Command, CommandError, Context, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        ccmd(
            "library.new",
            "library.new <name>",
            "Open a new, empty material library.",
            new,
        ),
        ccmd(
            "library.open",
            "library.open <path>",
            "Open a .lcmat material library.",
            open,
        ),
        ccmd(
            "library.save",
            "library.save [path]",
            "Write the active material library to its file or to a path.",
            save,
        ),
        ccmd(
            "library.close",
            "library.close [name] [force]",
            "Close a material library; `force` drops unsaved changes.",
            close,
        ),
        ccmd(
            "library.use",
            "library.use <name>",
            "Make an open library active.",
            use_library,
        ),
        ccmd(
            "library.rename",
            "library.rename <name>",
            "Rename the active material library.",
            rename,
        ),
        ccmd(
            "library.set_device",
            "library.set_device <name|none>",
            "Tie the active library to a device profile.",
            set_device,
        ),
        ccmd(
            "library.merge",
            "library.merge <path>",
            "Add the entries of another library file to the active one.",
            merge,
        ),
        ccmd(
            "material.add",
            "material.add <material> <thickness|-> <description> <color> [group=<name>]",
            "Store a layer's settings as a new material entry.",
            add,
        ),
        ccmd(
            "material.apply",
            "material.apply <id> <color>",
            "Copy a material entry's settings into a layer.",
            apply,
        ),
        ccmd(
            "material.follow",
            "material.follow <id> <color>",
            "Tie a layer to a material entry.",
            follow,
        ),
        ccmd(
            "material.unfollow",
            "material.unfollow <color>",
            "Stop a layer following its material entry.",
            unfollow,
        ),
        ccmd(
            "material.update",
            "material.update <id> <color>",
            "Replace an entry's settings with a layer's; followers change too.",
            update,
        ),
        ccmd(
            "material.set",
            "material.set <id> <key> <value>",
            "Change one setting of a material entry; followers change too.",
            set,
        ),
        ccmd(
            "material.describe",
            "material.describe <id> <material> <thickness|-> <description> [group=<name>]",
            "Change an entry's material, thickness or group and description.",
            describe,
        ),
        ccmd(
            "material.duplicate",
            "material.duplicate <id>",
            "Copy a material entry.",
            duplicate,
        ),
        ccmd(
            "material.delete",
            "material.delete <id>",
            "Remove a material entry; followers stop following.",
            delete,
        ),
        ccmd(
            "material.list",
            "material.list",
            "Show the active library's entries with their ids.",
            list,
        ),
    ]
}

fn new(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let name = args.rest(0)?;
    cx.workshop.new_library(&name).map_err(ws(args))?;
    Ok(Outcome::done(format!(
        "new material library \"{}\"",
        name.trim()
    )))
}

fn open(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    cx.workshop
        .open_library(Path::new(args.str(0)?))
        .map_err(ws(args))?;
    let name = cx.workshop.active().library.name.clone();
    Ok(saved(cx, format!("material library \"{name}\" is active")))
}

fn save(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 1)?;
    let path = if args.len() == 1 {
        Some(Path::new(args.str(0)?))
    } else {
        None
    };
    let written = cx.workshop.save_library(path).map_err(ws(args))?;
    Ok(saved(cx, format!("saved {}", written.display())))
}

fn close(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 2)?;
    let mut name = None;
    let mut force = false;
    for i in 0..args.len() {
        match args.str(i)? {
            "force" if i + 1 == args.len() => force = true,
            n if i == 0 => name = Some(n),
            _ => return Err(args.usage_error()),
        }
    }
    let closed = cx.workshop.close_library(name, force).map_err(ws(args))?;
    Ok(saved(cx, format!("closed material library \"{closed}\"")))
}

fn use_library(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let name = args.rest(0)?;
    cx.workshop.use_library(&name).map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("material library \"{}\" is active", name.trim()),
    ))
}

fn rename(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let name = args.rest(0)?;
    cx.workshop.rename_library(&name).map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("renamed the library to \"{}\"", name.trim()),
    ))
}

fn set_device(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let name = args.rest(0)?;
    let device = (name.trim() != "none").then(|| name.trim().to_owned());
    let message = device.as_ref().map_or_else(
        || "the library is not tied to a device".to_owned(),
        |d| format!("the library is for device profile \"{d}\""),
    );
    cx.workshop
        .edit(|lib| {
            lib.device = device;
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(Outcome::done(message))
}

fn merge(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let other = Workshop::read_library(Path::new(args.str(0)?)).map_err(ws(args))?;
    let (added, skipped) = cx
        .workshop
        .edit(|lib| Ok(lib.merge(&other)))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!(
        "merged \"{}\": {added} entries added, {skipped} already present",
        other.name
    )))
}

/// Reads `<material> <thickness|-> <description>` at `at` and an optional
/// `group=` after them.
fn key_at(args: &Args<'_>, at: usize, last: usize) -> Result<Key, CommandError> {
    let thickness = match args.str(at + 1)? {
        "-" => None,
        _ => Some(args.f64(at + 1, "thickness")?),
    };
    let mut group = None;
    for i in last..args.len() {
        match args.str(i)?.split_once('=') {
            Some(("group", g)) => group = Some(g),
            _ => return Err(args.usage_error()),
        }
    }
    Key::new(args.str(at)?, thickness, group, args.str(at + 2)?).map_err(ws(args))
}

fn id(args: &Args<'_>) -> Result<u32, CommandError> {
    args.parse::<u32>(0, "entry id")
}

/// The active library's entry `id`, with the library's name.
fn entry(cx: &Context<'_>, args: &Args<'_>, id: u32) -> Result<(String, Material), CommandError> {
    let open = cx.workshop.active();
    let entry = open.library.entry(id).map_err(ws(args))?.clone();
    Ok((open.library.name.clone(), entry))
}

/// Replaces layers of `project`, returning the inverse for each.
fn replace_layers(
    project: &mut Project,
    layers: Vec<Layer>,
) -> Result<Vec<Invocation>, CommandError> {
    let mut inverses = Vec::new();
    for layer in layers {
        let old = project.layer(layer.color)?.clone();
        if old == layer {
            continue;
        }
        project.replace_layer(layer.color, layer)?;
        let json = serde_json::to_string(&old).unwrap_or_default();
        inverses.push(Invocation::new(
            "layer.set_all",
            [old.color.to_string(), json],
        ));
    }
    inverses.reverse();
    Ok(inverses)
}

/// The outcome of a command that may have changed layers.
fn outcome(message: String, inverses: Vec<Invocation>) -> Outcome {
    if inverses.is_empty() {
        Outcome::done(message)
    } else {
        Outcome::undoable(message, combine(inverses))
    }
}

/// The layers of `project` that follow `old` (an entry of `library`), set
/// from `entry` (the entry as it is now; `None` when it was deleted).
fn followers(
    project: &Project,
    library: &str,
    old: &Material,
    entry: Option<&Material>,
) -> Vec<Layer> {
    project
        .layers()
        .iter()
        .filter(|l| {
            l.follows
                .as_ref()
                .is_some_and(|f| old.is_followed_by(library, f))
        })
        .map(|l| match entry {
            Some(e) => {
                let mut next = e.applied_to(l);
                next.follows = Some(e.follows(library));
                next
            }
            None => Layer {
                follows: None,
                ..l.clone()
            },
        })
        .collect()
}

/// Changes entry `id` of the active library with `change`, then updates
/// the project's layers that follow it.
fn change_entry(
    cx: &mut Context<'_>,
    args: &Args<'_>,
    id: u32,
    change: impl FnOnce(
        &mut lasercraft_workshop::Library,
    ) -> Result<(), lasercraft_workshop::WorkshopError>,
) -> Result<Vec<Invocation>, CommandError> {
    let (library, old) = entry(cx, args, id)?;
    let mut next = cx.workshop.active().library.clone();
    change(&mut next).map_err(ws(args))?;
    let new = next.entry(id).ok().cloned();
    let mut project = cx.project.clone();
    let changed = followers(&project, &library, &old, new.as_ref());
    let inverses = replace_layers(&mut project, changed)?;
    cx.workshop
        .edit(|lib| {
            *lib = next;
            Ok(())
        })
        .map_err(ws(args))?;
    *cx.project = project;
    Ok(inverses)
}

fn add(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 5)?;
    let key = key_at(args, 0, 4)?;
    let color = args.color_at(3)?;
    let layer = cx.project.layer(color)?.clone();
    let description = key.description.clone();
    let new_id = cx
        .workshop
        .edit(|lib| lib.add(key, &layer))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!(
        "added material entry {new_id} \"{description}\" from layer {color:02}"
    )))
}

fn layer_from(
    cx: &Context<'_>,
    args: &Args<'_>,
    at: usize,
) -> Result<(Color, Layer), CommandError> {
    let color = args.color_at(at)?;
    Ok((color, cx.project.layer(color)?.clone()))
}

fn apply(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let (_, entry) = entry(cx, args, id(args)?)?;
    let (color, layer) = layer_from(cx, args, 1)?;
    let mut next = entry.applied_to(&layer);
    next.follows = None;
    let inverses = replace_layers(cx.project, vec![next])?;
    Ok(outcome(
        format!(
            "layer {color:02} has the settings of \"{}\"",
            entry.description
        ),
        inverses,
    ))
}

fn follow(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let (library, entry) = entry(cx, args, id(args)?)?;
    let (color, layer) = layer_from(cx, args, 1)?;
    let mut next = entry.applied_to(&layer);
    next.follows = Some(entry.follows(&library));
    let inverses = replace_layers(cx.project, vec![next])?;
    Ok(outcome(
        format!(
            "layer {color:02} follows \"{}\" ({library})",
            entry.description
        ),
        inverses,
    ))
}

fn unfollow(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let (color, layer) = layer_from(cx, args, 0)?;
    if layer.follows.is_none() {
        return Err(args.invalid(format!("layer {color:02} does not follow a material")));
    }
    let next = Layer {
        follows: None,
        ..layer
    };
    let inverses = replace_layers(cx.project, vec![next])?;
    Ok(outcome(
        format!("layer {color:02} no longer follows a material"),
        inverses,
    ))
}

fn update(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let id = id(args)?;
    let (color, layer) = layer_from(cx, args, 1)?;
    let inverses = change_entry(cx, args, id, |lib| lib.set_settings(id, &layer))?;
    Ok(outcome(
        format!("material entry {id} now has layer {color:02}'s settings"),
        inverses,
    ))
}

fn set(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let id = id(args)?;
    let (key, value) = (args.str(1)?, args.str(2)?);
    if matches!(key, "name" | "output" | "show") {
        return Err(args.invalid(format!("`{key}` is not stored in material entries")));
    }
    let (_, entry) = entry(cx, args, id)?;
    let mut settings = entry.settings.clone();
    settings.set(key, value)?;
    let inverses = change_entry(cx, args, id, |lib| lib.set_settings(id, &settings))?;
    Ok(outcome(
        format!("material entry {id}: {key} = {value}"),
        inverses,
    ))
}

fn describe(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 5)?;
    let id = id(args)?;
    let key = key_at(args, 1, 4)?;
    let description = key.description.clone();
    let inverses = change_entry(cx, args, id, |lib| lib.describe(id, key))?;
    Ok(outcome(
        format!("material entry {id} is now \"{description}\""),
        inverses,
    ))
}

fn duplicate(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let id = id(args)?;
    let new_id = cx
        .workshop
        .edit(|lib| lib.duplicate(id))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!(
        "copied material entry {id} as {new_id}"
    )))
}

fn delete(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let id = id(args)?;
    let inverses = change_entry(cx, args, id, |lib| lib.remove(id).map(|_| ()))?;
    Ok(outcome(format!("removed material entry {id}"), inverses))
}

fn list(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let open = cx.workshop.active();
    let mut text = format!("{}\n", open.library.name);
    if open.library.entries().is_empty() {
        text.push_str("no materials yet\n");
    } else {
        text.push_str(&open.library.tree());
    }
    Ok(Outcome::done(text.trim_end().to_owned()))
}

#[cfg(test)]
mod tests;
