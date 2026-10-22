//! `rotary.*` and `registration.*` (`docs/spec/rotary.md`,
//! `docs/spec/registration.md`) and their device commands.

use lasercraft_device::{Request, StreamKind};
use lasercraft_geom::taper::{Taper, TopEdge};
use lasercraft_geom::{Affine, Path, Point, SubPath};
use lasercraft_project::{Fit, Geometry, Project, ROTARY_KEYS, Registration, Rotary, Target};

use super::device::{DEFAULT_SPEED, speed, xy};
use super::export::key_value;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

/// Head moves smaller than this do not count as moving, mm.
const MIN_MOVE: f64 = 1.0;
/// Targets closer than this are refused, mm.
const MIN_SEPARATION: f64 = 10.0;
/// A scaled fit off by more than this share is refused.
const MAX_SCALE_ERROR: f64 = 0.05;

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "rotary.set",
            "rotary.set <key> <value> [key=value ...]",
            "Change rotary settings: enabled on|off, kind rollers|chuck, mm_per_turn, roller_diameter, object_diameter, mirror on|off.",
            rotary_set,
        ),
        cmd(
            "rotary.clear",
            "rotary.clear",
            "Forget the rotary settings (back to the defaults, off).",
            rotary_clear,
        ),
        cmd(
            "device.rotary_test",
            "device.rotary_test [speed=<mm/s>]",
            "Turn the rotary one full turn and back with the laser off.",
            rotary_test,
        ),
        cmd(
            "registration.set_target",
            "registration.set_target <1|2> [id] head=X,Y",
            "Set a print registration target from the selected object and the head position.",
            set_target,
        ),
        cmd(
            "registration.fit",
            "registration.fit <scaled|rigid>",
            "Fit the job to the targets, with or without scaling.",
            fit,
        ),
        cmd(
            "registration.clear",
            "registration.clear",
            "Remove the print registration targets and fit.",
            clear,
        ),
        cmd(
            "shape.taper",
            "shape.taper <top|left|right> <top_diameter> <length> <bottom_diameter> [ids]",
            "Narrow the selection for a tapered object on the rotary.",
            taper,
        ),
        cmd(
            "registration.expected",
            "registration.expected [id] [speed=<mm/s>]",
            "Move the head to where target 2 should be, laser off.",
            expected,
        ),
    ]
}

/// The invocation that puts the rotary settings back to `old`.
fn rotary_inverse(old: &Rotary) -> Invocation {
    let mut args: Vec<String> = Vec::new();
    for key in ROTARY_KEYS {
        let value = old.get(key).unwrap_or_default();
        if args.is_empty() {
            args.push(key.to_owned());
            args.push(value);
        } else {
            args.push(format!("{key}={value}"));
        }
    }
    Invocation::new("rotary.set", args)
}

fn rotary_set(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 2 {
        return Err(args.usage_error());
    }
    let old = project.rotary();
    let mut new = old.clone();
    let mut pairs = vec![(args.str(0)?, args.str(1)?)];
    for i in 2..args.len() {
        pairs.push(key_value(args, args.str(i)?)?);
    }
    for (key, value) in &pairs {
        new.set(key, value)?;
    }
    project.set_rotary(new.clone())?;
    let mut message = if new.enabled {
        format!(
            "rotary on: {}, {} mm per turn, circumference {:.1} mm",
            new.kind,
            lasercraft_plan::num(new.mm_per_turn),
            new.circumference()
        )
    } else {
        "rotary settings stored; rotary is off".to_owned()
    };
    if new.enabled && !old.enabled {
        message.push_str(
            "\nGRBL's soft limits ($20=1) may stop rotary jobs with alarm 2; turn them off while the rotary is in use.",
        );
    }
    Ok(Outcome::undoable(message, rotary_inverse(&old)))
}

fn rotary_clear(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let old = project.rotary();
    project.set_rotary(Rotary::default())?;
    Ok(Outcome::undoable(
        "rotary settings cleared",
        rotary_inverse(&old),
    ))
}

fn rotary_test(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 1)?;
    let mut feed = DEFAULT_SPEED;
    if args.len() == 1 {
        let (key, value) = key_value(args, args.str(0)?)?;
        if key != "speed" {
            return Err(args.invalid(format!("unknown option `{key}`")));
        }
        feed = speed(args, value)?;
    }
    let r = project.rotary();
    let turn = if r.mirror {
        -r.mm_per_turn
    } else {
        r.mm_per_turn
    };
    let n = |v: f64| lasercraft_plan::num(v);
    let gcode = format!(
        "(LaserCraft rotary test: laser off)\nG21 G91\nM5 S0\nG1 Y{} F{}\nG4 P1\nG1 Y{}\nG90\n",
        n(turn),
        n(feed * 60.0),
        n(-turn)
    );
    Ok(Outcome::device(
        format!(
            "turning the rotary one turn ({} mm of Y) and back, laser off",
            n(r.mm_per_turn)
        ),
        Request::Stream {
            kind: StreamKind::Frame,
            gcode,
        },
    ))
}

/// The one object `ids` make up (a shape or a top-level group), its id and
/// its center.
fn one_object(
    project: &Project,
    ids: &[u32],
    args: &Args<'_>,
    n: &str,
) -> Result<(u32, Point), CommandError> {
    let objects = project.objects_of(ids)?;
    if objects.len() != 1 {
        return Err(args.invalid(format!(
            "Cannot set target {n}: {} objects are selected. Select one shape, or group the target's parts first.",
            objects.len()
        )));
    }
    let members = &objects[0];
    let first = project
        .shape(members[0])
        .ok_or_else(|| args.invalid("the target does not exist"))?;
    let id = if members.len() > 1 || first.top_group().is_some() {
        first.top_group().unwrap_or(members[0])
    } else {
        members[0]
    };
    let bounds = project
        .bounds_of(members)?
        .ok_or_else(|| args.invalid("the target has no size"))?;
    Ok((id, bounds.center()))
}

fn head_arg(
    args: &Args<'_>,
    from: usize,
    n: &str,
) -> Result<(Option<Point>, Vec<String>), CommandError> {
    let mut head = None;
    let mut rest = Vec::new();
    for i in from..args.len() {
        let raw = args.str(i)?;
        match raw.split_once('=') {
            Some(("head", value)) => head = Some(xy(args, "head", value)?),
            Some((key, _)) => return Err(args.invalid(format!("unknown option `{key}`"))),
            None => rest.push(raw.to_owned()),
        }
    }
    if head.is_none() {
        return Err(args.invalid(format!(
            "Cannot set target {n}: the machine did not report its position. Wait until it is Ready, then try again."
        )));
    }
    Ok((head, rest))
}

fn registration_inverse(old: &Registration) -> Invocation {
    if old.targets.is_empty() {
        return Invocation::bare("registration.clear");
    }
    let mut steps = vec![Invocation::bare("registration.clear").to_string()];
    for (i, t) in old.targets.iter().enumerate() {
        steps.push(
            Invocation::new(
                "registration.restore",
                [
                    (i + 1).to_string(),
                    t.object.to_string(),
                    format!("{},{}", t.design.x, t.design.y),
                    format!("{},{}", t.machine.x, t.machine.y),
                ],
            )
            .to_string(),
        );
    }
    if old.fit != Fit::None {
        steps.push(Invocation::new("registration.fit", [old.fit.as_str()]).to_string());
    }
    Invocation::new(crate::BATCH, steps)
}

fn set_target(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() == 0 {
        return Err(args.usage_error());
    }
    let n = args.str(0)?;
    if n != "1" && n != "2" {
        return Err(args.invalid("the target is 1 or 2"));
    }
    let (head, rest) = head_arg(args, 1, n)?;
    let head = head.unwrap_or(Point::new(0.0, 0.0));
    let ids = if rest.is_empty() {
        let selection = project.selection();
        if selection.is_empty() {
            return Err(args.invalid(format!(
                "Cannot set target {n}: 0 objects are selected. Select one shape, or group the target's parts first."
            )));
        }
        selection
    } else {
        rest.iter()
            .map(|r| {
                r.parse::<u32>()
                    .ok()
                    .filter(|id| project.shape(*id).is_some())
                    .ok_or_else(|| args.invalid(format!("`{r}` is not a shape id")))
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let (object, design) = one_object(project, &ids, args, n)?;
    let old = project.registration();
    let mut reg = old.clone();
    let target = Target {
        object,
        design,
        machine: head,
    };
    if n == "1" {
        reg.targets = vec![target];
        reg.fit = Fit::None;
    } else {
        let Some(first) = old.targets.first().copied() else {
            return Err(args.invalid("Set target 1 first."));
        };
        if first.object == object {
            return Err(args.invalid(
                "Cannot set target 2: it is the same object as target 1. Select the second target.",
            ));
        }
        if first.machine.distance(head) < MIN_MOVE {
            return Err(args.invalid(
                "Cannot set target 2: the head is still where target 1 was set. Move the head over the second mark first.",
            ));
        }
        let apart = first
            .machine
            .distance(head)
            .min(first.design.distance(design));
        if apart < MIN_SEPARATION {
            return Err(args.invalid(format!(
                "Cannot set target 2: the targets are only {} mm apart. Use targets at least {} mm apart.",
                lasercraft_plan::num((apart * 10.0).round() / 10.0),
                lasercraft_plan::num(MIN_SEPARATION)
            )));
        }
        reg.targets = vec![first, target];
        reg.fit = Fit::None;
    }
    project.set_registration(reg)?;
    Ok(Outcome::undoable(
        format!(
            "target {n} set: object {object} at X{} Y{}",
            lasercraft_plan::num(head.x),
            lasercraft_plan::num(head.y)
        ),
        registration_inverse(&old),
    ))
}

/// Restores one target exactly (undo of the registration commands).
fn restore(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(4, 4)?;
    let n = args.str(0)?;
    let object = args.parse::<u32>(1, "object id")?;
    let design = xy(args, "design", args.str(2)?)?;
    let machine = xy(args, "machine", args.str(3)?)?;
    let old = project.registration();
    let mut reg = old.clone();
    let t = Target {
        object,
        design,
        machine,
    };
    match (n, reg.targets.len()) {
        ("1", _) => reg.targets = vec![t],
        ("2", 1 | 2) => {
            reg.targets.truncate(1);
            reg.targets.push(t);
        }
        _ => return Err(args.invalid("Set target 1 first.")),
    }
    reg.fit = Fit::None;
    project.set_registration(reg)?;
    Ok(Outcome::undoable(
        format!("target {n} restored"),
        registration_inverse(&old),
    ))
}

fn fit(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let fit = match args.str(0)? {
        "scaled" => Fit::Scaled,
        "rigid" => Fit::Rigid,
        _ => return Err(args.invalid("the fit is scaled or rigid")),
    };
    let old = project.registration();
    if old.targets.len() != 2 {
        return Err(args.invalid("Set both targets first."));
    }
    let scale = old.scale().unwrap_or(1.0);
    let off = (scale - 1.0).abs();
    if fit == Fit::Scaled && off > MAX_SCALE_ERROR {
        return Err(args.invalid(format!(
            "the targets are {:.1}% {} apart on the bed than in the design; check the targets",
            off * 100.0,
            if scale > 1.0 { "farther" } else { "closer" }
        )));
    }
    let mut reg = old.clone();
    reg.fit = fit;
    project.set_registration(reg)?;
    let mut message = match fit {
        Fit::Scaled => "job fitted to the targets (moved, turned and scaled)".to_owned(),
        _ => "job placed on the targets (moved and turned)".to_owned(),
    };
    if off > 0.01 {
        message.push_str(&format!(
            "\nWarning: the targets are {:.1}% {} apart on the bed than in the design.",
            off * 100.0,
            if scale > 1.0 { "farther" } else { "closer" }
        ));
    }
    Ok(Outcome::undoable(message, registration_inverse(&old)))
}

fn clear(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let old = project.registration();
    project.set_registration(Registration::default())?;
    Ok(Outcome::undoable(
        "print registration cleared",
        registration_inverse(&old),
    ))
}

fn expected(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let mut feed = DEFAULT_SPEED;
    let mut ids = Vec::new();
    for i in 0..args.len() {
        let raw = args.str(i)?;
        match raw.split_once('=') {
            Some(("speed", v)) => feed = speed(args, v)?,
            Some((key, _)) => return Err(args.invalid(format!("unknown option `{key}`"))),
            None => ids.push(args.parse::<u32>(i, "shape id")?),
        }
    }
    let reg = project.registration();
    let Some(first) = reg.targets.first().copied() else {
        return Err(args.invalid("Set target 1 first."));
    };
    if ids.is_empty() {
        ids = project.selection();
    }
    if ids.is_empty() {
        return Err(args.invalid("Select the second target first."));
    }
    let (_, design) = one_object(project, &ids, args, "2")?;
    let area = project.work_area();
    let at = area.to_document(first.machine) + (design - first.design);
    let to = area.to_machine(at);
    if !(0.0..=area.width_mm).contains(&to.x) || !(0.0..=area.height_mm).contains(&to.y) {
        return Err(args.invalid("the expected target 2 is outside the work area; check target 1"));
    }
    Ok(Outcome::device(
        format!(
            "moving to the expected target 2 at X{} Y{}",
            lasercraft_plan::num(to.x),
            lasercraft_plan::num(to.y)
        ),
        Request::Jog {
            x: to.x,
            y: to.y,
            feed: feed * 60.0,
            relative: false,
        },
    ))
}

/// Curves are turned into points this close to the curve, mm.
const TAPER_TOLERANCE: f64 = 0.05;

fn taper(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 4 {
        return Err(args.usage_error());
    }
    let top = TopEdge::parse(args.str(0)?)
        .ok_or_else(|| args.invalid("the top is top, left or right"))?;
    let t = Taper {
        top,
        top_diameter: args.f64(1, "top diameter")?,
        length: args.f64(2, "length")?,
        bottom_diameter: args.f64(3, "bottom diameter")?,
    };
    t.validate().map_err(|e| args.invalid(e))?;
    let ids = args.ids(4, project)?;
    for id in &ids {
        if project.shape(*id).is_some_and(|s| s.geometry().is_image()) {
            return Err(args.invalid(format!(
                "Taper correction works on vector shapes; image {id} is selected."
            )));
        }
    }
    let bounds = project
        .bounds_of(&ids)?
        .ok_or_else(|| args.invalid("the selection has no size"))?;
    let mut changes = Vec::new();
    for id in &ids {
        let Some(shape) = project.shape(*id) else {
            continue;
        };
        let subpaths: Vec<SubPath> = shape
            .outlines(TAPER_TOLERANCE)
            .into_iter()
            .filter_map(|(points, closed)| {
                SubPath::from_points(&t.warp(&points, closed, &bounds), closed)
            })
            .collect();
        if subpaths.is_empty() {
            continue;
        }
        changes.push((
            *id,
            Geometry::Path(Path { subpaths }),
            shape.geometry().clone(),
            *shape.transform(),
        ));
    }
    let mut inverses = Vec::new();
    for (id, geometry, old_geometry, old_transform) in changes {
        geometry.validate()?;
        project.set_shape_geometry(id, geometry)?;
        project.set_shape_transform(id, Affine::IDENTITY)?;
        inverses.push(Invocation::new(
            "shape.set_geometry",
            [
                id.to_string(),
                serde_json::to_string(&old_geometry).unwrap_or_default(),
            ],
        ));
        let mut tr = vec![id.to_string()];
        tr.extend(old_transform.m.iter().map(ToString::to_string));
        inverses.push(Invocation::new("shape.set_transform", tr));
    }
    if inverses.is_empty() {
        return Err(args.invalid("nothing to correct"));
    }
    Ok(Outcome::undoable(
        format!(
            "taper correction on {} shape(s): {:.1}% narrower at the narrow end",
            inverses.len() / 2,
            t.correction(&bounds) * 100.0
        ),
        super::combine(inverses),
    ))
}

/// `registration.restore` is only for undo; it is registered with the rest
/// so batches can run it.
pub(super) fn hidden() -> Vec<Box<dyn Command>> {
    vec![cmd(
        "registration.restore",
        "registration.restore <1|2> <object> <x,y> <x,y>",
        "Put a registration target back exactly (used by undo).",
        restore,
    )]
}

#[cfg(test)]
mod tests;
