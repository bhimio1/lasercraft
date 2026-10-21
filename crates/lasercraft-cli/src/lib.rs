//! Headless LaserCraft: run commands and scripts against `.lcproj` files
//! (`docs/spec/cli.md`). Every change goes through the command registry, just
//! as in the app.

use std::fmt::Write as _;
use std::path::Path;

use lasercraft_commands::{CommandError, Invocation, Registry, SESSION_COMMANDS, Session};
use lasercraft_project::Project;

mod export;
mod preview;
mod simulate;

/// Usage text printed by `help` and on bad arguments.
pub const USAGE: &str = "\
usage:
  lasercraft-cli new <out.lcproj> [<width_mm> <height_mm>]
  lasercraft-cli run <project.lcproj> <command> [<command> ...]
  lasercraft-cli script <script.lcs> [<project.lcproj>]
  lasercraft-cli show <project.lcproj> [--json]
  lasercraft-cli preview <project.lcproj> [--svg <out.svg>] [--moves]
                 [--max-speed <mm/s>] [--accel <mm/s²>] [--max-power <%>]
  lasercraft-cli export <project.lcproj> <out.gcode> [--s-max <n>]
                 [--air m7|m8] [--finish] [--max-speed <mm/s>]
                 [--accel <mm/s²>] [--max-power <%>]
  lasercraft-cli simulate <project.lcproj> [--locked] [--user-origin X,Y]
                 <command> [<command> ...]
  lasercraft-cli commands
  lasercraft-cli help";

/// Why a CLI invocation failed.
#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    /// The arguments did not match [`USAGE`]. Exit code 2.
    Usage(String),
    /// A command or file operation failed. Exit code 1.
    Failed(String),
}

impl CliError {
    /// The process exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::Failed(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(reason) => write!(f, "{reason}\n{USAGE}"),
            Self::Failed(reason) => f.write_str(reason),
        }
    }
}

fn failed(err: impl std::fmt::Display) -> CliError {
    CliError::Failed(err.to_string())
}

fn usage(reason: &str) -> CliError {
    CliError::Usage(reason.to_owned())
}

fn session_with(project: Project) -> Session {
    Session::new(project, Registry::with_builtin())
}

/// Runs the CLI with `args` (without the program name) and returns what to
/// print on standard output.
///
/// # Errors
///
/// Returns [`CliError`] describing what went wrong.
pub fn run(args: &[String]) -> Result<String, CliError> {
    let (sub, rest) = args
        .split_first()
        .ok_or_else(|| usage("missing subcommand"))?;
    match sub.as_str() {
        "new" => new(rest),
        "run" => run_commands(rest),
        "script" => script(rest),
        "show" => show(rest),
        "preview" => preview::preview(rest),
        "export" => export::export(rest),
        "simulate" => simulate::simulate(rest),
        "commands" if rest.is_empty() => Ok(commands()),
        "help" | "--help" | "-h" => Ok(format!("{USAGE}\n")),
        other => Err(usage(&format!("unknown subcommand `{other}`"))),
    }
}

fn new(args: &[String]) -> Result<String, CliError> {
    let (out, size) = args
        .split_first()
        .ok_or_else(|| usage("new needs a file"))?;
    if !(size.is_empty() || size.len() == 2) {
        return Err(usage("new takes a width and a height, or neither"));
    }
    let mut session = session_with(Project::default());
    let mut line = Invocation::new("project.new", size.iter().cloned());
    session.run(&line).map_err(failed)?;
    line = Invocation::new("project.save", [out.clone()]);
    session.run(&line).map_err(failed)?;
    Ok(format!("created {out}\n"))
}

fn run_commands(args: &[String]) -> Result<String, CliError> {
    let (file, lines) = args
        .split_first()
        .ok_or_else(|| usage("run needs a file"))?;
    if lines.is_empty() {
        return Err(usage("run needs at least one command"));
    }
    let path = Path::new(file);
    let mut session = session_with(Project::load(path).map_err(failed)?);
    let mut out = String::new();
    for line in lines {
        let outcome = session
            .run_line(line)
            .map_err(|e| failed(format!("`{line}`: {e}")))?;
        let _ = writeln!(out, "{}", outcome.message);
    }
    // Written only when every command succeeded.
    session.project().save(path).map_err(failed)?;
    Ok(out)
}

fn script(args: &[String]) -> Result<String, CliError> {
    let (file, project) = match args {
        [file] => (file, None),
        [file, project] => (file, Some(project)),
        _ => return Err(usage("script needs a script file and an optional project")),
    };
    let text =
        std::fs::read_to_string(file).map_err(|e| failed(format!("cannot read {file}: {e}")))?;
    let project = match project {
        Some(p) => Project::load(Path::new(p)).map_err(failed)?,
        None => Project::default(),
    };
    let mut session = session_with(project);
    let mut out = String::new();
    for (number, line) in text.lines().enumerate() {
        match session.run_line(line) {
            Ok(outcome) => {
                let _ = writeln!(out, "{}", outcome.message);
            }
            Err(CommandError::EmptyLine) => {}
            Err(err) => {
                return Err(failed(format!(
                    "{out}{file}:{}: `{}`: {err}",
                    number + 1,
                    line.trim()
                )));
            }
        }
    }
    let _ = writeln!(out, "{}", summary(session.project()));
    Ok(out)
}

fn show(args: &[String]) -> Result<String, CliError> {
    let (file, json) = match args {
        [file] => (file, false),
        [file, flag] if flag == "--json" => (file, true),
        _ => return Err(usage("show needs a file and an optional --json")),
    };
    let project = Project::load(Path::new(file)).map_err(failed)?;
    Ok(if json {
        project.to_json()
    } else {
        format!("{}\n", summary(&project))
    })
}

/// A length for listings: at most three decimals, no trailing zeros.
fn mm(value: f64) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// A layer's settings in one line, listing fill settings only for fill layers.
fn layer_settings(l: &lasercraft_project::Layer) -> String {
    let mut text = format!(
        "{} {} mm/s, power {}–{}%, {} pass(es)",
        l.mode,
        mm(l.speed),
        mm(l.min_power),
        mm(l.max_power),
        l.passes
    );
    if l.mode == lasercraft_project::Mode::Fill {
        let _ = write!(
            text,
            ", interval {} mm at {}°{}{}{}",
            mm(l.interval),
            mm(l.angle),
            if l.bidirectional {
                ", both ways"
            } else {
                ", one way"
            },
            if l.overscan { ", overscan" } else { "" },
            if l.crosshatch { ", crosshatch" } else { "" }
        );
    }
    if l.mode == lasercraft_project::Mode::Line {
        if l.kerf != 0.0 {
            let _ = write!(text, ", kerf {} mm", mm(l.kerf));
        }
        if l.tabs > 0 {
            let _ = write!(text, ", {} tab(s) of {} mm", l.tabs, mm(l.tab_width));
        }
    }
    for (on, word) in [
        (!l.output, "output off"),
        (!l.show, "hidden"),
        (l.air, "air"),
    ] {
        if on {
            let _ = write!(text, ", {word}");
        }
    }
    text
}

/// Description of a project: one header line, then one line per shape in
/// drawing order with its bounds.
#[must_use]
pub fn summary(project: &Project) -> String {
    let area = project.work_area();
    let mut out = format!(
        "project \"{}\": work area {} × {} mm, origin {}, {} shape(s)",
        project.name(),
        area.width_mm,
        area.height_mm,
        area.origin,
        project.shapes().len()
    );
    for color in project.used_layers() {
        if let Ok(l) = project.layer(color) {
            let _ = write!(
                out,
                "\n  layer {color:>2} \"{}\": {}",
                l.name,
                layer_settings(l)
            );
        }
    }
    for shape in project.shapes() {
        let b = shape.bounds();
        let _ = write!(
            out,
            "\n  {:>4} @{:<2} {:<8} ({}, {}) – ({}, {})",
            shape.id(),
            shape.layer(),
            shape.geometry().kind(),
            mm(b.min.x),
            mm(b.min.y),
            mm(b.max.x),
            mm(b.max.y)
        );
        // M3: group chain, outermost first.
        if !shape.groups().is_empty() {
            let groups: Vec<String> = shape.groups().iter().map(ToString::to_string).collect();
            let _ = write!(out, " group {}", groups.join("/"));
        }
    }
    out
}

fn commands() -> String {
    let mut out = String::new();
    for command in Registry::with_builtin().iter() {
        let _ = writeln!(out, "{}\n    {}", command.usage(), command.summary());
    }
    for (_, usage, summary) in SESSION_COMMANDS {
        let _ = writeln!(out, "{usage}\n    {summary}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn bad_arguments_are_usage_errors() {
        for bad in [
            &[][..],
            &["frobnicate"],
            &["new"],
            &["new", "x.lcproj", "100"],
            &["run", "x.lcproj"],
            &["show"],
            &["show", "x", "--yaml"],
            &["script"],
            &["commands", "extra"],
            &["preview"],
            &["preview", "x.lcproj", "--frobnicate"],
            &["preview", "x.lcproj", "--accel"],
            &["preview", "x.lcproj", "--max-speed", "fast"],
            &["preview", "x.lcproj", "--svg"],
            &["preview", "a.lcproj", "b.lcproj"],
        ] {
            let err = run(&args(bad)).unwrap_err();
            assert_eq!(err.exit_code(), 2, "{bad:?}");
        }
    }

    #[test]
    fn commands_lists_registry_and_session_commands() {
        let text = run(&args(&["commands"])).unwrap();
        assert!(text.contains("project.set_work_area <width_mm> <height_mm>"));
        assert!(text.contains("edit.undo"));
        assert!(text.contains("edit.batch"));
    }
}
