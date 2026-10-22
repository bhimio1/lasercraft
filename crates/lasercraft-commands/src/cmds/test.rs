//! `test.*`: test patterns and their presets (`docs/spec/test-patterns.md`).
//! Patterns never change the project; presets are user data.

use std::path::Path;

use lasercraft_gcode::export_plan;
use lasercraft_plan::patterns::{Pattern, TestSettings};
use lasercraft_plan::{Machine, duration};
use serde_json::Value;

use super::export::{apply_option, key_value, write_atomically};
use super::profile::{saved, ws};
use crate::args::Args;
use crate::command::ccmd;
use crate::{Command, CommandError, Context, Outcome, job_settings};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        ccmd(
            "test.export",
            "test.export <speed_power|interval> <path> [json] [s_max=<n>] [air=m7|m8] [max_speed=<mm/s>] [accel=<mm/s²>] [max_power=<%>]",
            "Write a test pattern as GRBL G-code after the safety checks.",
            export,
        ),
        ccmd(
            "test.preset_save",
            "test.preset_save <name> <speed_power|interval> <json>",
            "Store test settings as a preset.",
            preset_save,
        ),
        ccmd(
            "test.preset_delete",
            "test.preset_delete <name> <speed_power|interval>",
            "Remove a test preset.",
            preset_delete,
        ),
        ccmd(
            "test.preset_list",
            "test.preset_list",
            "List the built-in and saved test presets.",
            preset_list,
        ),
        ccmd(
            "test.preset_import",
            "test.preset_import <path>",
            "Add the presets in a .lctests file.",
            preset_import,
        ),
        ccmd(
            "test.preset_export",
            "test.preset_export <path>",
            "Write the saved test presets to a .lctests file.",
            preset_export,
        ),
    ]
}

fn json(args: &Args<'_>, text: &str) -> Result<Value, CommandError> {
    serde_json::from_str(text).map_err(|e| args.invalid(format!("test settings are not JSON: {e}")))
}

/// Builds the pattern `kind` on the project's work area for `machine`, and
/// remembers its settings as the last used for `kind`.
pub(super) fn pattern(
    cx: &mut Context<'_>,
    args: &Args<'_>,
    kind: &str,
    settings: Option<&str>,
    machine: &Machine,
) -> Result<Pattern, CommandError> {
    let value = settings.map(|t| json(args, t)).transpose()?;
    let parsed = TestSettings::parse(kind, value.as_ref()).map_err(|e| args.invalid(e.0))?;
    let pattern = lasercraft_plan::patterns::build(&parsed, cx.project.work_area(), machine)
        .map_err(|e| args.invalid(e.0))?;
    cx.workshop.remember_settings(kind, parsed.to_value());
    Ok(pattern)
}

fn export(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    if args.len() < 2 {
        return Err(args.usage_error());
    }
    let (kind, path) = (args.str(0)?, args.str(1)?);
    let mut from = 2;
    let settings = if args.len() > 2 && args.str(2)?.trim_start().starts_with('{') {
        from = 3;
        Some(args.str(2)?)
    } else {
        None
    };
    let (mut machine, mut options, _) = job_settings(cx.project, cx.workshop);
    for i in from..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        if key == "finish" || !apply_option(args, key, value, &mut machine, &mut options)? {
            return Err(args.invalid(format!("unknown option `{key}`")));
        }
    }
    let pattern = pattern(cx, args, kind, settings, &machine)?;
    let warnings: Vec<String> = pattern
        .plan
        .warnings
        .iter()
        .map(|w| w.message.clone())
        .collect();
    let (w, h) = pattern.size;
    let out = export_plan(&pattern.project, pattern.plan, &machine, &options)
        .map_err(|e| args.invalid(e.to_string()))?;
    write_atomically(Path::new(path), &out.gcode)
        .map_err(|e| args.invalid(format!("cannot write {path}: {e}")))?;
    let mut message = format!(
        "exported {path}: {} test, {:.1} × {:.1} mm, {} lines, estimated {}",
        kind.replace('_', " and "),
        w,
        h,
        out.report.lines,
        duration(out.plan.stats.total_time)
    );
    for w in warnings {
        message.push_str("\nWarning: ");
        message.push_str(&w);
    }
    Ok(saved(cx, message))
}

fn preset_save(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(3, 3)?;
    let (name, kind) = (args.str(0)?, args.str(1)?);
    let value = json(args, args.str(2)?)?;
    // Only settings a pattern accepts are stored.
    TestSettings::parse(kind, Some(&value)).map_err(|e| args.invalid(e.0))?;
    cx.workshop
        .save_preset(name, kind, value)
        .map_err(ws(args))?;
    Ok(saved(cx, format!("saved preset \"{}\"", name.trim())))
}

fn preset_delete(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 2)?;
    let name = args.str(0)?;
    cx.workshop
        .delete_preset(name, args.str(1)?)
        .map_err(ws(args))?;
    Ok(saved(cx, format!("removed preset \"{}\"", name.trim())))
}

fn preset_list(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    let lines: Vec<String> = cx
        .workshop
        .presets()
        .into_iter()
        .map(|(p, builtin)| {
            format!(
                "{} ({}{})",
                p.name,
                p.kind,
                if builtin { ", built-in" } else { "" }
            )
        })
        .collect();
    Ok(Outcome::done(lines.join("\n")))
}

fn preset_import(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let n = cx
        .workshop
        .import_presets(Path::new(args.str(0)?))
        .map_err(ws(args))?;
    Ok(saved(cx, format!("imported {n} preset(s)")))
}

fn preset_export(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let path = args.str(0)?;
    let n = cx
        .workshop
        .export_presets(Path::new(path))
        .map_err(ws(args))?;
    Ok(Outcome::done(format!("wrote {n} preset(s) to {path}")))
}

#[cfg(test)]
mod tests {
    use lasercraft_device::{Request, StreamKind};

    use crate::{DeviceContext, Invocation, Registry, Session, with_context};

    fn session() -> Session {
        Session::new(
            lasercraft_project::Project::default(),
            Registry::with_builtin(),
        )
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("lasercraft-test-cmds-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn export_writes_verified_gcode_and_leaves_the_project_alone() {
        let mut s = session();
        let before = s.project().clone();
        let path = temp("grid.gcode");
        let line = Invocation::new(
            "test.export",
            [
                "speed_power",
                &path.display().to_string(),
                r#"{"rows": {"count": 3}, "columns": {"count": 3}}"#,
                "max_power=50",
            ],
        );
        let message = s.run(&line).unwrap().message;
        assert!(message.contains("speed and power test"), "{message}");
        assert!(
            message.contains("Warning: the device maximum is 50%"),
            "{message}"
        );
        let gcode = std::fs::read_to_string(&path).unwrap();
        assert!(gcode.contains("M4") && gcode.contains("G1"));
        assert_eq!(s.project(), &before);
        assert!(s.run_line("edit.undo").is_err(), "nothing recorded");
        // The settings are remembered for the next time.
        assert_eq!(
            s.workshop().last_settings("speed_power").unwrap()["rows"]["count"],
            3
        );
        // A bed too small refuses the pattern.
        s.run_line("project.set_work_area 30 30").unwrap();
        assert!(s.run(&line).is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn device_test_streams_the_pattern_with_the_job_placement() {
        let mut s = session();
        let ctx = DeviceContext {
            head: Some(lasercraft_geom::Point::new(100.0, 100.0)),
            user_origin: None,
            s_max: Some(255.0),
        };
        let run = |s: &mut Session, line: &str| {
            let inv = with_context(&Invocation::parse(line).unwrap(), &ctx);
            s.run(&inv)
        };
        let out = run(
            &mut s,
            r#"device.test interval "{\"patches\": 3}" from=current"#,
        )
        .unwrap();
        match out.device {
            Some(Request::Stream { kind, gcode }) => {
                assert_eq!(kind, StreamKind::Job);
                assert!(gcode.contains("S"), "power is in the program");
            }
            other => panic!("{other:?}"),
        }
        let frame = run(&mut s, "device.test_frame speed_power outline").unwrap();
        assert!(matches!(
            frame.device,
            Some(Request::Stream {
                kind: StreamKind::Frame,
                ..
            })
        ));
        assert!(run(&mut s, "device.test focus").is_err());
        assert!(run(&mut s, r#"device.test interval "{\"patches\": 1}""#).is_err());
    }

    #[test]
    fn presets_are_saved_checked_listed_and_exchanged() {
        let mut s = session();
        s.run_line(r#"test.preset_save Mine speed_power "{\"spacing\": 2}""#)
            .unwrap();
        assert!(
            s.run_line(r#"test.preset_save Bad speed_power "{\"spacing\": 99}""#)
                .is_err()
        );
        assert!(
            s.run_line(r#"test.preset_save Bad speed_power "{\"nope\": 1}""#)
                .is_err()
        );
        let list = s.run_line("test.preset_list").unwrap().message;
        assert!(
            list.contains("Diode engraving (speed_power, built-in)")
                && list.contains("Mine (speed_power)"),
            "{list}"
        );
        let path = temp("mine.lctests");
        s.run(&Invocation::new(
            "test.preset_export",
            [path.display().to_string()],
        ))
        .unwrap();
        s.run_line("test.preset_delete Mine speed_power").unwrap();
        assert!(
            s.run_line("test.preset_delete \"Diode cutting\" speed_power")
                .is_err()
        );
        s.run(&Invocation::new(
            "test.preset_import",
            [path.display().to_string()],
        ))
        .unwrap();
        assert!(
            s.run_line("test.preset_list")
                .unwrap()
                .message
                .contains("Mine")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn built_in_presets_build_valid_patterns() {
        for (preset, _) in lasercraft_workshop::Workshop::in_memory().presets() {
            let settings = lasercraft_plan::patterns::TestSettings::parse(
                &preset.kind,
                Some(&preset.settings),
            )
            .unwrap_or_else(|e| panic!("{}: {e}", preset.name));
            lasercraft_plan::patterns::build(
                &settings,
                &lasercraft_project::WorkArea::default(),
                &lasercraft_plan::Machine::default(),
            )
            .unwrap();
        }
    }
}
