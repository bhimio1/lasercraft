//! The controller against the simulator (`docs/spec/device.md`). Every test
//! that runs a job checks the laser fired only while it ran (safety rule 14).

use crate::sim::{Lit, SimConfig};
use crate::{Bench, Event, MachineState, Outcome, Request, StreamKind};

use super::*;

/// A square with travel, burn at M4, a fill-like stretch and the end.
const JOB: &str = "(test job)\nG90 G21\nM5 S0\nG0 X10 Y10\nM4 S0\nG1 X60 S500 F1200\nG1 Y60\nG1 X10\nG1 Y10\nG1 X12 S0\nG1 Y12 S300\nM5 S0\n";
const JOB_BURN: f64 = 202.0;

fn start(bench: &mut Bench, kind: StreamKind, gcode: &str) -> f64 {
    let at = bench.now();
    bench.request(Request::Stream {
        kind,
        gcode: gcode.to_owned(),
    });
    at
}

fn finished(bench: &mut Bench, seconds: f64) -> (Outcome, f64) {
    let from = bench.events.len();
    assert!(
        bench.run_until(seconds, |e| matches!(e, Event::Finished { .. })),
        "not finished: {:?}",
        bench.events_since(from)
    );
    let outcome = bench
        .events
        .iter()
        .rev()
        .find_map(|e| match e {
            Event::Finished { outcome, .. } => Some(outcome.clone()),
            _ => None,
        })
        .unwrap();
    (outcome, bench.now())
}

/// Every lit stretch lies within `[from, to]`.
fn lit_only_between(log: &[Lit], from: f64, to: f64) {
    for lit in log {
        assert!(
            lit.start >= from - 1e-9 && lit.end <= to + 1e-9,
            "laser on at {lit:?} outside {from}..{to}"
        );
    }
}

fn rejected(bench: &Bench, from: usize) -> Option<String> {
    bench.events_since(from).iter().find_map(|e| match e {
        Event::Rejected(r) => Some(r.clone()),
        _ => None,
    })
}

#[test]
fn handshake_reads_version_buffer_and_settings() {
    let bench = Bench::ready(SimConfig::default());
    let info = bench
        .events
        .iter()
        .find_map(|e| match e {
            Event::Ready(info) => Some(info.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(info.version.as_deref(), Some("1.1h"));
    assert_eq!(info.rx_buffer, 128);
    assert_eq!(info.laser_mode, Some(true));
    assert_eq!(info.s_max, Some(1000.0));
    assert_eq!(info.max_travel, [Some(400.0), Some(400.0)]);
    assert_eq!(info.homing, Some(true));
    let connected = bench
        .events
        .iter()
        .position(|e| matches!(e, Event::Connected { .. }));
    let ready = bench
        .events
        .iter()
        .position(|e| matches!(e, Event::Ready(_)));
    assert!(connected < ready && connected.is_some());
    assert!(bench.sim.laser_log().is_empty());
}

#[test]
fn a_job_streams_with_character_counting_and_completes() {
    let mut bench = Bench::ready(SimConfig::default());
    let from = bench.events.len();
    let t0 = start(&mut bench, StreamKind::Job, JOB);
    let most = bench.controller.outstanding();
    while bench.controller.is_streaming() && bench.now() < t0 + 30.0 {
        bench.step();
    }
    let (outcome, t1) = (
        bench.events_since(from).iter().find_map(|e| match e {
            Event::Finished { outcome, .. } => Some(outcome.clone()),
            _ => None,
        }),
        bench.now(),
    );
    assert_eq!(outcome, Some(Outcome::Completed));
    assert!(most > 1, "several lines were in flight ({most})");
    assert_eq!(bench.sim.overflows(), 0);
    assert!(
        (bench.sim.lit_length() - JOB_BURN).abs() < 0.01,
        "{}",
        bench.sim.lit_length()
    );
    lit_only_between(&bench.sim.laser_log(), t0, t1);
    let progress: Vec<(usize, usize)> = bench
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Progress {
                acknowledged,
                total,
                ..
            } => Some((*acknowledged, *total)),
            _ => None,
        })
        .collect();
    assert_eq!(progress.first(), Some(&(0, 11)));
    assert_eq!(progress.last(), Some(&(11, 11)));
}

#[test]
fn send_response_keeps_one_line_in_flight() {
    let mut bench = Bench::new(
        SimConfig::default(),
        Settings {
            protocol: Protocol::SendResponse,
            ..Settings::default()
        },
    );
    assert!(bench.run_until(10.0, |e| matches!(e, Event::Ready(_))));
    bench.run_for(0.3);
    start(&mut bench, StreamKind::Job, JOB);
    while bench.controller.is_streaming() {
        bench.step();
        assert!(bench.controller.outstanding() <= 1);
    }
    assert!((bench.sim.lit_length() - JOB_BURN).abs() < 0.01);
}

#[test]
fn pause_and_resume_with_m4() {
    let mut bench = Bench::ready(SimConfig::default());
    let t0 = start(&mut bench, StreamKind::Job, JOB);
    bench.run_for(1.5);
    assert!(bench.sim.is_lit());
    bench.request(Request::Pause);
    bench.run_for(1.0);
    assert_eq!(bench.sim.state(), MachineState::Hold);
    assert!(!bench.sim.is_lit(), "M4 is dark when stopped");
    let held = bench.sim.position();
    bench.run_for(1.0);
    assert_eq!(bench.sim.position(), held);
    bench.request(Request::Resume);
    let (outcome, t1) = finished(&mut bench, 30.0);
    assert_eq!(outcome, Outcome::Completed);
    assert!((bench.sim.lit_length() - JOB_BURN).abs() < 0.01);
    lit_only_between(&bench.sim.laser_log(), t0, t1);
}

#[test]
fn pausing_an_m3_job_also_stops_the_spindle() {
    let job = JOB.replace("M4", "M3");
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, &job);
    bench.run_for(1.5);
    assert!(bench.sim.is_lit());
    bench.request(Request::Pause);
    bench.run_for(0.5);
    assert_eq!(bench.sim.state(), MachineState::Hold);
    assert!(!bench.sim.is_lit(), "spindle stop sent after the hold");
    // Lit only while decelerating: at most hold time plus a poll.
    let last = *bench.sim.laser_log().last().unwrap();
    assert!(last.end - last.start < 5.0);
    bench.request(Request::Resume);
    let (outcome, _) = finished(&mut bench, 30.0);
    assert_eq!(outcome, Outcome::Completed);
    assert!((bench.sim.lit_length() - JOB_BURN).abs() < 0.01);
}

#[test]
fn stop_holds_then_resets_and_keeps_the_position() {
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, JOB);
    bench.run_for(1.5);
    let from = bench.events.len();
    bench.request(Request::Stop);
    let stopped_at = bench.now();
    assert!(matches!(
        bench.events_since(from).first(),
        Some(Event::Finished {
            outcome: Outcome::Stopped,
            ..
        })
    ));
    bench.run_for(2.0);
    assert!(
        !bench
            .events_since(from)
            .iter()
            .any(|e| matches!(e, Event::Alarm { .. })),
        "a reset after the hold raises no alarm"
    );
    assert_eq!(bench.sim.state(), MachineState::Idle);
    assert!(!bench.sim.is_lit());
    let log = bench.sim.laser_log();
    lit_only_between(&log, 0.0, stopped_at + 0.3);
    let lit = bench.sim.lit_length();
    bench.run_for(5.0);
    assert_eq!(bench.sim.lit_length(), lit, "nothing more runs");
    assert!(!bench.controller.is_streaming());
    // The machine is usable again.
    let from = bench.events.len();
    start(&mut bench, StreamKind::Job, JOB);
    assert_eq!(rejected(&bench, from), None);
}

#[test]
fn stop_when_idle_only_resets() {
    let mut bench = Bench::ready(SimConfig::default());
    let from = bench.events.len();
    bench.request(Request::Stop);
    bench.run_for(0.5);
    assert_eq!(bench.sim.state(), MachineState::Idle, "no stray feed hold");
    assert!(
        !bench
            .events_since(from)
            .iter()
            .any(|e| matches!(e, Event::Alarm { .. }))
    );
}

#[test]
fn an_error_stops_the_job_and_flushes_grbl() {
    let job = JOB.replace("G1 Y60\n", "G1 Y60\nG2 X1 Y1 I1\n");
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, &job);
    let (outcome, t1) = finished(&mut bench, 10.0);
    let Outcome::Failed(reason) = outcome else {
        panic!("{outcome:?}");
    };
    assert!(
        reason.contains("line 7 `G2 X1 Y1 I1`: error 20"),
        "{reason}"
    );
    bench.run_for(5.0);
    assert!(!bench.sim.is_lit());
    lit_only_between(&bench.sim.laser_log(), 0.0, t1 + 1.1);
    // The burn stopped well short of the whole job.
    assert!(bench.sim.lit_length() < JOB_BURN - 50.0);
    assert_eq!(bench.sim.state(), MachineState::Idle);
}

#[test]
fn an_alarm_stops_the_job() {
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, JOB);
    bench.run_for(1.5);
    bench.sim.raise_alarm(1);
    let alarm_at = bench.now();
    let (outcome, _) = finished(&mut bench, 1.0);
    assert!(
        matches!(&outcome, Outcome::Failed(r) if r.contains("alarm 1")),
        "{outcome:?}"
    );
    assert!(bench.events.iter().any(|e| matches!(
        e,
        Event::Alarm { code: Some(1), message } if message.contains("limit switch")
    )));
    bench.run_for(3.0);
    lit_only_between(&bench.sim.laser_log(), 0.0, alarm_at);
    assert_eq!(bench.sim.state(), MachineState::Alarm);
    // Moving is refused until the alarm is cleared.
    let from = bench.events.len();
    bench.request(Request::Jog {
        x: 1.0,
        y: 0.0,
        feed: 100.0,
        relative: true,
    });
    assert!(rejected(&bench, from).unwrap().contains("not idle (Alarm)"));
    bench.request(Request::Unlock);
    bench.run_for(0.5);
    assert_eq!(bench.controller.state(), Some(MachineState::Idle));
}

#[test]
fn a_stall_stops_the_job() {
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, JOB);
    bench.run_for(1.0);
    bench.sim.set_silent(true);
    let silent_at = bench.now();
    let (outcome, t) = finished(&mut bench, 5.0);
    assert_eq!(
        outcome,
        Outcome::Failed("the controller stopped answering".into())
    );
    assert!(t - silent_at < 2.5, "{}", t - silent_at);
    assert!(bench.events.iter().any(|e| matches!(e, Event::Stalled)));
    bench.run_for(2.0);
    assert!(
        !bench.sim.is_lit(),
        "hold and reset still reached the machine"
    );
}

#[test]
fn unplugging_stops_the_job() {
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, JOB);
    bench.run_for(1.0);
    bench.sim.disconnect();
    let (outcome, _) = finished(&mut bench, 1.0);
    assert!(matches!(outcome, Outcome::Failed(r) if r.contains("disconnected")));
    assert!(
        bench
            .events
            .iter()
            .any(|e| matches!(e, Event::Disconnected { .. }))
    );
    assert!(bench.controller.is_closed());
}

#[test]
fn jobs_need_laser_mode_frames_do_not() {
    let mut bench = Bench::ready(SimConfig {
        laser_mode: false,
        ..SimConfig::default()
    });
    let from = bench.events.len();
    start(&mut bench, StreamKind::Job, JOB);
    assert!(rejected(&bench, from).unwrap().contains("$32=0"));
    let frame = "G90 G21\nM5 S0\nG0 X10 Y10\nG1 X20 F3000\nG1 Y20\nG1 X10\nG1 Y10\nM5 S0\n";
    let t0 = start(&mut bench, StreamKind::Frame, frame);
    let (outcome, _) = finished(&mut bench, 10.0);
    assert_eq!(outcome, Outcome::Completed);
    assert!(t0 >= 0.0);
    assert!(bench.sim.laser_log().is_empty());
}

#[test]
fn moving_requests_are_refused_while_busy_or_not_idle() {
    let mut bench = Bench::ready(SimConfig::default());
    start(&mut bench, StreamKind::Job, JOB);
    for request in [
        Request::Jog {
            x: 1.0,
            y: 1.0,
            feed: 100.0,
            relative: true,
        },
        Request::Home,
        Request::Unlock,
        Request::Console("G0 X0".into()),
        Request::Stream {
            kind: StreamKind::Frame,
            gcode: "G0 X1".into(),
        },
    ] {
        let from = bench.events.len();
        bench.request(request.clone());
        assert!(
            rejected(&bench, from).unwrap().contains("while a job"),
            "{request:?}"
        );
    }
    let mut locked = Bench::ready(SimConfig {
        start_locked: true,
        ..SimConfig::default()
    });
    let from = locked.events.len();
    start(&mut locked, StreamKind::Job, JOB);
    assert!(
        rejected(&locked, from)
            .unwrap()
            .contains("not idle (Alarm)")
    );
    let from = locked.events.len();
    start(&mut locked, StreamKind::Job, "(only a comment)\n");
    assert!(rejected(&locked, from).is_some());
}

#[test]
fn homing_suspends_stall_detection() {
    let mut bench = Bench::ready(SimConfig {
        homing_time: 5.0,
        start_locked: true,
        ..SimConfig::default()
    });
    bench.request(Request::Home);
    let from = bench.events.len();
    bench.run_for(6.0);
    assert!(
        !bench
            .events_since(from)
            .iter()
            .any(|e| matches!(e, Event::Stalled)),
        "no status while homing is not a stall"
    );
    assert_eq!(bench.controller.state(), Some(MachineState::Idle));
    assert_eq!(bench.controller.position(), Some([0.0, 0.0, 0.0]));
}

#[test]
fn jogs_move_the_head_with_the_laser_off() {
    let mut bench = Bench::ready(SimConfig::default());
    for _ in 0..3 {
        bench.request(Request::Jog {
            x: 10.0,
            y: -0.0,
            feed: 3000.0,
            relative: true,
        });
    }
    bench.request(Request::Jog {
        x: 5.0,
        y: 7.5,
        feed: 3000.0,
        relative: false,
    });
    bench.run_for(3.0);
    assert_eq!(bench.sim.position(), [5.0, 7.5]);
    assert_eq!(bench.controller.position(), Some([5.0, 7.5, 0.0]));
    assert!(bench.sim.laser_log().is_empty());
    let from = bench.events.len();
    bench.request(Request::Jog {
        x: 1000.0,
        y: 0.0,
        feed: 3000.0,
        relative: true,
    });
    bench.run_for(0.2);
    assert!(rejected(&bench, from).unwrap().contains("error 15"));
}

#[test]
fn console_lines_and_realtime_queries() {
    let mut bench = Bench::ready(SimConfig::default());
    let from = bench.events.len();
    bench.request(Request::Console("$$".into()));
    bench.request(Request::Console("?".into()));
    bench.request(Request::Console("G4 P0.1".into()));
    bench.request(Request::Console("$Q".into()));
    bench.run_for(0.5);
    let lines: Vec<(bool, String, bool)> = bench
        .events_since(from)
        .iter()
        .filter_map(|e| match e {
            Event::Console {
                sent,
                text,
                routine,
            } => Some((*sent, text.clone(), *routine)),
            _ => None,
        })
        .collect();
    let shown: Vec<&(bool, String, bool)> = lines.iter().filter(|l| !l.2).collect();
    assert!(shown.iter().any(|l| l.0 && l.1 == "$$"));
    assert!(shown.iter().any(|l| !l.0 && l.1 == "$32=1"));
    assert!(
        shown.iter().any(|l| !l.0 && l.1.starts_with("<Idle")),
        "{lines:?}"
    );
    assert!(shown.iter().any(|l| !l.0 && l.1 == "error:3"));
    assert!(
        lines.iter().any(|l| l.2 && l.1.starts_with("<Idle")),
        "polls are routine"
    );
}

#[test]
fn nothing_fires_on_connect_jog_home_frame_or_idle() {
    let mut bench = Bench::ready(SimConfig::default());
    bench.request(Request::Home);
    bench.run_for(3.0);
    bench.request(Request::Jog {
        x: 50.0,
        y: 50.0,
        feed: 6000.0,
        relative: true,
    });
    bench.run_for(2.0);
    start(
        &mut bench,
        StreamKind::Frame,
        "G90 G21\nM5 S0\nG0 X60 Y60\nG1 X90 F3000\nG1 Y90\nG1 X60\nG1 Y60\nG0 X50 Y50\nM5 S0\n",
    );
    let (outcome, _) = finished(&mut bench, 10.0);
    assert_eq!(outcome, Outcome::Completed);
    bench.run_for(30.0);
    assert!(bench.sim.laser_log().is_empty());
    assert_eq!(bench.sim.position(), [50.0, 50.0]);
}

fn feed(c: &mut Controller, now: f64, lines: &[&str]) {
    for line in lines {
        c.received(line, now);
    }
}

#[test]
fn a_running_grbl_without_banner_is_probed() {
    let mut c = Controller::new("p", Settings::default(), 0.0);
    c.tick(1.0);
    assert!(c.take_output().is_empty());
    c.tick(3.0);
    assert_eq!(c.take_output(), b"?");
    feed(&mut c, 3.1, &["<Idle|MPos:1.000,2.000,0.000|FS:0,0>"]);
    assert!(
        c.take_events()
            .iter()
            .any(|e| matches!(e, Event::Connected { .. }))
    );
    assert_eq!(c.take_output(), b"$I\n");

    let mut silent = Controller::new("p", Settings::default(), 0.0);
    silent.tick(3.0);
    silent.tick(6.0);
    assert!(silent.is_closed());
    assert!(
        silent
            .take_events()
            .iter()
            .any(|e| matches!(e, Event::Disconnected { reason } if reason.contains("no reply")))
    );
}

#[test]
fn a_handshake_without_answers_still_finishes() {
    let mut c = Controller::new("p", Settings::default(), 0.0);
    feed(&mut c, 0.1, &["Grbl 1.1f ['$' for help]"]);
    c.tick(3.2);
    assert!(c.is_ready());
    let ready = c.take_events().into_iter().find_map(|e| match e {
        Event::Ready(info) => Some(info),
        _ => None,
    });
    let info = ready.unwrap();
    assert_eq!(info.laser_mode, None);
    assert_eq!(info.version.as_deref(), Some("1.1f"));
}

#[test]
fn surplus_answers_switch_to_send_response() {
    let mut c = Controller::new("p", Settings::default(), 0.0);
    feed(&mut c, 0.0, &["Grbl 1.1h ['$' for help]"]);
    c.tick(0.0);
    feed(&mut c, 0.0, &["ok", "$32=1", "ok", "<Idle|MPos:0,0,0>"]);
    assert!(c.is_ready());
    c.request(
        Request::Stream {
            kind: StreamKind::Job,
            gcode: JOB.to_owned(),
        },
        0.1,
    );
    assert!(c.outstanding() > 1);
    let sent = c.outstanding();
    for _ in 0..sent {
        feed(&mut c, 0.2, &["ok"]);
    }
    assert!(!c.is_ping_pong());
    let in_flight = c.outstanding();
    for _ in 0..=in_flight {
        feed(&mut c, 0.3, &["ok"]);
    }
    assert!(c.is_ping_pong());
    assert!(c.outstanding() <= 1);
}

#[test]
fn helpers() {
    assert_eq!(
        program_lines("(a)\n G1 X1 ; note\n\n(b) M5 (c)\n"),
        ["G1 X1", "M5"]
    );
    assert!(has_m3("M3 S0") && has_m3("G1 M03") && !has_m3("M30") && !has_m3("M4 S3"));
    assert!(needs_alone("$$") && needs_alone("$32=1") && needs_alone("G10 L20 P1 X0"));
    assert!(!needs_alone("$J=G91 X1 F100") && !needs_alone("G1 X1"));
    assert_eq!(
        jog_line(1.5, -0.0, 3000.0, true),
        "$J=G21 G91 X1.5 Y0 F3000"
    );
    assert_eq!(
        jog_line(10.0, 20.25, 600.0, false),
        "$J=G21 G90 X10 Y20.25 F600"
    );
}
