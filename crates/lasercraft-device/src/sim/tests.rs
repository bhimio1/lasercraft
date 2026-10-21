//! The simulator against GRBL's documented behavior.

use super::*;

fn sim() -> Sim {
    let mut s = Sim::new(SimConfig::default());
    assert!(s.read().starts_with("Grbl 1.1h ['$' for help]"));
    s
}

/// Sends `text` and lets `seconds` pass; returns what came back.
fn run(s: &mut Sim, text: &str, seconds: f64) -> String {
    s.write(text.as_bytes());
    s.advance(seconds);
    s.read()
}

#[test]
fn queries_answer_like_grbl() {
    let mut s = sim();
    let out = run(&mut s, "$I\n", 0.0);
    assert_eq!(out, "[VER:1.1h.20190825:]\r\n[OPT:V,15,128]\r\nok\r\n");
    let out = run(&mut s, "$$\n", 0.0);
    for line in ["$22=1", "$30=1000", "$32=1", "$130=400", "$131=400", "ok"] {
        assert!(out.contains(line), "{line}: {out}");
    }
    let out = run(&mut s, "?", 0.0);
    assert_eq!(
        out,
        "<Idle|MPos:0.000,0.000,0.000|FS:0,0|WCO:0.000,0.000,0.000>\r\n"
    );
    assert!(!run(&mut s, "?", 0.0).contains("WCO"));
    assert_eq!(run(&mut s, "$32=0\n", 0.0), "ok\r\n");
    assert_eq!(s.setting(32), Some("0"));
    assert_eq!(run(&mut s, "$Q\n", 0.0), "error:3\r\n");
    assert_eq!(run(&mut s, "\n", 0.0), "ok\r\n");
}

#[test]
fn a_program_burns_only_on_lit_g1_moves() {
    let mut s = sim();
    let program = "G90 G21\nM5 S0\nG0 X10 Y10\nM4 S0\nG1 X20 S500 F600\nG1 X25 S0\nG1 Y20 S800\nM5 S0\nG0 X0 Y0\n";
    let out = run(&mut s, program, 10.0);
    assert_eq!(out.matches("ok").count(), 9, "{out}");
    assert_eq!(s.state(), MachineState::Idle);
    assert_eq!(s.position(), [0.0, 0.0]);
    let log = s.laser_log();
    assert_eq!(log.len(), 2, "{log:?}");
    assert!((log[0].length - 10.0).abs() < 1e-9 && log[0].s == 500.0);
    assert!((log[1].length - 10.0).abs() < 1e-9 && log[1].s == 800.0);
    // 10 mm at 10 mm/s is one second.
    assert!((log[0].end - log[0].start - 1.0).abs() < 0.01, "{log:?}");
    assert!(!s.is_lit());
}

#[test]
fn feed_hold_with_m4_turns_the_laser_off_m3_keeps_it_unless_spindle_stop() {
    for (mode, lit_while_held) in [("M4", false), ("M3", true)] {
        let mut s = sim();
        run(&mut s, &format!("G90 G21\n{mode} S100\nG1 X50 F600\n"), 1.0);
        assert!(s.is_lit());
        run(&mut s, "!", 0.2);
        assert_eq!(s.state(), MachineState::Hold);
        let held_at = s.position();
        assert!(run(&mut s, "?", 0.0).starts_with("<Hold:0"));
        assert_eq!(s.is_lit(), lit_while_held, "{mode}");
        s.write(&[SPINDLE_STOP]);
        s.advance(0.1);
        assert!(!s.is_lit(), "{mode}: spindle stop");
        assert_eq!(s.position(), held_at);
        run(&mut s, "~", 0.1);
        assert!(s.is_lit(), "{mode}: resumed");
        run(&mut s, "", 10.0);
        assert_eq!(s.position(), [50.0, 0.0]);
        assert!((s.lit_length() - 50.0).abs() < 1e-6, "{mode}");
    }
}

#[test]
fn spindle_stop_is_ignored_while_moving() {
    let mut s = sim();
    run(&mut s, "M3 S100\nG1 X50 F600\n", 0.5);
    s.write(&[SPINDLE_STOP]);
    s.advance(0.1);
    assert!(s.is_lit());
}

#[test]
fn reset_in_motion_alarms_reset_at_rest_keeps_position() {
    let mut s = sim();
    run(&mut s, "G1 X50 F600\n", 1.0);
    let out = run(&mut s, "\u{18}", 0.0);
    assert!(out.starts_with("ALARM:3\r\n"), "{out}");
    assert!(
        out.contains("Grbl 1.1h") && out.contains("'$X' to unlock"),
        "{out}"
    );
    assert_eq!(s.state(), MachineState::Alarm);
    assert_eq!(run(&mut s, "G0 X1\n", 0.1), "error:9\r\n");
    assert!(run(&mut s, "$X\n", 0.0).contains("Unlocked"));
    let at = s.position();
    run(&mut s, "!", 0.0);
    let out = run(&mut s, "\u{18}", 0.0);
    assert!(!out.contains("ALARM"), "{out}");
    assert_eq!(s.state(), MachineState::Idle);
    assert_eq!(s.position(), at);
}

#[test]
fn reset_after_a_completed_hold_keeps_position() {
    let mut s = sim();
    run(&mut s, "G1 X50 F600\n", 1.0);
    run(&mut s, "!", 0.2);
    let out = run(&mut s, "\u{18}", 0.0);
    assert!(!out.contains("ALARM"), "{out}");
    assert_eq!(s.state(), MachineState::Idle);
}

#[test]
fn the_receive_buffer_overflows_past_128_bytes() {
    let mut s = sim();
    let line = format!("G1 X1 F100 ({})\n", "x".repeat(40));
    for _ in 0..20 {
        s.write(line.as_bytes());
    }
    assert!(s.overflows() > 0);
    let mut fresh = sim();
    fresh.write(&[b' '; 128]);
    assert_eq!(fresh.overflows(), 0);
}

#[test]
fn a_full_planner_leaves_lines_in_the_buffer() {
    let mut s = sim();
    let batch = |s: &mut Sim, from: usize, to: usize| {
        for i in from..to {
            s.write(format!("G1 X{i} F60\n").as_bytes());
        }
        s.read().matches("ok").count()
    };
    assert_eq!(batch(&mut s, 1, 11), 10);
    // 15 planner blocks: five more fit, three lines wait in the buffer.
    assert_eq!(batch(&mut s, 11, 19), 5);
    assert_eq!(s.overflows(), 0);
    s.advance(0.01);
    assert_eq!(s.read().matches("ok").count(), 1, "a block started running");
    s.advance(2.0);
    assert_eq!(s.read().matches("ok").count(), 2);
}

#[test]
fn errors_for_bad_lines() {
    let mut s = sim();
    for (line, want) in [
        ("G2 X1 Y1 I1\n", "error:20"),
        ("G1 X1\n", "error:22"),
        ("G1 X1.2.3 F10\n", "error:2"),
        ("$J=G91 X1\n", "error:16"),
        ("$J=G91 X1 F100 S5\n", "error:16"),
        ("$J=G90 X500 F100\n", "error:15"),
        (&format!("({})\n", "x".repeat(90)), "error:11"),
    ] {
        assert_eq!(run(&mut s, line, 0.0), format!("{want}\r\n"), "{line}");
    }
    run(&mut s, "G1 X10 F600\n", 0.1);
    assert_eq!(run(&mut s, "$H\n", 0.0), "error:8\r\n");
    assert_eq!(run(&mut s, "$$\n", 0.0), "error:8\r\n");
}

#[test]
fn soft_limits_alarm_on_g_code_past_the_travel() {
    let mut s = sim();
    let out = run(&mut s, "G0 X401\n", 0.1);
    assert_eq!(out, "ALARM:2\r\n");
    assert_eq!(s.state(), MachineState::Alarm);
}

#[test]
fn jogs_move_without_light_and_a_hold_cancels_them() {
    let mut s = sim();
    run(&mut s, "M4 S1000\n", 0.0);
    let out = run(&mut s, "$J=G21 G91 X10 Y5 F6000\n", 0.05);
    assert_eq!(out, "ok\r\n");
    assert_eq!(s.state(), MachineState::Jog);
    s.advance(1.0);
    assert_eq!(s.position(), [10.0, 5.0]);
    assert_eq!(s.state(), MachineState::Idle);
    run(&mut s, "$J=G21 G90 X300 Y5 F600\n", 0.5);
    run(&mut s, "!", 0.1);
    assert_eq!(s.state(), MachineState::Idle, "the hold cancelled the jog");
    assert!(s.position()[0] < 20.0);
    assert!(s.laser_log().is_empty());
}

#[test]
fn homing_answers_ok_when_done_and_no_status_meanwhile() {
    let mut s = Sim::new(SimConfig {
        start_locked: true,
        ..SimConfig::default()
    });
    assert!(s.read().contains("'$X' to unlock"));
    assert_eq!(s.state(), MachineState::Alarm);
    run(&mut s, "$J=G91 X1 F100\n", 0.0);
    assert_eq!(run(&mut s, "$H\n", 0.0), "");
    assert_eq!(run(&mut s, "?", 1.0), "");
    assert_eq!(s.state(), MachineState::Home);
    assert_eq!(run(&mut s, "", 1.1), "ok\r\n");
    assert_eq!(s.state(), MachineState::Idle);
    let mut off = Sim::new(SimConfig {
        homing: false,
        ..SimConfig::default()
    });
    off.read();
    assert_eq!(run(&mut off, "$H\n", 0.0), "error:5\r\n");
}

#[test]
fn faults_alarm_stall_and_unplug() {
    let mut s = sim();
    run(&mut s, "M4 S500\nG1 X100 F600\n", 1.0);
    assert!(s.is_lit());
    s.raise_alarm(1);
    assert!(!s.is_lit());
    assert_eq!(s.read(), "ALARM:1\r\n");
    let at = s.position();
    s.advance(1.0);
    assert_eq!(s.position(), at);

    let mut s = sim();
    s.set_silent(true);
    assert_eq!(run(&mut s, "?G0 X1\n", 1.0), "");
    assert_eq!(s.position(), [1.0, 0.0], "a silent machine still moves");

    let mut s = sim();
    run(&mut s, "G1 X100 F600\n", 0.5);
    s.disconnect();
    assert!(!s.is_connected());
    let at = s.position();
    run(&mut s, "G0 X0\n", 1.0);
    assert_eq!(s.position(), at);
}

#[test]
fn without_laser_mode_the_laser_stays_on_during_travel() {
    let mut s = Sim::new(SimConfig {
        laser_mode: false,
        ..SimConfig::default()
    });
    s.read();
    run(&mut s, "M4 S500\nG0 X100\n", 2.0);
    assert!(s.lit_length() > 99.0, "{:?}", s.laser_log());
}
