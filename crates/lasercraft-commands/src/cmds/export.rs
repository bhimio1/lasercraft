//! `project.export_gcode` (`docs/spec/gcode.md`).

use std::path::Path;

use lasercraft_gcode::{Air, Options, export};
use lasercraft_plan::{Machine, duration};
use lasercraft_project::Project;

use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![cmd(
        "project.export_gcode",
        "project.export_gcode <path> [s_max=<n>] [air=m7|m8] [finish=on|off] [max_speed=<mm/s>] [accel=<mm/s²>] [max_power=<%>]",
        "Write the job as GRBL G-code after the work-area and safety checks.",
        export_gcode,
    )]
}

/// Splits `key=value`.
pub(super) fn key_value<'a>(
    args: &Args<'_>,
    raw: &'a str,
) -> Result<(&'a str, &'a str), CommandError> {
    raw.split_once('=')
        .ok_or_else(|| args.invalid(format!("`{raw}` is not key=value")))
}

/// A finite number from an option value.
pub(super) fn number(args: &Args<'_>, key: &str, value: &str) -> Result<f64, CommandError> {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| args.invalid(format!("{key} needs a number, not `{value}`")))
}

/// Applies one machine or export option; `false` if `key` is not one.
pub(super) fn apply_option(
    args: &Args<'_>,
    key: &str,
    value: &str,
    machine: &mut Machine,
    options: &mut Options,
) -> Result<bool, CommandError> {
    match key {
        "s_max" => options.s_max = number(args, key, value)?,
        "air" => {
            options.air = value
                .parse::<Air>()
                .map_err(|e| args.invalid(e.to_string()))?;
        }
        "finish" => {
            options.finish_at_origin = match value {
                "on" => true,
                "off" => false,
                _ => return Err(args.invalid("finish must be on or off")),
            };
        }
        "max_speed" => machine.max_speed = number(args, key, value)?,
        "accel" => machine.acceleration = number(args, key, value)?,
        "max_power" => machine.max_power = number(args, key, value)?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// Machine and export options from `key=value` arguments.
fn options(args: &Args<'_>, from: usize) -> Result<(Machine, Options), CommandError> {
    let mut machine = Machine::default();
    let mut options = Options::default();
    for i in from..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        if !apply_option(args, key, value, &mut machine, &mut options)? {
            return Err(args.invalid(format!("unknown option `{key}`")));
        }
    }
    Ok((machine, options))
}

/// Writes `text` next to `path` and renames it into place.
fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

fn export_gcode(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let path = args.str(0)?;
    let (machine, options) = options(args, 1)?;
    let out = export(project, &machine, &options).map_err(|e| args.invalid(e.to_string()))?;
    write_atomically(Path::new(path), &out.gcode)
        .map_err(|e| args.invalid(format!("cannot write {path}: {e}")))?;
    Ok(Outcome::done(format!(
        "exported {path}: {} lines, burn {:.1} mm, max S {}, estimated {}",
        out.report.lines,
        out.report.burn_length,
        out.report.max_s,
        duration(out.plan.stats.total_time)
    )))
}

#[cfg(test)]
mod tests {
    use crate::{Registry, Session};

    use super::*;

    fn temp(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lasercraft-export-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap();
        }
        s
    }

    #[test]
    fn writes_a_verified_program_and_is_not_an_edit() {
        let dir = temp("ok");
        let path = dir.join("job.gcode");
        let mut s = session(&["shape.add_rect 10 10 20 20 @1"]);
        let line = format!(
            "project.export_gcode \"{}\" s_max=255 air=m7 finish=on",
            path.display()
        );
        let out = s.run_line(&line).unwrap();
        assert!(out.message.contains("burn 80.0 mm"), "{}", out.message);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("S51") && text.ends_with("G0 X0 Y0\n"),
            "{text}"
        );
        assert_eq!(s.undo_depth(), 1, "export is not undoable");
        assert!(!dir.join("job.gcode.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refusals_write_nothing() {
        let dir = temp("refuse");
        let path = dir.join("job.gcode");
        let quoted = format!("\"{}\"", path.display());
        let mut s = session(&["shape.add_rect 390 390 20 20"]);
        let err = s
            .run_line(&format!("project.export_gcode {quoted}"))
            .unwrap_err();
        assert!(err.to_string().contains("does not fit"), "{err}");
        let mut empty = session(&[]);
        assert!(
            empty
                .run_line(&format!("project.export_gcode {quoted}"))
                .unwrap_err()
                .to_string()
                .contains("nothing to export")
        );
        for bad in [
            "s_max=0",
            "air=m9",
            "finish=maybe",
            "bogus=1",
            "s_max",
            "max_speed=x",
        ] {
            let mut ok = session(&["shape.add_rect 10 10 5 5"]);
            assert!(
                ok.run_line(&format!("project.export_gcode {quoted} {bad}"))
                    .is_err(),
                "{bad}"
            );
        }
        assert!(!path.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
