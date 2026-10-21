//! `lasercraft-cli simulate`: device commands against the simulated GRBL
//! through the real streaming controller, on simulated time
//! (`docs/spec/cli.md`, `docs/spec/laser.md`).

use std::fmt::Write as _;
use std::path::Path;

use lasercraft_commands::{DeviceContext, Invocation, with_context};
use lasercraft_device::sim::SimConfig;
use lasercraft_device::{Bench, Event, MachineState, Request};
use lasercraft_geom::Point;
use lasercraft_project::Project;

use crate::{CliError, failed, mm, session_with, usage};

/// The longest a stream may run, simulated seconds.
const STREAM_LIMIT: f64 = 3600.0;

/// The longest anything else may take, simulated seconds.
const SETTLE_LIMIT: f64 = 120.0;

fn point(flag: &str, value: &str) -> Result<Point, CliError> {
    value
        .split_once(',')
        .and_then(|(x, y)| Some(Point::new(x.parse().ok()?, y.parse().ok()?)))
        .ok_or_else(|| usage(&format!("{flag} needs X,Y")))
}

/// Steps until the request has played out.
fn settle(bench: &mut Bench, request: &Request) {
    if matches!(request, Request::Stream { .. }) {
        if bench.controller.is_streaming() {
            bench.run_until(STREAM_LIMIT, |e| matches!(e, Event::Finished { .. }));
            bench.run_for(0.5);
        }
        return;
    }
    bench.run_for(0.3);
    let end = bench.now() + SETTLE_LIMIT;
    while bench.now() < end {
        let resting = matches!(
            bench.controller.state(),
            Some(MachineState::Idle | MachineState::Alarm | MachineState::Hold)
        );
        if resting && bench.controller.outstanding() == 0 && !bench.controller.is_streaming() {
            return;
        }
        bench.step();
    }
}

fn describe(event: &Event, out: &mut String) {
    let _ = match event {
        Event::Finished { kind, outcome } => writeln!(out, "{kind} {outcome}"),
        Event::Rejected(reason) => writeln!(out, "refused: {reason}"),
        Event::Alarm { code, message } => match code {
            Some(code) => writeln!(out, "alarm {code}: {message}"),
            None => writeln!(out, "alarm: {message}"),
        },
        Event::Stalled => writeln!(out, "the controller stopped answering"),
        Event::Disconnected { reason } => writeln!(out, "disconnected: {reason}"),
        Event::Console {
            sent: false,
            text,
            routine: false,
        } => writeln!(out, "< {text}"),
        _ => Ok(()),
    };
}

/// Runs `simulate <project.lcproj> [--locked] [--user-origin X,Y] <command>…`.
pub(crate) fn simulate(args: &[String]) -> Result<String, CliError> {
    let [file, rest @ ..] = args else {
        return Err(usage("simulate needs a project and commands"));
    };
    let mut config = SimConfig::default();
    let mut ctx = DeviceContext::default();
    let mut lines = Vec::new();
    let mut iter = rest.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--locked" => config.start_locked = true,
            "--user-origin" => {
                let value = iter
                    .next()
                    .ok_or_else(|| usage("--user-origin needs X,Y"))?;
                ctx.user_origin = Some(point(arg, value)?);
            }
            flag if flag.starts_with("--") => {
                return Err(usage(&format!("unknown flag `{flag}`")));
            }
            line => lines.push(line.to_owned()),
        }
    }
    if lines.is_empty() {
        return Err(usage("simulate needs at least one command"));
    }
    let project = Project::load(Path::new(file)).map_err(failed)?;
    config.width = project.work_area().width_mm;
    config.height = project.work_area().height_mm;
    let mut session = session_with(project);
    let mut bench = Bench::ready(config);
    let mut out = String::new();
    for line in &lines {
        let parsed = Invocation::parse(line).map_err(|e| failed(format!("`{line}`: {e}")))?;
        ctx.head = bench.controller.position().map(|p| Point::new(p[0], p[1]));
        ctx.s_max = bench.controller.info().s_max;
        let invocation = with_context(&parsed, &ctx);
        let outcome = session
            .run(&invocation)
            .map_err(|e| failed(format!("{out}`{line}`: {e}")))?;
        let _ = writeln!(out, "> {line}\n{}", outcome.message);
        match outcome.device {
            Some(Request::Connect { .. } | Request::Disconnect) => {
                let _ = writeln!(out, "(the simulator stays connected)");
            }
            Some(request) => {
                let from = bench.events.len();
                bench.request(request.clone());
                settle(&mut bench, &request);
                for event in bench.events_since(from) {
                    describe(event, &mut out);
                }
            }
            None => {}
        }
    }
    let log = bench.sim.laser_log();
    let [x, y] = bench.sim.position();
    let _ = writeln!(
        out,
        "simulator: laser on for {} mm in {} stretch(es), head at X{} Y{}, {}, {} s",
        mm(bench.sim.lit_length()),
        log.len(),
        mm(x),
        mm(y),
        bench.sim.state(),
        mm((bench.now() * 10.0).round() / 10.0)
    );
    if bench.sim.overflows() > 0 {
        return Err(failed(format!(
            "{out}the receive buffer overflowed {} time(s): a streaming bug",
            bench.sim.overflows()
        )));
    }
    Ok(out)
}
