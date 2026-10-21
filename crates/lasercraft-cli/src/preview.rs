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
    machine: Machine,
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
    let mut machine = Machine::default();
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
            "--max-speed" => machine.max_speed = value(arg, rest.next())?,
            "--accel" => machine.acceleration = value(arg, rest.next())?,
            "--max-power" => machine.max_power = value(arg, rest.next())?,
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
        machine,
    })
}

/// Runs `preview <project.lcproj> [--svg out.svg] [--moves] [machine flags]`.
pub(crate) fn preview(args: &[String]) -> Result<String, CliError> {
    let o = options(args)?;
    let project = Project::load(Path::new(&o.file)).map_err(failed)?;
    let plan = lasercraft_plan::plan(&project, &o.machine).map_err(failed)?;
    let mut out = format!("job \"{}\"\n", project.name());
    if o.moves {
        out.push_str(&plan.dump());
    } else {
        out.push_str(&plan.header());
        out.push_str(&plan.summary());
    }
    let area = project.work_area();
    match plan.check_bounds(area) {
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
fn svg(project: &Project, plan: &Plan) -> String {
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
