//! `lasercraft-cli export`: G-code through the `project.export_gcode`
//! command (`docs/spec/cli.md`, `docs/spec/gcode.md`).

use std::path::Path;

use lasercraft_commands::Invocation;
use lasercraft_project::Project;

use crate::{CliError, failed, session_with, usage};

/// Flags with a value, and the command option each becomes.
const VALUE_FLAGS: [(&str, &str); 5] = [
    ("--s-max", "s_max"),
    ("--air", "air"),
    ("--max-speed", "max_speed"),
    ("--accel", "accel"),
    ("--max-power", "max_power"),
];

/// Runs `export <project.lcproj> <out.gcode> [flags]`.
pub(crate) fn export(args: &[String]) -> Result<String, CliError> {
    let [file, out, flags @ ..] = args else {
        return Err(usage("export needs a project and an output file"));
    };
    let mut command = vec![out.clone()];
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        if flag == "--finish" {
            command.push("finish=on".to_owned());
            continue;
        }
        let key = VALUE_FLAGS
            .iter()
            .find(|(f, _)| f == flag)
            .map(|(_, k)| k)
            .ok_or_else(|| usage(&format!("unknown flag `{flag}`")))?;
        let value = rest
            .next()
            .ok_or_else(|| usage(&format!("{flag} needs a value")))?;
        command.push(format!("{key}={value}"));
    }
    let project = Project::load(Path::new(file)).map_err(failed)?;
    let mut session = session_with(project);
    let outcome = session
        .run(&Invocation::new("project.export_gcode", command))
        .map_err(failed)?;
    Ok(format!("{}\n", outcome.message))
}
