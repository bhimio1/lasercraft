//! `pattern`: build a test pattern headlessly and print its plan, optionally
//! writing its toolpath as SVG and its G-code (`docs/spec/cli.md`,
//! `docs/spec/test-patterns.md`).

use std::fmt::Write as _;
use std::path::Path;

use lasercraft_plan::num;
use lasercraft_plan::patterns::from_json;
use lasercraft_project::Project;

use crate::{CliError, failed, usage};

/// Runs `pattern <kind> [<json>] [--project p] [--svg out] [--gcode out]`.
pub(crate) fn pattern(args: &[String]) -> Result<String, CliError> {
    let (kind, rest) = args
        .split_first()
        .ok_or_else(|| usage("pattern needs speed_power or interval"))?;
    let mut json = None;
    let mut project = None;
    let mut svg = None;
    let mut gcode = None;
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let mut next = |what: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| usage(&format!("{arg} needs {what}")))
        };
        match arg.as_str() {
            "--project" => project = Some(next("a project file")?),
            "--svg" => svg = Some(next("a file")?),
            "--gcode" => gcode = Some(next("a file")?),
            flag if flag.starts_with("--") => {
                return Err(usage(&format!("unknown pattern option `{flag}`")));
            }
            _ if json.is_none() => json = Some(arg.clone()),
            _ => return Err(usage("pattern takes one settings object")),
        }
    }
    let project = match project {
        Some(p) => Project::load(Path::new(&p)).map_err(failed)?,
        None => Project::default(),
    };
    let (machine, options, warning) =
        lasercraft_commands::job_settings(&project, &crate::workshop());
    let pattern =
        from_json(kind, json.as_deref(), project.work_area(), &machine).map_err(failed)?;
    let mut out = format!(
        "{}: {} × {} mm\n",
        pattern.project.name(),
        num((pattern.size.0 * 100.0).round() / 100.0),
        num((pattern.size.1 * 100.0).round() / 100.0)
    );
    if let Some(w) = warning {
        let _ = writeln!(out, "{w}");
    }
    out.push_str(&pattern.plan.header());
    out.push_str(&pattern.plan.summary());
    let area = project.work_area();
    match pattern.plan.check_bounds(area) {
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
    if let Some(path) = &svg {
        std::fs::write(path, crate::preview::svg(&pattern.project, &pattern.plan))
            .map_err(|e| failed(format!("cannot write {path}: {e}")))?;
        let _ = writeln!(out, "wrote {path}");
    }
    if let Some(path) = &gcode {
        let exported =
            lasercraft_gcode::export_plan(&pattern.project, pattern.plan, &machine, &options)
                .map_err(failed)?;
        std::fs::write(path, &exported.gcode)
            .map_err(|e| failed(format!("cannot write {path}: {e}")))?;
        let _ = writeln!(out, "wrote {path}: {} lines", exported.report.lines);
    }
    Ok(out)
}
