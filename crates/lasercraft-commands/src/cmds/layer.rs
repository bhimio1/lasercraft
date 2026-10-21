//! `layer.*` and `shape.set_layer`: layer settings, order and membership
//! (`docs/spec/layers.md`).

use lasercraft_project::{Color, Layer, Project};

use super::combine;
use super::shape::remove;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "layer.set",
            "layer.set <color> <setting> <value>",
            "Change one layer setting (see docs/spec/layers.md).",
            set,
        ),
        cmd(
            "layer.set_all",
            "layer.set_all <color> <json>",
            "Replace all of a layer's settings.",
            set_all,
        ),
        cmd(
            "layer.copy_settings",
            "layer.copy_settings <from> <to>",
            "Copy every setting except the name from one layer to another.",
            copy_settings,
        ),
        cmd(
            "layer.move",
            "layer.move <color> <up|down|top|bottom>",
            "Move a layer in output order among the layers that have shapes.",
            move_layer,
        ),
        cmd(
            "layer.set_order",
            "layer.set_order <color> ...",
            "Set the output order of all 30 layers.",
            set_order,
        ),
        cmd(
            "layer.delete",
            "layer.delete <color>",
            "Delete every shape on a layer.",
            delete,
        ),
        cmd(
            "layer.select",
            "layer.select <color> [add]",
            "Select the shapes on a layer.",
            select,
        ),
        cmd(
            "shape.set_layer",
            "shape.set_layer <color> [ids]",
            "Move shapes to a layer.",
            set_layer,
        ),
    ]
}

fn set(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let color = args.color_at(0)?;
    let key = args.str(1)?;
    let old = project.layer(color)?.get(key)?;
    project.set_layer_setting(color, key, args.str(2)?)?;
    let new = project.layer(color)?.get(key)?;
    Ok(Outcome::undoable(
        format!("layer {color} {key} = {new}"),
        Invocation::new("layer.set", [color.to_string(), key.to_owned(), old]),
    ))
}

/// Replaces a layer's settings; the inverse restores the old ones exactly.
fn replace(
    project: &mut Project,
    color: Color,
    settings: Layer,
) -> Result<Invocation, CommandError> {
    let old = project.layer(color)?.clone();
    project.replace_layer(color, settings)?;
    let json = serde_json::to_string(&old).unwrap_or_default();
    Ok(Invocation::new("layer.set_all", [color.to_string(), json]))
}

fn set_all(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let color = args.color_at(0)?;
    let settings: Layer = serde_json::from_str(args.str(1)?)
        .map_err(|e| args.invalid(format!("not layer settings: {e}")))?;
    let inverse = replace(project, color, settings)?;
    Ok(Outcome::undoable(
        format!("layer {color} settings replaced"),
        inverse,
    ))
}

fn copy_settings(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let (from, to) = (args.color_at(0)?, args.color_at(1)?);
    let mut settings = project.layer(from)?.clone();
    settings.name.clone_from(&project.layer(to)?.name);
    let inverse = replace(project, to, settings)?;
    Ok(Outcome::undoable(
        format!("copied layer {from} settings to layer {to}"),
        inverse,
    ))
}

fn apply_order(project: &mut Project, order: &[Color]) -> Result<Outcome, CommandError> {
    let old = project.layer_order();
    if old == order {
        return Ok(Outcome::done("layer order unchanged"));
    }
    project.set_layer_order(order)?;
    Ok(Outcome::undoable(
        "layer order changed",
        Invocation::new("layer.set_order", old.iter().map(ToString::to_string)),
    ))
}

/// The full order after moving `color` among the used layers. Unused layers
/// keep their positions.
fn moved(order: &[Color], used: &[Color], color: Color, to: &str) -> Option<Vec<Color>> {
    let at = used.iter().position(|c| *c == color)?;
    let mut new_used = used.to_vec();
    let item = new_used.remove(at);
    let target = match to {
        "up" => at.saturating_sub(1),
        "down" => (at + 1).min(new_used.len()),
        "top" => 0,
        "bottom" => new_used.len(),
        _ => return None,
    };
    new_used.insert(target, item);
    let mut next = new_used.into_iter();
    Some(
        order
            .iter()
            .map(|c| {
                if used.contains(c) {
                    next.next().unwrap_or(*c)
                } else {
                    *c
                }
            })
            .collect(),
    )
}

fn move_layer(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let color = args.color_at(0)?;
    let to = args.str(1)?;
    if !matches!(to, "up" | "down" | "top" | "bottom") {
        return Err(args.invalid(format!("`{to}` is not up, down, top or bottom")));
    }
    let order = moved(&project.layer_order(), &project.used_layers(), color, to)
        .ok_or_else(|| args.invalid(format!("layer {color} has no shapes")))?;
    apply_order(project, &order)
}

fn set_order(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let order = (0..args.len())
        .map(|i| args.color_at(i))
        .collect::<Result<Vec<_>, _>>()?;
    apply_order(project, &order)
}

fn delete(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let color = args.color_at(0)?;
    let ids = project.shapes_on(color);
    if ids.is_empty() {
        return Ok(Outcome::done(format!("layer {color} has no shapes")));
    }
    let inverse = remove(project, &ids)?;
    Ok(Outcome::undoable(
        format!("deleted {} shape(s) on layer {color}", ids.len()),
        inverse,
    ))
}

fn select(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 2)?;
    let color = args.color_at(0)?;
    let mut ids = project.shapes_on(color);
    match args.len() {
        1 => {}
        _ if args.str(1)? == "add" => ids.extend(project.selection()),
        _ => return Err(args.usage_error()),
    }
    let count = ids.len();
    project.set_selection(ids)?;
    Ok(Outcome::done(format!("{count} selected")))
}

fn set_layer(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let color = args.color_at(0)?;
    let ids = args.ids(1, project)?;
    let mut inverses = Vec::new();
    for id in &ids {
        let old = project
            .shape(*id)
            .map_or(0, lasercraft_project::Shape::layer);
        if old != color {
            inverses.push(Invocation::new(
                "shape.set_layer",
                [old.to_string(), id.to_string()],
            ));
        }
    }
    if inverses.is_empty() {
        return Ok(Outcome::done(format!("already on layer {color}")));
    }
    for id in &ids {
        project.set_shape_layer(*id, color)?;
    }
    Ok(Outcome::undoable(
        format!("moved {} shape(s) to layer {color}", inverses.len()),
        combine(inverses),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{Registry, Session};

    use super::*;

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap();
        }
        s
    }

    #[test]
    fn set_changes_one_setting_and_undoes() {
        let mut s = session(&[]);
        s.run_line("layer.set 3 speed 150").unwrap();
        s.run_line("layer.set 3 interval 254lpi").unwrap();
        s.run_line(r#"layer.set 3 name "Deep cut""#).unwrap();
        let layer = s.project().layer(3).unwrap();
        assert_eq!(layer.speed, 150.0);
        assert!((layer.interval - 0.1).abs() < 1e-12);
        assert_eq!(layer.name, "Deep cut");
        for _ in 0..3 {
            s.run_line("edit.undo").unwrap();
        }
        assert_eq!(s.project(), &Project::default());
        for bad in [
            "layer.set 30 speed 1",
            "layer.set 3 speed 0",
            "layer.set 3 warp 1",
            "layer.set 3 speed",
            "layer.set 3 min_power 50",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn planning_settings_set_and_undo() {
        let mut s = session(&[]);
        s.run_line("layer.set 4 kerf 0.12").unwrap();
        s.run_line("layer.set 4 tabs 3").unwrap();
        s.run_line("layer.set 4 tab_width 0.8").unwrap();
        let layer = s.project().layer(4).unwrap();
        assert_eq!((layer.kerf, layer.tabs, layer.tab_width), (0.12, 3, 0.8));
        for bad in [
            "layer.set 4 kerf 9",
            "layer.set 4 tabs 2.5",
            "layer.set 4 tab_width 0",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        for _ in 0..3 {
            s.run_line("edit.undo").unwrap();
        }
        assert_eq!(s.project(), &Project::default());
    }

    #[test]
    fn copy_settings_keeps_the_name() {
        let mut s = session(&[
            "layer.set 1 mode fill",
            "layer.set 1 speed 300",
            "layer.set 1 name Src",
        ]);
        s.run_line("layer.copy_settings 1 2").unwrap();
        let to = s.project().layer(2).unwrap();
        assert_eq!(
            (to.mode, to.speed, to.name.as_str()),
            (lasercraft_project::Mode::Fill, 300.0, "Layer 02")
        );
        assert_eq!(to.color, 2);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().layer(2).unwrap(), &Layer::new(2));
    }

    #[test]
    fn set_all_round_trips_json() {
        let mut s = session(&[]);
        let mut settings = Layer::new(5);
        settings.passes = 4;
        let json = serde_json::to_string(&settings).unwrap();
        s.run(&Invocation::new("layer.set_all", ["5", &json]))
            .unwrap();
        assert_eq!(s.project().layer(5).unwrap().passes, 4);
        assert!(
            s.run(&Invocation::new("layer.set_all", ["5", "{}"]))
                .is_err()
        );
    }

    #[test]
    fn move_reorders_only_used_layers() {
        let order: Vec<Color> = (0..30).collect();
        let used = [2, 5, 9];
        let up = moved(&order, &used, 9, "up").unwrap();
        assert_eq!(&up[..10], &[0, 1, 2, 3, 4, 9, 6, 7, 8, 5]);
        let top = moved(&order, &used, 9, "top").unwrap();
        assert_eq!(&top[..10], &[0, 1, 9, 3, 4, 2, 6, 7, 8, 5]);
        assert_eq!(moved(&order, &used, 2, "up").unwrap(), order);
        assert!(moved(&order, &used, 3, "up").is_none());
    }

    #[test]
    fn move_and_order_commands_change_used_layers_and_undo() {
        let mut s = session(&["shape.add_rect 0 0 1 1 @2", "shape.add_rect 0 0 1 1 @5"]);
        assert_eq!(s.project().used_layers(), [2, 5]);
        s.run_line("layer.move 5 top").unwrap();
        assert_eq!(s.project().used_layers(), [5, 2]);
        let depth = s.undo_depth();
        s.run_line("layer.move 5 up").unwrap();
        assert_eq!(s.undo_depth(), depth, "no-op not recorded");
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().used_layers(), [2, 5]);
        assert!(s.run_line("layer.move 7 up").is_err());
        assert!(s.run_line("layer.move 5 sideways").is_err());
        assert!(s.run_line("layer.set_order 1 2 3").is_err());
    }

    #[test]
    fn delete_removes_shapes_and_undo_restores() {
        let mut s = session(&[
            "shape.add_rect 0 0 1 1 @2",
            "shape.add_rect 5 0 1 1 @3",
            "shape.add_rect 9 0 1 1 @2",
        ]);
        let before = s.project().clone();
        s.run_line("layer.delete 2").unwrap();
        assert_eq!(s.project().order(), [2]);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &before);
        let depth = s.undo_depth();
        s.run_line("layer.delete 7").unwrap();
        assert_eq!(s.undo_depth(), depth);
    }

    #[test]
    fn select_and_set_layer() {
        let mut s = session(&["shape.add_rect 0 0 1 1 @2", "shape.add_rect 5 0 1 1 @3"]);
        s.run_line("layer.select 2").unwrap();
        assert_eq!(s.project().selection(), [1]);
        s.run_line("layer.select 3 add").unwrap();
        assert_eq!(s.project().selection(), [1, 2]);
        s.run_line("shape.set_layer 7").unwrap();
        assert_eq!(s.project().used_layers(), [7]);
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().used_layers(), [2, 3]);
        assert!(s.run_line("shape.set_layer 30 1").is_err());
        assert!(s.run_line("layer.select 2 replace").is_err());
    }

    #[test]
    fn add_commands_take_a_color() {
        let mut s = session(&[
            "shape.add_ellipse 5 5 2 2 @29",
            "shape.add_polyline closed 0 0 5 0 3 3 @4",
        ]);
        assert_eq!(s.project().shape(1).unwrap().layer(), 29);
        assert_eq!(s.project().shape(2).unwrap().layer(), 4);
        assert!(s.run_line("shape.add_line 0 0 1 1 @30").is_err());
        assert!(s.run_line("shape.add_line 0 0 1 1 @x").is_err());
    }
}
