//! `preview`: plan a job headlessly, print its statistics and optionally
//! write the toolpath as SVG (`docs/spec/cli.md`, `docs/spec/planning.md`).

use std::fmt::Write as _;
use std::path::Path;

use lasercraft_geom::{Point, Rect};
use lasercraft_plan::{Machine, MoveKind, Plan, num};
use lasercraft_project::Project;
use lasercraft_project::layer::palette;

use crate::{CliError, failed, usage};

struct Options {
    file: String,
    svg: Option<String>,
    moves: bool,
    overrides: Overrides,
}

/// Machine flags given on the command line.
#[derive(Default)]
struct Overrides {
    max_speed: Option<f64>,
    acceleration: Option<f64>,
    max_power: Option<f64>,
}

impl Overrides {
    fn apply(&self, machine: &mut Machine) {
        machine.max_speed = self.max_speed.unwrap_or(machine.max_speed);
        machine.acceleration = self.acceleration.unwrap_or(machine.acceleration);
        machine.max_power = self.max_power.unwrap_or(machine.max_power);
    }
}

fn value(flag: &str, text: Option<&String>) -> Result<f64, CliError> {
    text.and_then(|t| t.parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .ok_or_else(|| usage(&format!("{flag} needs a number")))
}

fn options(args: &[String]) -> Result<Options, CliError> {
    let mut file = None;
    let mut svg = None;
    let mut moves = false;
    let mut overrides = Overrides::default();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--svg" => {
                svg = Some(
                    rest.next()
                        .ok_or_else(|| usage("--svg needs a file"))?
                        .clone(),
                );
            }
            "--moves" => moves = true,
            "--max-speed" => overrides.max_speed = Some(value(arg, rest.next())?),
            "--accel" => overrides.acceleration = Some(value(arg, rest.next())?),
            "--max-power" => overrides.max_power = Some(value(arg, rest.next())?),
            flag if flag.starts_with("--") => {
                return Err(usage(&format!("unknown preview option `{flag}`")));
            }
            _ if file.is_none() => file = Some(arg.clone()),
            _ => return Err(usage("preview takes one project file")),
        }
    }
    let file = file.ok_or_else(|| usage("preview needs a project file"))?;
    Ok(Options {
        file,
        svg,
        moves,
        overrides,
    })
}

/// The rotary and print registration line of a preview, if either is on
/// (`docs/spec/rotary.md`, `docs/spec/registration.md`).
#[must_use]
pub fn setups_line(project: &Project, plan: &lasercraft_plan::Plan) -> Option<String> {
    let mut parts = Vec::new();
    if project.rotary_on() {
        let height = plan.stats.bounds.map_or(0.0, |b| b.height());
        parts.push(format!(
            "rotary: Y turns {}° for the design's height; Y is not checked against the bed",
            num((project.rotary().degrees(height) * 10.0).round() / 10.0)
        ));
    }
    if project.registration_transform().is_some() {
        parts.push(format!(
            "print registration: job fitted to the targets ({})",
            project.registration().fit.as_str()
        ));
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

/// Runs `preview <project.lcproj> [--svg out.svg] [--moves] [machine flags]`.
pub(crate) fn preview(args: &[String]) -> Result<String, CliError> {
    let o = options(args)?;
    let project = Project::load(Path::new(&o.file)).map_err(failed)?;
    // The project's device profile first, then the flags.
    let (mut machine, _, warning) = lasercraft_commands::job_settings(&project, &crate::workshop());
    o.overrides.apply(&mut machine);
    let plan = lasercraft_plan::plan(&project, &machine).map_err(failed)?;
    let mut out = format!("job \"{}\"\n", project.name());
    if let Some(w) = warning {
        let _ = writeln!(out, "{w}");
    }
    if o.moves {
        out.push_str(&plan.dump());
    } else {
        out.push_str(&plan.header());
        out.push_str(&plan.summary());
    }
    let area = project.work_area();
    if let Some(line) = setups_line(&project, &plan) {
        let _ = writeln!(out, "{line}");
    }
    let check = if project.rotary_on() {
        plan.check_bounds_x(area)
    } else {
        plan.check_bounds(area)
    };
    match check {
        Ok(()) => {
            let _ = writeln!(
                out,
                "fits the work area ({} × {} mm)",
                num(area.width_mm),
                num(area.height_mm)
            );
        }
        Err(e) => {
            let _ = writeln!(out, "does not fit: {e}");
        }
    }
    if let Some(path) = &o.svg {
        std::fs::write(path, svg(&project, &plan))
            .map_err(|e| failed(format!("cannot write {path}: {e}")))?;
        let _ = writeln!(out, "wrote {path}");
    }
    Ok(out)
}

fn hex(color: u8) -> String {
    let [r, g, b] = palette()
        .get(usize::from(color))
        .copied()
        .unwrap_or([128, 128, 128]);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The toolpath as SVG in document millimetres: the bed outline, burn moves
/// in layer colors, unlit moves dashed orange, travel dashed red.
pub(crate) fn svg(project: &Project, plan: &Plan) -> String {
    let area = project.work_area();
    let bed = Rect::from_corners(Point::ZERO, Point::new(area.width_mm, area.height_mm));
    let view = plan.stats.bounds.map_or(bed, |b| b.union(bed));
    let margin = 5.0;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" width=\"{}mm\" height=\"{}mm\">\n",
        num(view.min.x - margin),
        num(view.min.y - margin),
        num(view.width() + 2.0 * margin),
        num(view.height() + 2.0 * margin),
        num(view.width() + 2.0 * margin),
        num(view.height() + 2.0 * margin),
    );
    let _ = writeln!(
        out,
        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"#2a2\" stroke-width=\"0.3\"/>",
        num(area.width_mm),
        num(area.height_mm)
    );
    // One path per run of moves with the same look.
    let style = |kind: MoveKind, layer: u8| match kind {
        MoveKind::Burn => format!("stroke=\"{}\" stroke-width=\"0.2\"", hex(layer)),
        MoveKind::Unlit => {
            "stroke=\"#f80\" stroke-width=\"0.1\" stroke-dasharray=\"0.5 0.5\"".to_owned()
        }
        MoveKind::Travel => {
            "stroke=\"#d22\" stroke-width=\"0.1\" stroke-dasharray=\"1 1\"".to_owned()
        }
    };
    let mut current: Option<(String, String, Point)> = None;
    let flush = |out: &mut String, run: Option<(String, String, Point)>| {
        if let Some((look, d, _)) = run {
            let _ = writeln!(out, "<path fill=\"none\" {look} d=\"{d}\"/>");
        }
    };
    for m in &plan.moves {
        let look = style(m.kind, m.layer);
        match &mut current {
            Some((l, d, end)) if *l == look => {
                if *end != m.from {
                    let _ = write!(d, " M{} {}", num(m.from.x), num(m.from.y));
                }
                let _ = write!(d, " L{} {}", num(m.to.x), num(m.to.y));
                *end = m.to;
            }
            _ => {
                flush(&mut out, current.take());
                let d = format!(
                    "M{} {} L{} {}",
                    num(m.from.x),
                    num(m.from.y),
                    num(m.to.x),
                    num(m.to.y)
                );
                current = Some((look, d, m.to));
            }
        }
    }
    flush(&mut out, current);
    out.push_str("</svg>\n");
    out
}
