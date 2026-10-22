//! `camera.*` and the alignment pattern's device commands
//! (`docs/spec/camera.md`, "Commands"). Cameras belong to the project's
//! device profile; changing them is workshop data, saved at once and not
//! undoable.

use std::path::Path;

use lasercraft_camera::align::{self, Alignment};
use lasercraft_camera::calibrate::{View, calibrate};
use lasercraft_camera::dots::find_sheet;
use lasercraft_camera::lens::{Lens, Model};
use lasercraft_camera::marks::{DOT, Pattern, RING_INNER, find_marks};
use lasercraft_camera::overlay::Overlay;
use lasercraft_camera::{Picture, sheet};
use lasercraft_device::{Request, StreamKind};
use lasercraft_gcode::{FrameShape, Placement, export_plan, frame_plan};
use lasercraft_geom::{Affine, Path as GeomPath, Point};
use lasercraft_project::{Geometry, Mode, Project};
use lasercraft_workshop::{Camera, Mount, Source};
use serde::Deserialize;

use super::device::{DEFAULT_SPEED, speed, xy};
use super::export::{apply_option, key_value, write_atomically};
use super::profile::{saved, ws};
use crate::args::Args;
use crate::command::ccmd;
use crate::{Command, CommandError, Context, Outcome, job_settings};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        ccmd(
            "camera.add",
            "camera.add <name> <usb|network|simulated> [device=… index=… format=WxH | url=… rate=…] [mount=bed|head]",
            "Add a camera to the project's device profile.",
            add,
        ),
        ccmd(
            "camera.remove",
            "camera.remove <name>",
            "Remove a camera with its lens correction and alignments.",
            remove,
        ),
        ccmd(
            "camera.rename",
            "camera.rename <old> <new>",
            "Rename a camera.",
            rename,
        ),
        ccmd(
            "camera.set",
            "camera.set <name> <mount|use|opacity|fit_width|fit_height|fit_dx|fit_dy|skip|device|index|format|url|rate> <value>",
            "Change one camera setting.",
            set,
        ),
        ccmd(
            "camera.set_control",
            "camera.set_control <name> <control> <value|default>",
            "Store a picture setting of a camera (brightness, contrast, exposure, focus, …).",
            set_control,
        ),
        ccmd(
            "camera.pattern",
            "camera.pattern <path.svg>",
            "Write the calibration sheet as SVG.",
            pattern_svg,
        ),
        ccmd(
            "camera.calibrate",
            "camera.calibrate <name> <standard|wide|fisheye> <picture> ...",
            "Find the calibration sheet in pictures and store the camera's lens correction.",
            calibrate_cmd,
        ),
        ccmd(
            "camera.set_lens",
            "camera.set_lens <name> <json|path.lccam|none>",
            "Store a lens correction, or remove it.",
            set_lens,
        ),
        ccmd(
            "camera.align",
            "camera.align <name> <json> [head=X,Y]",
            "Store a bed alignment from four picture points and their bed points.",
            align_cmd,
        ),
        ccmd(
            "camera.align_find",
            "camera.align_find <name> <picture> [pattern json] [height=<mm>] [head=X,Y]",
            "Find the burned alignment marks in a picture and store the bed alignment.",
            align_find,
        ),
        ccmd(
            "camera.use_alignment",
            "camera.use_alignment <name> <alignment>",
            "Use another stored alignment.",
            use_alignment,
        ),
        ccmd(
            "camera.remove_alignment",
            "camera.remove_alignment <name> <alignment>",
            "Remove a stored alignment.",
            remove_alignment,
        ),
        ccmd(
            "camera.import",
            "camera.import <path>",
            "Add the camera in a .lccam file.",
            import,
        ),
        ccmd(
            "camera.export",
            "camera.export <name> <path>",
            "Write a camera to a .lccam file.",
            export,
        ),
        ccmd(
            "camera.overlay",
            "camera.overlay <name> <picture> <out.png>",
            "Write the overlay of one picture on the bed.",
            overlay,
        ),
        ccmd(
            "camera.list",
            "camera.list",
            "List the device profile's cameras.",
            list,
        ),
        ccmd(
            "device.camera_pattern",
            "device.camera_pattern [json] [speed=<mm/s>] [power=<%>] [s_max=<n>] [air=m7|m8]",
            "Burn the camera alignment pattern on the connected laser.",
            burn,
        ),
        ccmd(
            "device.camera_pattern_frame",
            "device.camera_pattern_frame [json] [speed=<mm/s>] [head=X,Y]",
            "Trace the camera alignment pattern's box with the laser off.",
            burn_frame,
        ),
    ]
}

/// The project's device profile, which cameras belong to.
fn profile(cx: &Context<'_>, args: &Args<'_>) -> Result<String, CommandError> {
    let name = cx.project.device().ok_or_else(|| {
        args.invalid("Cameras belong to a device profile. Choose one in the Machine panel first.")
    })?;
    cx.workshop
        .profile(name)
        .map(|p| p.name.clone())
        .ok_or_else(|| {
            args.invalid(format!(
                "the device profile \"{name}\" is not on this computer"
            ))
        })
}

fn picture(args: &Args<'_>, path: &str) -> Result<Picture, CommandError> {
    Picture::load(Path::new(path)).map_err(|e| args.invalid(e.0))
}

fn add(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 2 {
        return Err(args.usage_error());
    }
    let p = profile(cx, args)?;
    let name = args.str(0)?;
    let source = match args.str(1)? {
        "usb" => Source::Usb {
            device: String::new(),
            index: 0,
            format: String::new(),
        },
        "network" => Source::Network {
            url: "http://".to_owned(),
            rate: 2.0,
        },
        "simulated" => Source::Simulated,
        other => return Err(args.invalid(format!("`{other}` is not usb, network or simulated"))),
    };
    let network = matches!(source, Source::Network { .. });
    let mut camera = Camera::new(name, source).map_err(ws(args))?;
    let mut has_url = false;
    for i in 2..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        has_url |= key == "url";
        camera.set(key, value).map_err(ws(args))?;
    }
    if network && !has_url {
        return Err(args.invalid("a network camera needs url=<address>"));
    }
    let summary = camera.source.summary();
    cx.workshop.add_camera(&p, camera).map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("added camera \"{}\" ({summary}) to {p}", name.trim()),
    ))
}

fn remove(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let p = profile(cx, args)?;
    let removed = cx
        .workshop
        .remove_camera(&p, args.str(0)?)
        .map_err(ws(args))?;
    Ok(saved(cx, format!("removed camera \"{}\"", removed.name)))
}

fn rename(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let p = profile(cx, args)?;
    cx.workshop
        .rename_camera(&p, args.str(0)?, args.str(1)?)
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("renamed camera to \"{}\"", args.str(1)?.trim()),
    ))
}

fn set(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let p = profile(cx, args)?;
    let (name, key, value) = (args.str(0)?, args.str(1)?, args.str(2)?);
    cx.workshop
        .edit_camera(&p, name, |c| c.set(key, value))
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("camera \"{}\" {key} = {value}", name.trim()),
    ))
}

fn set_control(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let p = profile(cx, args)?;
    let (name, key, raw) = (args.str(0)?, args.str(1)?, args.str(2)?.trim());
    let value = if raw == "default" {
        None
    } else {
        Some(
            raw.parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| args.invalid(format!("`{raw}` is not a number or `default`")))?,
        )
    };
    cx.workshop
        .edit_camera(&p, name, |c| c.set_control(key, value))
        .map_err(ws(args))?;
    let title = lasercraft_camera::controls::known(key)
        .map_or_else(String::new, lasercraft_camera::controls::title);
    Ok(saved(
        cx,
        match value {
            Some(v) => format!(
                "camera \"{}\" {title}: {}",
                name.trim(),
                lasercraft_plan::num(v)
            ),
            None => format!("camera \"{}\" {title}: the camera's default", name.trim()),
        },
    ))
}

fn pattern_svg(_: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let path = args.str(0)?;
    write_atomically(Path::new(path), &sheet::svg())
        .map_err(|e| args.invalid(format!("cannot write {path}: {e}")))?;
    Ok(Outcome::done(format!(
        "wrote the calibration sheet to {path}: print it flat, at any size"
    )))
}

fn calibrate_cmd(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 3 {
        return Err(args.usage_error());
    }
    let p = profile(cx, args)?;
    let name = args.str(0)?;
    cx.workshop.camera(&p, name).map_err(ws(args))?;
    let model = Model::parse(args.str(1)?)
        .ok_or_else(|| args.invalid("the model is standard, wide or fisheye"))?;
    let mut views: Vec<View> = Vec::new();
    let mut lines = Vec::new();
    let mut size = None;
    for i in 2..args.len() {
        let path = args.str(i)?;
        let pic = picture(args, path)?;
        let this = (pic.width(), pic.height());
        if size.is_some_and(|s| s != this) {
            return Err(args.invalid(format!("{path} has another size than the first picture")));
        }
        size = Some(this);
        match find_sheet(&pic) {
            Ok(dots) => {
                lines.push(format!("{path}: {} dots", dots.len()));
                views.push(
                    dots.into_iter()
                        .map(|((i, j), px)| (sheet::point(i, j), px))
                        .collect(),
                );
            }
            Err(e) => lines.push(format!("{path}: left out, {e}")),
        }
    }
    let (w, h) = size.unwrap_or((1, 1));
    let mut lens = calibrate(model, w, h, &views)
        .map_err(|e| args.invalid(format!("{}\n{}", e.0, lines.join("\n"))))?;
    lens.created = lasercraft_camera::timestamp_now();
    let score = lens.score();
    let rms = lens.rms;
    cx.workshop
        .edit_camera(&p, name, |c| {
            c.lens = Some(lens);
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!(
            "{} lens correction for \"{}\": fit {score:.0}% ({rms:.2} pixels)\n{}",
            model.title(),
            name.trim(),
            lines.join("\n")
        ),
    ))
}

fn set_lens(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let p = profile(cx, args)?;
    let name = args.str(0)?;
    let lens: Option<Lens> = match args.str(1)? {
        "none" => None,
        text if !text.trim_start().starts_with('{') => {
            // A camera file: take its lens correction.
            let file: serde_json::Value = std::fs::read_to_string(text)
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .ok_or_else(|| args.invalid(format!("cannot read the camera file {text}")))?;
            let lens = file
                .pointer("/camera/lens")
                .filter(|v| !v.is_null())
                .ok_or_else(|| args.invalid(format!("{text} holds no lens correction")))?;
            let lens: Lens = serde_json::from_value(lens.clone())
                .map_err(|e| args.invalid(format!("not a lens correction: {e}")))?;
            lens.validate().map_err(|e| args.invalid(e.0))?;
            Some(lens)
        }
        text => {
            let lens: Lens = serde_json::from_str(text)
                .map_err(|e| args.invalid(format!("not a lens correction: {e}")))?;
            lens.validate().map_err(|e| args.invalid(e.0))?;
            Some(lens)
        }
    };
    let removed = lens.is_none();
    cx.workshop
        .edit_camera(&p, name, |c| {
            c.lens = lens;
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        if removed {
            format!("removed the lens correction of \"{}\"", name.trim())
        } else {
            format!("stored the lens correction of \"{}\"", name.trim())
        },
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignInput {
    picture: [[f64; 2]; 4],
    bed: [[f64; 2]; 4],
    #[serde(default)]
    height: f64,
}

/// The material height an alignment picture was taken at, and where the
/// head was (`head=X,Y`, machine mm).
#[derive(Debug, Clone, Copy, Default)]
struct Taken {
    height: f64,
    head: Option<Point>,
}

fn store_alignment(
    cx: &mut Context<'_>,
    args: &Args<'_>,
    profile: &str,
    name: &str,
    raw: [Point; 4],
    bed: [Point; 4],
    taken: Taken,
) -> Result<String, CommandError> {
    let camera = cx.workshop.camera(profile, name).map_err(ws(args))?;
    // A bed camera ignores the head position the app adds.
    let head = match (camera.mount, taken.head) {
        (Mount::Head, None) => {
            return Err(args.invalid(
                "A head camera's alignment needs the head's position: connect the laser.",
            ));
        }
        (Mount::Head, head) => head.map(|h| cx.project.work_area().to_document(h)),
        (Mount::Bed, _) => None,
    };
    // Without a lens correction the size is unknown here; 0 means "the size
    // of the picture" (`Camera::lens_or_plain`).
    let lens = camera.lens_or_plain(0, 0);
    let height = taken.height;
    let matrix = align::fit(&lens, &raw, &bed).map_err(|e| args.invalid(e.0))?;
    let label = if height > 0.0 {
        format!("{} mm", lasercraft_plan::num(height))
    } else {
        "Bed".to_owned()
    };
    let alignment = Alignment {
        name: label.clone(),
        height,
        matrix,
        lens: lens.created.clone(),
        width: lens.width,
        height_px: lens.height,
        created: lasercraft_camera::timestamp_now(),
        head,
    };
    cx.workshop
        .edit_camera(profile, name, |c| {
            c.add_alignment(alignment);
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(label)
}

fn align_cmd(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 3)?;
    let p = profile(cx, args)?;
    let name = args.str(0)?;
    let input: AlignInput = serde_json::from_str(args.str(1)?)
        .map_err(|e| args.invalid(format!("alignment points are not valid JSON: {e}")))?;
    let mut head = None;
    if args.len() > 2 {
        let (key, value) = key_value(args, args.str(2)?)?;
        if key != "head" {
            return Err(args.invalid(format!("unknown option `{key}`")));
        }
        head = Some(xy(args, key, value)?);
    }
    let pt = |v: [f64; 2]| Point::new(v[0], v[1]);
    let label = store_alignment(
        cx,
        args,
        &p,
        name,
        input.picture.map(pt),
        input.bed.map(pt),
        Taken {
            height: input.height,
            head,
        },
    )?;
    Ok(saved(
        cx,
        format!("aligned \"{}\" to the bed ({label})", name.trim()),
    ))
}

fn pattern_arg(
    cx: &Context<'_>,
    args: &Args<'_>,
    text: Option<&str>,
) -> Result<Pattern, CommandError> {
    let area = cx.project.work_area();
    let pattern = match text {
        Some(t) => serde_json::from_str::<Pattern>(t)
            .map_err(|e| args.invalid(format!("the pattern is not valid JSON: {e}")))?,
        None => Pattern::default_for(area.width_mm, area.height_mm),
    };
    pattern
        .validate(area.width_mm, area.height_mm)
        .map_err(|e| args.invalid(e.0))?;
    Ok(pattern)
}

fn align_find(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 2 {
        return Err(args.usage_error());
    }
    let p = profile(cx, args)?;
    let name = args.str(0)?;
    let pic = picture(args, args.str(1)?)?;
    let (mut json, mut taken) = (None, Taken::default());
    for i in 2..args.len() {
        let raw = args.str(i)?;
        if raw.trim_start().starts_with('{') {
            json = Some(raw);
            continue;
        }
        let (key, value) = key_value(args, raw)?;
        match key {
            "height" => {
                taken.height = value
                    .parse::<f64>()
                    .ok()
                    .filter(|h| (0.0..=500.0).contains(h))
                    .ok_or_else(|| args.invalid("height must be 0–500 mm"))?;
            }
            "head" => taken.head = Some(xy(args, key, value)?),
            _ => return Err(args.invalid(format!("unknown option `{key}`"))),
        }
    }
    let pattern = pattern_arg(cx, args, json)?;
    let area = cx.project.work_area();
    let lens = cx
        .workshop
        .camera(&p, name)
        .map_err(ws(args))?
        .lens_or_plain(pic.width(), pic.height());
    let scaled = pic.resized(lens.width, lens.height);
    let raw = find_marks(&scaled).map_err(|e| args.invalid(e.0))?;
    let label = store_alignment(
        cx,
        args,
        &p,
        name,
        raw,
        pattern.centers(area.width_mm, area.height_mm),
        taken,
    )?;
    Ok(saved(
        cx,
        format!(
            "found the 4 marks and aligned \"{}\" to the bed ({label})",
            name.trim()
        ),
    ))
}

fn use_alignment(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let p = profile(cx, args)?;
    let (name, which) = (args.str(0)?, args.str(1)?);
    cx.workshop
        .edit_camera(&p, name, |c| {
            let i = c
                .alignments
                .iter()
                .position(|a| a.name.eq_ignore_ascii_case(which.trim()))
                .ok_or_else(|| {
                    lasercraft_workshop::WorkshopError::new(format!(
                        "no alignment named \"{which}\""
                    ))
                })?;
            let a = c.alignments.remove(i);
            c.alignments.insert(0, a);
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("\"{}\" uses the alignment {which}", name.trim()),
    ))
}

fn remove_alignment(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let p = profile(cx, args)?;
    let (name, which) = (args.str(0)?, args.str(1)?);
    cx.workshop
        .edit_camera(&p, name, |c| {
            let before = c.alignments.len();
            c.alignments
                .retain(|a| !a.name.eq_ignore_ascii_case(which.trim()));
            if c.alignments.len() == before {
                return Err(lasercraft_workshop::WorkshopError::new(format!(
                    "no alignment named \"{which}\""
                )));
            }
            Ok(())
        })
        .map_err(ws(args))?;
    Ok(saved(
        cx,
        format!("removed the alignment {which} of \"{}\"", name.trim()),
    ))
}

fn import(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let p = profile(cx, args)?;
    let name = cx
        .workshop
        .import_camera(&p, Path::new(args.str(0)?))
        .map_err(ws(args))?;
    Ok(saved(cx, format!("imported camera \"{name}\"")))
}

fn export(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let p = profile(cx, args)?;
    let path = args.str(1)?;
    cx.workshop
        .export_camera(&p, args.str(0)?, Path::new(path))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!(
        "wrote camera \"{}\" to {path}",
        args.str(0)?.trim()
    )))
}

fn overlay(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let p = profile(cx, args)?;
    let camera = cx.workshop.camera(&p, args.str(0)?).map_err(ws(args))?;
    let Some(alignment) = camera.alignment() else {
        return Err(args.invalid("the camera needs a bed alignment first"));
    };
    let pic = picture(args, args.str(1)?)?;
    let lens = camera.lens_or_plain(pic.width(), pic.height());
    let area = cx.project.work_area();
    let mut out = Overlay::empty(area.width_mm, area.height_mm);
    out.paint(&pic, &lens, alignment, &camera.fit, None);
    let path = args.str(2)?;
    std::fs::write(path, out.encode_png())
        .map_err(|e| args.invalid(format!("cannot write {path}: {e}")))?;
    Ok(Outcome::done(format!(
        "wrote the overlay to {path}: {} × {} pixels, {} mm per pixel",
        out.width,
        out.height,
        lasercraft_plan::num(out.mm_per_px)
    )))
}

fn list(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let p = profile(cx, args)?;
    let cams = cx.workshop.cameras(&p).map_err(ws(args))?;
    if cams.is_empty() {
        return Ok(Outcome::done(format!("{p} has no cameras")));
    }
    Ok(Outcome::done(
        cams.iter()
            .map(Camera::summary)
            .collect::<Vec<_>>()
            .join("\n"),
    ))
}

/// Settings for the burned pattern: a "Camera alignment" entry of the
/// open library, else 100 mm/s at 20 % (`docs/spec/camera.md`).
fn burn_settings(cx: &Context<'_>) -> (f64, f64) {
    cx.workshop
        .active()
        .library
        .entries()
        .iter()
        .find(|e| e.description.eq_ignore_ascii_case("camera alignment"))
        .map_or((100.0, 20.0), |e| (e.settings.speed, e.settings.max_power))
}

/// A scratch project holding the alignment pattern on layer 0, filled.
fn pattern_project(
    cx: &Context<'_>,
    pattern: &Pattern,
    speed: f64,
    power: f64,
) -> Result<Project, CommandError> {
    let area = *cx.project.work_area();
    let mut p = Project::default();
    p.set_work_area_size(area.width_mm, area.height_mm)?;
    p.set_origin(area.origin);
    p.set_name("Camera alignment pattern")?;
    let mut layer = p.layer(0)?.clone();
    layer.mode = Mode::Fill;
    layer.speed = speed;
    layer.max_power = power;
    layer.min_power = 0.0;
    layer.interval = 0.1;
    layer.name = "Alignment marks".to_owned();
    p.replace_layer(0, layer)?;
    let r = pattern.mark / 2.0;
    let ring = |c: Point| {
        let mut path = GeomPath::ellipse(r, r);
        path.subpaths
            .extend(GeomPath::ellipse(r * RING_INNER, r * RING_INNER).subpaths);
        path.transformed(&Affine::translate(c.x, c.y))
    };
    let dot = |c: Point| {
        (
            Geometry::Ellipse {
                rx: r * DOT,
                ry: r * DOT,
            },
            Affine::translate(c.x, c.y),
        )
    };
    for c in pattern.centers(area.width_mm, area.height_mm) {
        p.add_shape(Geometry::Path(ring(c)), Affine::IDENTITY)?;
        let (g, t) = dot(c);
        p.add_shape(g, t)?;
    }
    let (g, t) = dot(pattern.center(area.width_mm, area.height_mm));
    p.add_shape(g, t)?;
    Ok(p)
}

/// What a pattern burn or frame was asked for.
struct Burn {
    pattern: Pattern,
    speed: f64,
    power: f64,
    frame_speed: f64,
    head: Option<Point>,
    machine: lasercraft_plan::Machine,
    options: lasercraft_gcode::Options,
}

fn burn_args(cx: &Context<'_>, args: &Args<'_>, frame: bool) -> Result<Burn, CommandError> {
    let (mut speed_mm, mut power) = burn_settings(cx);
    let mut frame_speed = DEFAULT_SPEED;
    let mut json = None;
    let mut head = None;
    let (mut machine, mut options, _) = job_settings(cx.project, cx.workshop);
    for i in 0..args.len() {
        let raw = args.str(i)?;
        if raw.trim_start().starts_with('{') {
            json = Some(raw);
            continue;
        }
        let (key, value) = key_value(args, raw)?;
        match key {
            "speed" if frame => frame_speed = speed(args, value)?,
            "speed" => speed_mm = speed(args, value)?,
            "power" if !frame => {
                power = value
                    .parse::<f64>()
                    .ok()
                    .filter(|p| (0.0..=100.0).contains(p))
                    .ok_or_else(|| args.invalid("power must be 0–100 %"))?;
            }
            "head" if frame => head = Some(xy(args, key, value)?),
            _ if !frame && apply_option(args, key, value, &mut machine, &mut options)? => {}
            _ => return Err(args.invalid(format!("unknown option `{key}`"))),
        }
    }
    // The pattern is placed by bed coordinates (docs/spec/camera.md).
    options.placement = Placement::Absolute;
    let pattern = pattern_arg(cx, args, json)?;
    Ok(Burn {
        pattern,
        speed: speed_mm,
        power,
        frame_speed,
        head,
        machine,
        options,
    })
}

fn burn(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let Burn {
        pattern,
        speed: speed_mm,
        power,
        machine,
        options,
        ..
    } = burn_args(cx, args, false)?;
    let project = pattern_project(cx, &pattern, speed_mm, power)?;
    let plan =
        lasercraft_plan::plan(&project, &machine).map_err(|e| args.invalid(e.to_string()))?;
    let out =
        export_plan(&project, plan, &machine, &options).map_err(|e| args.invalid(e.to_string()))?;
    let mut message = format!(
        "burning the camera alignment pattern (absolute coordinates): {} × {} mm, marks {} mm, {} mm/s at {}%, estimated {}",
        lasercraft_plan::num(pattern.width),
        lasercraft_plan::num(pattern.height),
        lasercraft_plan::num(pattern.mark),
        lasercraft_plan::num(speed_mm),
        lasercraft_plan::num(power.min(machine.max_power)),
        lasercraft_plan::duration(out.plan.stats.total_time)
    );
    for w in &out.plan.warnings {
        message.push_str("\nWarning: ");
        message.push_str(&w.message);
    }
    Ok(Outcome::device(
        message,
        Request::Stream {
            kind: StreamKind::Job,
            gcode: out.gcode,
        },
    ))
}

fn burn_frame(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let Burn {
        pattern,
        speed: speed_mm,
        power,
        frame_speed,
        head,
        machine,
        options,
    } = burn_args(cx, args, true)?;
    let project = pattern_project(cx, &pattern, speed_mm, power)?;
    let plan =
        lasercraft_plan::plan(&project, &machine).map_err(|e| args.invalid(e.to_string()))?;
    let speed_frame = frame_speed.min(machine.max_speed);
    let gcode = frame_plan(
        &project,
        plan,
        &machine,
        &options,
        FrameShape::Box,
        speed_frame,
        head,
    )
    .map_err(|e| args.invalid(e.to_string()))?;
    Ok(Outcome::device(
        format!(
            "framing the camera alignment pattern at {} mm/s, laser off",
            lasercraft_plan::num(speed_frame)
        ),
        Request::Stream {
            kind: StreamKind::Frame,
            gcode,
        },
    ))
}

#[cfg(test)]
mod tests;
