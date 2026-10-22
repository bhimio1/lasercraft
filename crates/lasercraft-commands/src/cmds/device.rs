//! `device.*`: run jobs on the laser (`docs/spec/laser.md`). These never
//! change the project and are not undoable; each returns a device request in
//! its outcome for the front end to deliver.

use lasercraft_device::{Request, StreamKind};
use lasercraft_gcode::{Anchor, FrameShape, Options, Placement, export, frame, machine_delta};
use lasercraft_geom::Point;
use lasercraft_plan::{Machine, duration};
use lasercraft_project::Project;

use super::export::{apply_option, key_value, number};
use crate::args::Args;
use crate::command::{ccmd, cmd};
use crate::{Command, CommandError, Context, Invocation, Outcome, job_settings};

/// Default jog and framing speed, mm/s.
pub const DEFAULT_SPEED: f64 = 50.0;

/// Highest jog and framing speed accepted, mm/s.
const MAX_SPEED: f64 = 1000.0;

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "device.connect",
            "device.connect <port> [baud]",
            "Connect to a laser on a serial port, or to `simulator`.",
            connect,
        ),
        cmd(
            "device.disconnect",
            "device.disconnect",
            "Stop anything running and disconnect.",
            |_, a| simple(a, Request::Disconnect, "disconnecting"),
        ),
        ccmd(
            "device.start",
            "device.start [from=absolute|current|user] [at=X,Y] [anchor=<name>] [s_max=<n>] [air=m7|m8] [finish=on|off] [max_speed=<mm/s>] [accel=<mm/s²>] [max_power=<%>]",
            "Run the job on the connected laser after the safety checks.",
            start,
        ),
        ccmd(
            "device.frame",
            "device.frame [outline] [from=absolute|current|user] [at=X,Y] [anchor=<name>] [speed=<mm/s>] [head=X,Y] [max_speed=<mm/s>] [accel=<mm/s²>]",
            "Trace the job's box (or its outline) with the laser off.",
            frame_job,
        ),
        ccmd(
            "device.test",
            "device.test <speed_power|interval> [json] [from=absolute|current|user] [at=X,Y] [anchor=<name>] [s_max=<n>] [air=m7|m8] [max_speed=<mm/s>] [accel=<mm/s²>] [max_power=<%>]",
            "Run a test pattern on the connected laser after the safety checks.",
            test_start,
        ),
        ccmd(
            "device.test_frame",
            "device.test_frame <speed_power|interval> [json] [outline] [from=absolute|current|user] [at=X,Y] [anchor=<name>] [speed=<mm/s>] [head=X,Y]",
            "Trace a test pattern's box (or outline) with the laser off.",
            test_frame,
        ),
        cmd(
            "device.pause",
            "device.pause",
            "Pause the running job (feed hold).",
            |_, a| simple(a, Request::Pause, "pausing"),
        ),
        cmd(
            "device.resume",
            "device.resume",
            "Resume a paused job.",
            |_, a| simple(a, Request::Resume, "resuming"),
        ),
        cmd(
            "device.stop",
            "device.stop",
            "Stop the job: feed hold, then soft reset.",
            |_, a| simple(a, Request::Stop, "stopping"),
        ),
        cmd(
            "device.jog",
            "device.jog <up|down|left|right> <mm> [speed=<mm/s>] [head=X,Y]",
            "Move the head with the laser off, as seen on screen.",
            jog,
        ),
        cmd(
            "device.move_to",
            "device.move_to <x> <y> [speed=<mm/s>]",
            "Move the head to a machine position with the laser off.",
            move_to,
        ),
        cmd(
            "device.home",
            "device.home",
            "Run the homing cycle ($H).",
            |_, a| simple(a, Request::Home, "homing"),
        ),
        cmd(
            "device.unlock",
            "device.unlock",
            "Clear an alarm lock ($X); the position may be wrong.",
            |_, a| simple(a, Request::Unlock, "unlocking"),
        ),
        cmd(
            "device.send",
            "device.send <line>",
            "Send a line to the controller as typed.",
            send,
        ),
    ]
}

/// What the front end knows about the machine, for [`with_context`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DeviceContext {
    /// The head's last reported position, machine mm.
    pub head: Option<Point>,
    /// The stored user origin, machine mm.
    pub user_origin: Option<Point>,
    /// The controller's `$30`.
    pub s_max: Option<f64>,
}

fn has_key(invocation: &Invocation, key: &str) -> bool {
    invocation
        .args
        .iter()
        .any(|a| a.split_once('=').is_some_and(|(k, _)| k == key))
}

fn arg_value<'a>(invocation: &'a Invocation, key: &str) -> Option<&'a str> {
    invocation
        .args
        .iter()
        .find_map(|a| a.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| v))
}

fn point_arg(key: &str, p: Point) -> String {
    format!("{key}={},{}", fmt(p.x), fmt(p.y))
}

/// Adds what the user did not type and the front end knows: `at=` for the
/// current position or user origin, `head=` for framing and jogging, and
/// `s_max=` from the controller (`docs/spec/laser.md`). Other invocations
/// come back unchanged.
#[must_use]
pub fn with_context(invocation: &Invocation, ctx: &DeviceContext) -> Invocation {
    let mut out = invocation.clone();
    let name = invocation.name.as_str();
    let job = matches!(
        name,
        "device.start" | "device.frame" | "device.test" | "device.test_frame"
    );
    if job && !has_key(invocation, "at") {
        let at = match arg_value(invocation, "from") {
            Some("current") => ctx.head,
            Some("user") => ctx.user_origin,
            _ => None,
        };
        if let Some(at) = at {
            out.args.push(point_arg("at", at));
        }
    }
    if matches!(
        name,
        "device.start" | "device.test" | "device.camera_pattern"
    ) && !has_key(invocation, "s_max")
        && let Some(s_max) = ctx.s_max
    {
        out.args.push(format!("s_max={}", fmt(s_max)));
    }
    if matches!(
        name,
        "device.frame"
            | "device.test_frame"
            | "device.jog"
            | "device.camera_pattern_frame"
            | "registration.set_target"
            | "camera.align"
            | "camera.align_find"
    ) && !has_key(invocation, "head")
        && let Some(head) = ctx.head
    {
        out.args.push(point_arg("head", head));
    }
    out
}

fn simple(args: &Args<'_>, request: Request, message: &str) -> Result<Outcome, CommandError> {
    args.expect(0, 0)?;
    Ok(Outcome::device(message, request))
}

fn connect(_: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 2)?;
    let port = args.str(0)?.to_owned();
    let baud = if args.len() > 1 {
        args.parse::<u32>(1, "baud rate")?
    } else {
        115_200
    };
    if baud == 0 {
        return Err(args.invalid("baud rate must be positive"));
    }
    Ok(Outcome::device(
        format!("connecting to {port} at {baud} baud"),
        Request::Connect { port, baud },
    ))
}

/// `X,Y` in machine mm.
pub(super) fn xy(args: &Args<'_>, key: &str, value: &str) -> Result<Point, CommandError> {
    let (x, y) = value
        .split_once(',')
        .ok_or_else(|| args.invalid(format!("{key} needs X,Y, not `{value}`")))?;
    Ok(Point::new(number(args, key, x)?, number(args, key, y)?))
}

pub(super) fn speed(args: &Args<'_>, value: &str) -> Result<f64, CommandError> {
    let v = number(args, "speed", value)?;
    if v > 0.0 && v <= MAX_SPEED {
        Ok(v)
    } else {
        Err(args.invalid(format!(
            "speed must be above 0 and at most {MAX_SPEED} mm/s"
        )))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum StartFrom {
    Absolute,
    Current,
    User,
}

/// Placement, machine, export options and the remaining `key=value`s.
struct JobArgs {
    machine: Machine,
    options: Options,
    from: StartFrom,
    speed: f64,
    head: Option<Point>,
    outline: bool,
}

fn job_args(
    cx: &Context<'_>,
    args: &Args<'_>,
    framing: bool,
    from: usize,
) -> Result<JobArgs, CommandError> {
    let (machine, options, _) = job_settings(cx.project, cx.workshop);
    let mut job = JobArgs {
        machine,
        options,
        from: StartFrom::Absolute,
        speed: DEFAULT_SPEED,
        head: None,
        outline: false,
    };
    let mut at = None;
    let mut anchor = Anchor::default();
    for i in from..args.len() {
        let raw = args.str(i)?;
        if framing && i == from && raw == "outline" {
            job.outline = true;
            continue;
        }
        let (key, value) = key_value(args, raw)?;
        match key {
            "from" => {
                job.from = match value {
                    "absolute" => StartFrom::Absolute,
                    "current" => StartFrom::Current,
                    "user" => StartFrom::User,
                    _ => return Err(args.invalid("from must be absolute, current or user")),
                };
            }
            "at" => at = Some(xy(args, key, value)?),
            "anchor" => {
                anchor = value
                    .parse::<Anchor>()
                    .map_err(|e| args.invalid(e.to_string()))?;
            }
            "speed" if framing => job.speed = speed(args, value)?,
            "head" if framing => job.head = Some(xy(args, key, value)?),
            _ if apply_option(args, key, value, &mut job.machine, &mut job.options)? => {}
            _ => return Err(args.invalid(format!("unknown option `{key}`"))),
        }
    }
    job.options.placement = match (job.from, at) {
        (StartFrom::Absolute, _) => Placement::Absolute,
        (StartFrom::Current, Some(at)) => Placement::Relative {
            at,
            anchor,
            return_to_start: true,
        },
        (StartFrom::User, Some(at)) => Placement::Relative {
            at,
            anchor,
            return_to_start: false,
        },
        (StartFrom::Current, None) => {
            return Err(args.invalid(
                "starting from the current position needs the head position (at=X,Y): connect and wait for a status report",
            ));
        }
        (StartFrom::User, None) => {
            return Err(args
                .invalid("starting from the user origin needs it (at=X,Y): set an origin first"));
        }
    };
    Ok(job)
}

fn placement_text(options: &Options) -> String {
    match options.placement {
        Placement::Absolute => "absolute coordinates".to_owned(),
        Placement::Relative { at, anchor, .. } => {
            format!("{anchor} at X{} Y{}", fmt(at.x), fmt(at.y))
        }
    }
}

fn fmt(v: f64) -> String {
    lasercraft_plan::num(v)
}

/// Print registration places the job by its targets
/// (`docs/spec/registration.md`): any other placement is replaced.
fn registered(cx: &Context<'_>, job: &mut JobArgs) -> String {
    if cx.project.registration_transform().is_none() {
        return String::new();
    }
    let moved = job.options.placement != Placement::Absolute;
    job.options.placement = Placement::Absolute;
    if moved {
        "\nPrint registration places the job by its targets.".to_owned()
    } else {
        String::new()
    }
}

fn start(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let mut job = job_args(cx, args, false, 0)?;
    let note = registered(cx, &mut job);
    let out =
        export(cx.project, &job.machine, &job.options).map_err(|e| args.invalid(e.to_string()))?;
    let mut warnings = String::new();
    for w in &out.plan.warnings {
        warnings.push_str("\nWarning: ");
        warnings.push_str(&w.message);
    }
    Ok(Outcome::device(
        format!(
            "starting the job ({}): {} lines, burn {:.1} mm, estimated {}{note}{warnings}",
            placement_text(&job.options),
            out.report.lines,
            out.report.burn_length,
            duration(out.plan.stats.total_time)
        ),
        Request::Stream {
            kind: StreamKind::Job,
            gcode: out.gcode,
        },
    ))
}

fn frame_job(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let mut job = job_args(cx, args, true, 0)?;
    let note = registered(cx, &mut job);
    let shape = if job.outline {
        FrameShape::Outline
    } else {
        FrameShape::Box
    };
    let speed = job.speed.min(job.machine.max_speed);
    let gcode = frame(
        cx.project,
        &job.machine,
        &job.options,
        shape,
        speed,
        job.head,
    )
    .map_err(|e| args.invalid(e.to_string()))?;
    Ok(Outcome::device(
        format!(
            "framing the job's {} ({}) at {} mm/s, laser off{note}",
            if job.outline { "outline" } else { "box" },
            placement_text(&job.options),
            fmt(speed)
        ),
        Request::Stream {
            kind: StreamKind::Frame,
            gcode,
        },
    ))
}

/// The kind and settings a `device.test*` command names, and where its job
/// arguments start.
fn test_parts<'a>(args: &Args<'a>) -> Result<(&'a str, Option<&'a str>, usize), CommandError> {
    if args.len() == 0 {
        return Err(args.usage_error());
    }
    let kind = args.str(0)?;
    if args.len() > 1 && args.str(1)?.trim_start().starts_with('{') {
        Ok((kind, Some(args.str(1)?), 2))
    } else {
        Ok((kind, None, 1))
    }
}

fn test_start(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let (kind, json, from) = test_parts(args)?;
    let job = job_args(cx, args, false, from)?;
    let pattern = super::test::pattern(cx, args, kind, json, &job.machine)?;
    let out =
        lasercraft_gcode::export_plan(&pattern.project, pattern.plan, &job.machine, &job.options)
            .map_err(|e| args.invalid(e.to_string()))?;
    Ok(Outcome::device(
        format!(
            "starting the {} ({}): {} lines, estimated {}",
            pattern.project.name().to_lowercase(),
            placement_text(&job.options),
            out.report.lines,
            duration(out.plan.stats.total_time)
        ),
        Request::Stream {
            kind: StreamKind::Job,
            gcode: out.gcode,
        },
    ))
}

fn test_frame(cx: &mut Context<'_>, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let (kind, json, from) = test_parts(args)?;
    let job = job_args(cx, args, true, from)?;
    let pattern = super::test::pattern(cx, args, kind, json, &job.machine)?;
    let shape = if job.outline {
        FrameShape::Outline
    } else {
        FrameShape::Box
    };
    let speed = job.speed.min(job.machine.max_speed);
    let gcode = lasercraft_gcode::frame_plan(
        &pattern.project,
        pattern.plan,
        &job.machine,
        &job.options,
        shape,
        speed,
        job.head,
    )
    .map_err(|e| args.invalid(e.to_string()))?;
    Ok(Outcome::device(
        format!(
            "framing the {} at {} mm/s, laser off",
            pattern.project.name().to_lowercase(),
            fmt(speed)
        ),
        Request::Stream {
            kind: StreamKind::Frame,
            gcode,
        },
    ))
}

fn jog(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 4)?;
    let direction = args.str(0)?;
    let distance = args.f64(1, "distance")?;
    if !(0.01..=1000.0).contains(&distance) {
        return Err(args.invalid("distance must be between 0.01 and 1000 mm"));
    }
    let mut feed = DEFAULT_SPEED;
    let mut head = None;
    for i in 2..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        match key {
            "speed" => feed = speed(args, value)?,
            "head" => head = Some(xy(args, key, value)?),
            _ => return Err(args.invalid(format!("unknown option `{key}`"))),
        }
    }
    let screen = match direction {
        "up" => Point::new(0.0, -distance),
        "down" => Point::new(0.0, distance),
        "left" => Point::new(-distance, 0.0),
        "right" => Point::new(distance, 0.0),
        _ => return Err(args.invalid("direction must be up, down, left or right")),
    };
    let area = project.work_area();
    let mut d = machine_delta(area, screen);
    if let Some(h) = head {
        // Stop at the bed's edge (docs/spec/laser.md, "Jogging and homing").
        let to = Point::new(
            (h.x + d.x).clamp(0.0, area.width_mm),
            (h.y + d.y).clamp(0.0, area.height_mm),
        );
        d = to - h;
        if d.length() < 1e-9 {
            return Err(args.invalid(format!("the head is already at the {direction} edge")));
        }
    }
    Ok(Outcome::device(
        format!("jogging {direction} {} mm", fmt(d.length())),
        Request::Jog {
            x: d.x,
            y: d.y,
            feed: feed * 60.0,
            relative: true,
        },
    ))
}

fn move_to(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 3)?;
    let to = args.point(0)?;
    let mut feed = DEFAULT_SPEED;
    if args.len() > 2 {
        let (key, value) = key_value(args, args.str(2)?)?;
        if key != "speed" {
            return Err(args.invalid(format!("unknown option `{key}`")));
        }
        feed = speed(args, value)?;
    }
    let area = project.work_area();
    if !(0.0..=area.width_mm).contains(&to.x) || !(0.0..=area.height_mm).contains(&to.y) {
        return Err(args.invalid(format!(
            "X{} Y{} is outside the work area {} × {}",
            fmt(to.x),
            fmt(to.y),
            fmt(area.width_mm),
            fmt(area.height_mm)
        )));
    }
    Ok(Outcome::device(
        format!("moving to X{} Y{}", fmt(to.x), fmt(to.y)),
        Request::Jog {
            x: to.x,
            y: to.y,
            feed: feed * 60.0,
            relative: false,
        },
    ))
}

fn send(_: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    let line = args.rest(0)?;
    if line.trim().is_empty() {
        return Err(args.usage_error());
    }
    Ok(Outcome::device(
        format!("sending `{line}`"),
        Request::Console(line),
    ))
}

#[cfg(test)]
mod tests {
    use lasercraft_device::sim::SimConfig;
    use lasercraft_device::{Bench, Event, Outcome as Ended};

    use crate::{Registry, Session};

    use super::*;

    fn session(lines: &[&str]) -> Session {
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        for line in lines {
            s.run_line(line).unwrap();
        }
        s
    }

    fn request(s: &mut Session, line: &str) -> Request {
        s.run_line(line)
            .unwrap_or_else(|e| panic!("{line}: {e}"))
            .device
            .unwrap_or_else(|| panic!("{line}: no request"))
    }

    fn error(s: &mut Session, line: &str) -> String {
        s.run_line(line).unwrap_err().to_string()
    }

    #[test]
    fn simple_requests() {
        let mut s = session(&[]);
        for (line, want) in [
            ("device.disconnect", Request::Disconnect),
            ("device.pause", Request::Pause),
            ("device.resume", Request::Resume),
            ("device.stop", Request::Stop),
            ("device.home", Request::Home),
            ("device.unlock", Request::Unlock),
            ("device.send \"$$\"", Request::Console("$$".into())),
            (
                "device.connect simulator",
                Request::Connect {
                    port: "simulator".into(),
                    baud: 115_200,
                },
            ),
            (
                "device.connect /dev/ttyUSB0 250000",
                Request::Connect {
                    port: "/dev/ttyUSB0".into(),
                    baud: 250_000,
                },
            ),
        ] {
            assert_eq!(request(&mut s, line), want, "{line}");
        }
        assert!(s.run_line("device.pause now").is_err());
        assert!(s.run_line("device.connect").is_err());
        assert!(s.run_line("device.connect x 0").is_err());
        assert!(s.run_line("device.send").is_err());
        assert_eq!(s.undo_depth(), 0, "device commands are not undoable");
        assert_eq!(s.project(), &Project::default(), "and change nothing");
    }

    #[test]
    fn jogs_follow_the_screen_and_stop_at_the_edge() {
        let mut s = session(&[]);
        // Front-left origin: up on screen is +Y on the machine.
        assert_eq!(
            request(&mut s, "device.jog up 10"),
            Request::Jog {
                x: 0.0,
                y: 10.0,
                feed: 3000.0,
                relative: true
            }
        );
        assert_eq!(
            request(&mut s, "device.jog left 5 speed=20"),
            Request::Jog {
                x: -5.0,
                y: 0.0,
                feed: 1200.0,
                relative: true
            }
        );
        assert_eq!(
            request(&mut s, "device.jog left 50 head=20,30"),
            Request::Jog {
                x: -20.0,
                y: 0.0,
                feed: 3000.0,
                relative: true
            }
        );
        assert!(error(&mut s, "device.jog down 5 head=20,0").contains("already at the down edge"));
        s.run_line("project.set_origin rear-right").unwrap();
        assert_eq!(
            request(&mut s, "device.jog up 10"),
            Request::Jog {
                x: 0.0,
                y: -10.0,
                feed: 3000.0,
                relative: true
            }
        );
        for bad in [
            "device.jog sideways 1",
            "device.jog up 0",
            "device.jog up 5000",
            "device.jog up 1 speed=0",
            "device.jog up 1 speed=5000",
            "device.jog up 1 fast=1",
            "device.jog up 1 head=1",
        ] {
            assert!(s.run_line(bad).is_err(), "{bad}");
        }
        assert_eq!(
            request(&mut s, "device.move_to 10 20"),
            Request::Jog {
                x: 10.0,
                y: 20.0,
                feed: 3000.0,
                relative: false
            }
        );
        assert!(error(&mut s, "device.move_to 500 0").contains("outside the work area"));
    }

    fn stream(request: Request) -> (StreamKind, String) {
        match request {
            Request::Stream { kind, gcode } => (kind, gcode),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn start_builds_a_checked_placed_job() {
        let mut s = session(&["shape.add_rect 10 10 20 10"]);
        let (kind, gcode) = stream(request(&mut s, "device.start"));
        assert_eq!(kind, StreamKind::Job);
        assert!(gcode.contains("M4") && gcode.ends_with("M5 S0\n"));
        let out = s
            .run_line("device.start from=current at=100,100 anchor=center")
            .unwrap();
        assert!(
            out.message.contains("center at X100 Y100"),
            "{}",
            out.message
        );
        let (_, gcode) = stream(out.device.unwrap());
        assert!(gcode.ends_with("G0 X100 Y100\nM5 S0\n"), "{gcode}");
        assert!(error(&mut s, "device.start from=current").contains("head position"));
        assert!(error(&mut s, "device.start from=user").contains("set an origin"));
        assert!(error(&mut s, "device.start from=user at=395,395").contains("does not fit"));
        assert!(error(&mut s, "device.start from=nowhere").contains("absolute, current or user"));
        assert!(error(&mut s, "device.start anchor=middle").contains("job origin"));
        assert!(error(&mut s, "device.start speed=5").contains("unknown option"));
        assert!(error(&mut s, "device.start at=5").contains("X,Y"));
        assert!(error(&mut session(&[]), "device.start").contains("nothing to export"));
    }

    #[test]
    fn frame_builds_a_laser_off_program() {
        let mut s = session(&["shape.add_rect 10 10 20 10"]);
        let out = s
            .run_line("device.frame outline speed=20 head=5,5")
            .unwrap();
        assert!(out.message.contains("outline"), "{}", out.message);
        let (kind, gcode) = stream(out.device.unwrap());
        assert_eq!(kind, StreamKind::Frame);
        assert!(!gcode.contains("M4") && !gcode.contains("M3"));
        assert!(
            gcode.contains("F1200") && gcode.ends_with("G0 X5 Y5\nM5 S0\n"),
            "{gcode}"
        );
        assert!(s.run_line("device.frame speed=0").is_err());
        assert!(s.run_line("device.frame s_max=2000 bogus=1").is_err());
        assert!(
            s.run_line("device.frame from=user at=1,1 anchor=top-right")
                .is_err()
        );
    }

    #[test]
    fn context_fills_in_what_the_front_end_knows() {
        let ctx = DeviceContext {
            head: Some(Point::new(12.5, 30.0)),
            user_origin: Some(Point::new(100.0, 50.0)),
            s_max: Some(255.0),
        };
        let fill = |line: &str| with_context(&Invocation::parse(line).unwrap(), &ctx).to_string();
        assert_eq!(
            fill("device.start from=current"),
            "device.start from=current at=12.5,30 s_max=255"
        );
        assert_eq!(
            fill("device.start from=user s_max=1000"),
            "device.start from=user s_max=1000 at=100,50"
        );
        assert_eq!(fill("device.start"), "device.start s_max=255");
        assert_eq!(
            fill("device.frame outline from=current at=1,2"),
            "device.frame outline from=current at=1,2 head=12.5,30"
        );
        assert_eq!(fill("device.jog up 5"), "device.jog up 5 head=12.5,30");
        assert_eq!(fill("device.home"), "device.home");
        assert_eq!(fill("shape.add_rect 1 1 2 2"), "shape.add_rect 1 1 2 2");
        let none = DeviceContext::default();
        assert_eq!(
            with_context(
                &Invocation::parse("device.start from=current").unwrap(),
                &none
            )
            .to_string(),
            "device.start from=current"
        );
    }

    /// The whole path: command → request → controller → simulated GRBL.
    #[test]
    fn a_started_job_runs_on_the_simulator() {
        let mut s = session(&["shape.add_rect 10 10 20 10", "layer.set 0 speed 20"]);
        let job = request(
            &mut s,
            "device.start from=current at=50,50 anchor=bottom-left",
        );
        let mut bench = Bench::ready(SimConfig::default());
        bench.request(job);
        assert!(bench.run_until(30.0, |e| matches!(e, Event::Finished { .. })));
        assert!(bench.events.iter().any(|e| matches!(
            e,
            Event::Finished {
                outcome: Ended::Completed,
                ..
            }
        )));
        assert!(
            (bench.sim.lit_length() - 60.0).abs() < 0.01,
            "{}",
            bench.sim.lit_length()
        );
        assert_eq!(bench.sim.position(), [50.0, 50.0], "back at the start");
        assert_eq!(bench.sim.overflows(), 0);
    }
}
