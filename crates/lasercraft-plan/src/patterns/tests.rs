use lasercraft_project::WorkArea;
use serde_json::json;

use super::*;
use crate::MoveKind;

fn area() -> WorkArea {
    WorkArea::default()
}

fn build_json(
    kind: &str,
    value: serde_json::Value,
    machine: &Machine,
) -> Result<Pattern, PlanError> {
    build(&TestSettings::parse(kind, Some(&value))?, &area(), machine)
}

/// The layer settings of each burn move's box, in run order: (speed,
/// power) of the first burn move after each travel on layer 0.
fn box_runs(plan: &Plan) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut last_shape = None;
    for m in plan
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Burn && m.layer == BOX)
    {
        if m.shape != last_shape {
            out.push((m.speed, m.power));
            last_shape = m.shape;
        }
    }
    out
}

#[test]
fn the_default_grid_has_every_box_labels_and_a_border() {
    let p = build(
        &TestSettings::parse("speed_power", None).unwrap(),
        &area(),
        &Machine::default(),
    )
    .unwrap();
    let runs = box_runs(&p.plan);
    assert_eq!(runs.len(), 100);
    // Least energy first: the fastest speed and the lowest power.
    assert_eq!(runs[0], (100.0, 10.0));
    assert_eq!(*runs.last().unwrap(), (10.0, 100.0));
    for w in runs.windows(2) {
        let e = |(speed, power): (f64, f64)| power / speed;
        assert!(e(w[0]) <= e(w[1]) + 1e-12, "{w:?}");
    }
    // Labels and border run before the boxes.
    let first_box = p
        .plan
        .moves
        .iter()
        .position(|m| m.layer == BOX && m.kind == MoveKind::Burn)
        .unwrap();
    assert!(
        p.plan.moves[..first_box]
            .iter()
            .any(|m| m.layer == LABELS && m.kind == MoveKind::Burn)
    );
    assert!(
        p.plan.moves[..first_box]
            .iter()
            .any(|m| m.layer == BORDER && m.kind == MoveKind::Burn)
    );
    // Centered on the bed and inside it.
    let b = p.plan.stats.bounds.unwrap();
    assert!(
        (b.center().x - 200.0).abs() < 1.0 && (b.center().y - 200.0).abs() < 1.0,
        "{b:?}"
    );
    assert!(p.plan.check_bounds(&area()).is_ok());
    assert!(p.size.0 > 60.0 && p.size.0 < 120.0, "{:?}", p.size);
    // The scratch project holds the shapes on layers 0–2.
    assert!(p.project.shapes().iter().any(|s| s.layer() == LABELS));
    assert_eq!(p.project.layer(BOX).unwrap().name, "Boxes");
}

#[test]
fn settings_merge_with_the_defaults_and_are_checked() {
    let p = build_json(
        "speed_power",
        json!({
            "rows": {"count": 3, "setting": "passes", "from": 1, "to": 3},
            "columns": {"count": 2},
            "draw_labels": false,
            "draw_border": false,
            "center": [50, 60]
        }),
        &Machine::default(),
    )
    .unwrap();
    assert_eq!(box_runs(&p.plan).len(), 6);
    assert!(p.plan.moves.iter().all(|m| m.layer == BOX));
    let b = p.plan.stats.bounds.unwrap();
    assert!(
        (b.center().x - 50.0).abs() < 0.5 && (b.center().y - 60.0).abs() < 0.5,
        "{b:?}"
    );
    for bad in [
        json!({"rows": {"setting": "power"}}),
        json!({"rows": {"count": 1}}),
        json!({"rows": {"to": 50000}}),
        json!({"colour": 3}),
        json!({"box": {"speed": -1}}),
        json!([1, 2]),
    ] {
        assert!(
            build_json("speed_power", bad.clone(), &Machine::default()).is_err(),
            "{bad}"
        );
    }
    assert!(TestSettings::parse("focus", None).is_err());
    // Round trip through the complete JSON.
    let s = TestSettings::parse("speed_power", Some(&json!({"spacing": 3}))).unwrap();
    assert_eq!(
        TestSettings::parse("speed_power", Some(&s.to_value())).unwrap(),
        s
    );
}

#[test]
fn powers_above_the_device_maximum_are_clamped_and_warned() {
    let machine = Machine {
        max_power: 50.0,
        ..Machine::default()
    };
    let p = build_json("speed_power", json!({}), &machine).unwrap();
    assert!(p.plan.moves.iter().all(|m| m.power <= 50.0));
    assert!(
        p.plan
            .warnings
            .iter()
            .any(|w| w.message == "the device maximum is 50%. 50% is used.")
    );
    let fine = build_json("speed_power", json!({"columns": {"to": 40}}), &machine).unwrap();
    assert!(
        fine.plan
            .warnings
            .iter()
            .all(|w| !w.message.contains("device maximum"))
    );
}

#[test]
fn the_interval_test_runs_the_widest_spacing_first() {
    let p = build_json("interval", json!({"patches": 4}), &Machine::default()).unwrap();
    // Each patch is one fill; the widest interval has the fewest lines.
    let mut lines_per_patch = Vec::new();
    let mut last = None;
    for m in p
        .plan
        .moves
        .iter()
        .filter(|m| m.layer == BOX && m.kind == MoveKind::Burn)
    {
        if m.shape != last {
            lines_per_patch.push(0);
            last = m.shape;
        }
        *lines_per_patch.last_mut().unwrap() += 1;
    }
    assert_eq!(lines_per_patch.len(), 4);
    assert!(
        lines_per_patch.windows(2).all(|w| w[0] < w[1]),
        "{lines_per_patch:?}"
    );
    assert!(p.plan.moves.iter().any(|m| m.layer == LABELS));
    // Gradients engrave images.
    let g = build_json(
        "interval",
        json!({"patches": 2, "fill": "gradient"}),
        &Machine::default(),
    )
    .unwrap();
    assert!(g.project.shapes().iter().any(|s| s.geometry().is_image()));
    assert!(
        g.plan
            .moves
            .iter()
            .any(|m| m.layer == BOX && m.kind == MoveKind::Burn)
    );
    assert!(
        build_json(
            "interval",
            json!({"smallest": 0.5, "largest": 0.2}),
            &Machine::default()
        )
        .is_err()
    );
}

#[test]
fn a_pattern_too_big_for_the_bed_is_caught_by_the_bounds_check() {
    let small = WorkArea::new(40.0, 40.0, lasercraft_project::Origin::FrontLeft).unwrap();
    let p = build(
        &TestSettings::parse("speed_power", None).unwrap(),
        &small,
        &Machine::default(),
    )
    .unwrap();
    assert!(p.plan.check_bounds(&small).is_err());
}
