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
