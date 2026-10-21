//! Export tests: golden programs and the safety properties of
//! `docs/spec/gcode.md`.

use lasercraft_geom::{Affine, Point};
use lasercraft_plan::Machine;
use lasercraft_project::{Geometry, Origin, Project};

use super::*;

fn rect(p: &mut Project, x: f64, y: f64, w: f64, h: f64, color: u8) -> u32 {
    p.add_shape_on(
        Geometry::Rect {
            width: w,
            height: h,
        },
        Affine::translate(x, y),
        color,
    )
    .unwrap()
}

fn set(p: &mut Project, color: u8, settings: &[(&str, &str)]) {
    for (k, v) in settings {
        p.set_layer_setting(color, k, v).unwrap();
    }
}

/// A small job: a filled square engraved, then a cut square with air.
fn job() -> Project {
    let mut p = Project::with_work_area(100.0, 80.0).unwrap();
    p.set_name("Golden (job); test").unwrap();
    rect(&mut p, 10.0, 10.0, 4.0, 2.0, 2);
    rect(&mut p, 5.0, 5.0, 20.0, 20.0, 1);
    set(
        &mut p,
        2,
        &[
            ("mode", "fill"),
            ("interval", "0.5"),
            ("speed", "100"),
            ("max_power", "40"),
        ],
    );
    set(
        &mut p,
        1,
        &[
            ("speed", "8"),
            ("max_power", "90"),
            ("air", "on"),
            ("passes", "2"),
        ],
    );
    let mut order = p.layer_order();
    let (a, b) = (
        order.iter().position(|c| *c == 1).unwrap(),
        order.iter().position(|c| *c == 2).unwrap(),
    );
    order.swap(a, b);
    p.set_layer_order(&order).unwrap();
    p
}

fn export_default(p: &Project) -> Result<Exported, ExportError> {
    export(p, &Machine::default(), &Options::default())
}

fn check_golden(name: &str, text: &str) {
    let path = format!("{}/tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
    if std::env::var_os("LASERCRAFT_UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, text).unwrap();
    }
    let golden = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        text, golden,
        "{name} differs; set LASERCRAFT_UPDATE_GOLDEN=1 to rewrite"
    );
}

#[test]
fn job_matches_golden_program() {
    let out = export_default(&job()).unwrap();
    check_golden("job.gcode", &out.gcode);
}

#[test]
fn verifier_agrees_with_the_plan() {
    let out = export_default(&job()).unwrap();
    let s = &out.plan.stats;
    // Coordinates are rounded to 0.001 mm, so lengths agree closely.
    assert!(
        (out.report.burn_length - s.burn_length).abs() < 0.01,
        "{:?} vs {:?}",
        out.report,
        s
    );
    assert!((out.report.unlit_length - s.unlit_length).abs() < 0.01);
    assert!((out.report.travel_length - s.travel_length).abs() < 0.01);
    assert_eq!(out.report.max_s, 900.0);
}

#[test]
fn layout_of_the_program() {
    let text = export_default(&job()).unwrap().gcode;
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("(LaserCraft"));
    assert_eq!(lines[1], "(Job: Golden _job__ test)");
    let first_motion = lines
        .iter()
        .position(|l| l.starts_with('G') && l.contains('X'))
        .unwrap();
    assert!(lines[..first_motion].contains(&"G90 G21"));
    assert!(lines[..first_motion].contains(&"M5 S0"));
    assert_eq!(
        lines.last(),
        Some(&"M9"),
        "air was used, so it is switched off last"
    );
    assert_eq!(lines[lines.len() - 2], "M5 S0");
    // Travel never carries power.
    for l in &lines {
        if l.starts_with("G0") {
            assert!(!l.contains('S') && !l.contains('F'), "{l}");
        }
        assert!(l.len() <= MAX_LINE);
    }
    // Air on before the cut layer; dynamic power by default.
    assert!(lines.contains(&"M8"));
    assert!(lines.contains(&"M4 S0"));
    assert!(!lines.contains(&"M3 S0"));
}

#[test]
fn constant_power_layers_use_m3_and_air_can_use_m7() {
    let mut p = job();
    set(&mut p, 1, &[("constant_power", "on")]);
    let options = Options {
        air: Air::M7,
        finish_at_origin: true,
        ..Options::default()
    };
    let text = export(&p, &Machine::default(), &options).unwrap().gcode;
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.contains(&"M3 S0") && lines.contains(&"M4 S0"));
    assert!(lines.contains(&"M7") && !lines.contains(&"M8"));
    assert_eq!(lines.last(), Some(&"G0 X0 Y0"));
}

#[test]
fn power_is_clamped_to_the_machine_and_scaled_by_s_max() {
    let p = job();
    let machine = Machine {
        max_power: 50.0,
        ..Machine::default()
    };
    let options = Options {
        s_max: 255.0,
        ..Options::default()
    };
    let out = export(&p, &machine, &options).unwrap();
    // Layer 1 asks for 90%, the machine allows 50%: 127.5 of 255.
    assert_eq!(out.report.max_s, 127.5);
    assert!(out.gcode.contains("S102"), "layer 2 at 40% of 255");
    // Odd ceilings never round up past the limit.
    let machine = Machine {
        max_power: 33.333,
        ..Machine::default()
    };
    let out = export(&p, &machine, &options).unwrap();
    assert!(out.report.max_s <= 255.0 * 0.33333);
}

#[test]
fn off_bed_jobs_are_refused_before_anything_is_written() {
    let mut p = job();
    rect(&mut p, 90.0, 70.0, 20.0, 5.0, 1);
    match export_default(&p) {
        Err(ExportError::OutOfBounds(e)) => {
            let msg = ExportError::OutOfBounds(e).to_string();
            assert!(msg.contains("does not fit"), "{msg}");
        }
        other => panic!("expected out of bounds, got {other:?}"),
    }
}

#[test]
fn empty_or_disabled_jobs_are_refused() {
    let p = Project::default();
    assert_eq!(export_default(&p).unwrap_err(), ExportError::Nothing);
    let mut p = job();
    set(&mut p, 1, &[("output", "off")]);
    set(&mut p, 2, &[("output", "off")]);
    assert_eq!(export_default(&p).unwrap_err(), ExportError::Nothing);
}

#[test]
fn bad_options_are_refused() {
    let p = job();
    let bad = Options {
        s_max: 0.0,
        ..Options::default()
    };
    assert!(matches!(
        export(&p, &Machine::default(), &bad),
        Err(ExportError::Invalid(_))
    ));
    let bad_machine = Machine {
        max_speed: 0.0,
        ..Machine::default()
    };
    assert!(matches!(
        export(&p, &bad_machine, &Options::default()),
        Err(ExportError::Invalid(_))
    ));
    assert!("m9".parse::<Air>().is_err());
    assert_eq!("M7".parse::<Air>().unwrap(), Air::M7);
}

#[test]
fn every_origin_corner_stays_in_positive_coordinates() {
    for origin in [
        Origin::FrontLeft,
        Origin::FrontRight,
        Origin::RearLeft,
        Origin::RearRight,
    ] {
        let mut p = job();
        p.set_origin(origin);
        let out = export_default(&p).unwrap();
        let [x0, y0, x1, y1] = out.report.bounds.unwrap();
        assert!(
            x0 >= 0.0 && y0 >= 0.0 && x1 <= 100.0 && y1 <= 80.0,
            "{origin}: {:?}",
            out.report.bounds
        );
        // Same toolpath, so the same lengths.
        assert!(
            (out.report.burn_length - export_default(&job()).unwrap().report.burn_length).abs()
                < 0.01
        );
    }
}

#[test]
fn curves_export_through_the_planner() {
    let mut p = Project::with_work_area(100.0, 100.0).unwrap();
    p.add_shape_on(
        Geometry::Ellipse { rx: 10.0, ry: 10.0 },
        Affine::translate(50.0, 50.0),
        0,
    )
    .unwrap();
    let out = export_default(&p).unwrap();
    let circumference = 2.0 * std::f64::consts::PI * 10.0;
    assert!(
        (out.report.burn_length - circumference).abs() < 0.05,
        "{}",
        out.report.burn_length
    );
    let _ = Point::ZERO;
}

fn one_square() -> Project {
    // 20 × 10 mm cut at document (10, 10) on a 100 × 80 bed, front-left origin:
    // machine X 10..30, Y 60..70.
    let mut p = Project::with_work_area(100.0, 80.0).unwrap();
    rect(&mut p, 10.0, 10.0, 20.0, 10.0, 1);
    p
}

fn relative(at: (f64, f64), anchor: Anchor, back: bool) -> Options {
    Options {
        placement: Placement::Relative {
            at: Point::new(at.0, at.1),
            anchor,
            return_to_start: back,
        },
        ..Options::default()
    }
}

#[test]
fn relative_placement_puts_the_job_origin_on_the_point() {
    let p = one_square();
    let m = Machine::default();
    // Bottom-left (front-left) of the job at machine (50, 20).
    let out = export(&p, &m, &relative((50.0, 20.0), Anchor::BottomLeft, false)).unwrap();
    assert!(out.gcode.contains("G0 X50 Y20\n"), "{}", out.gcode);
    assert_eq!(out.report.bounds.map(|b| (b[2], b[3])), Some((70.0, 30.0)));
    // Centered on (50, 40), returning there.
    let out = export(&p, &m, &relative((50.0, 40.0), Anchor::Center, true)).unwrap();
    assert!(out.gcode.contains("G0 X40 Y35\n"), "{}", out.gcode);
    assert!(out.gcode.ends_with("G0 X50 Y40\nM5 S0\n"), "{}", out.gcode);
    // Absolute mode ignores everything else.
    let out = export(&p, &m, &Options::default()).unwrap();
    assert!(
        out.gcode.contains("(Bounds: X10..30 Y60..70 mm)"),
        "{}",
        out.gcode
    );
}

#[test]
fn placement_off_the_bed_is_refused() {
    let p = one_square();
    let m = Machine::default();
    let err = export(&p, &m, &relative((90.0, 20.0), Anchor::BottomLeft, false)).unwrap_err();
    assert!(err.to_string().contains("does not fit"), "{err}");
    // The return move counts too: the start point itself is off the bed.
    let err = export(&p, &m, &relative((5.0, -1.0), Anchor::Center, true)).unwrap_err();
    assert!(matches!(err, ExportError::OutOfBounds(_)), "{err}");
}

#[test]
fn the_first_move_is_always_written() {
    // The head may be anywhere when a job starts, even exactly at the first
    // cut's start in machine terms; the move to it must still be there.
    let mut p = Project::with_work_area(100.0, 80.0).unwrap();
    rect(&mut p, 0.0, 70.0, 10.0, 10.0, 1);
    let out = export(&p, &Machine::default(), &Options::default()).unwrap();
    let first_motion = out
        .gcode
        .lines()
        .find(|l| l.starts_with("G0") || l.starts_with("G1"))
        .unwrap();
    assert!(first_motion.starts_with("G0"), "{}", out.gcode);
}

#[test]
fn frames_are_laser_off_and_traced_at_the_given_speed() {
    let p = one_square();
    let m = Machine::default();
    let text = frame(&p, &m, &Options::default(), FrameShape::Box, 50.0, None).unwrap();
    assert!(!text.contains("M3") && !text.contains("M4"), "{text}");
    assert!(text.contains("F3000"), "{text}");
    let limits = Limits {
        width: 100.0,
        height: 80.0,
        s_ceiling: 0.0,
    };
    let report = verify(&text, &limits).unwrap();
    assert_eq!(report.burn_length, 0.0);
    assert!((report.unlit_length - 60.0).abs() < 1e-9, "{report:?}");
    for corner in ["X10 Y70", "X30 Y70", "X30 Y60", "X10 Y60"] {
        assert!(text.contains(corner), "{corner}: {text}");
    }
    // With a known head position the frame starts and ends there.
    let text = frame(
        &p,
        &m,
        &relative((50.0, 40.0), Anchor::Center, true),
        FrameShape::Box,
        50.0,
        Some(Point::new(50.0, 40.0)),
    )
    .unwrap();
    assert!(text.ends_with("G0 X50 Y40\nM5 S0\n"), "{text}");
    assert!(text.contains("G0 X40 Y45\n"), "{text}");
}

#[test]
fn outline_frames_follow_the_convex_hull() {
    let mut p = Project::with_work_area(100.0, 100.0).unwrap();
    p.add_shape_on(
        Geometry::Polyline {
            points: vec![
                Point::new(10.0, 10.0),
                Point::new(50.0, 10.0),
                Point::new(10.0, 50.0),
            ],
            closed: true,
        },
        Affine::IDENTITY,
        1,
    )
    .unwrap();
    let m = Machine::default();
    let boxed = frame(&p, &m, &Options::default(), FrameShape::Box, 20.0, None).unwrap();
    let hull = frame(&p, &m, &Options::default(), FrameShape::Outline, 20.0, None).unwrap();
    let limits = Limits {
        width: 100.0,
        height: 100.0,
        s_ceiling: 0.0,
    };
    let b = verify(&boxed, &limits).unwrap().unlit_length;
    let h = verify(&hull, &limits).unwrap().unlit_length;
    assert!((b - 160.0).abs() < 1e-6, "{b}");
    assert!((h - (80.0 + 40f64.hypot(40.0))).abs() < 1e-6, "{h}");
}

#[test]
fn frame_refusals() {
    let p = one_square();
    let m = Machine::default();
    let opts = Options::default();
    assert!(frame(&p, &m, &opts, FrameShape::Box, 0.0, None).is_err());
    assert!(frame(&p, &m, &opts, FrameShape::Box, 1000.0, None).is_err());
    assert_eq!(
        frame(&Project::default(), &m, &opts, FrameShape::Box, 10.0, None).unwrap_err(),
        ExportError::Nothing
    );
    let off = relative((95.0, 20.0), Anchor::BottomLeft, false);
    assert!(matches!(
        frame(&p, &m, &off, FrameShape::Box, 10.0, None).unwrap_err(),
        ExportError::OutOfBounds(_)
    ));
}
