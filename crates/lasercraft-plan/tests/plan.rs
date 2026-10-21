//! Planner behavior on projects built through the command registry.

use lasercraft_commands::{Registry, Session};
use lasercraft_geom::Point;
use lasercraft_plan::{Edge, Machine, MoveKind, Plan, plan};
use lasercraft_project::Project;

fn project(lines: &[&str]) -> Project {
    let mut s = Session::new(Project::default(), Registry::with_builtin());
    for line in lines {
        s.run_line(line).unwrap_or_else(|e| panic!("{line}: {e}"));
    }
    s.project().clone()
}

fn planned(lines: &[&str]) -> Plan {
    plan(&project(lines), &Machine::default()).unwrap()
}

#[test]
fn empty_project_plans_nothing() {
    let p = planned(&[]);
    assert!(p.moves.is_empty() && p.layers.is_empty() && p.warnings.is_empty());
    assert_eq!(p.stats.bounds, None);
    assert_eq!(p.stats.total_time, 0.0);
    // Front-left origin of the default 400 × 400 bed.
    assert_eq!(p.start, Point::new(0.0, 400.0));
    assert!(p.check_bounds(Project::default().work_area()).is_ok());
}

#[test]
fn layers_run_in_output_order_and_output_off_is_skipped() {
    let p = planned(&[
        "shape.add_rect 10 10 10 10 @1",
        "shape.add_rect 50 10 10 10 @2",
        "shape.add_rect 90 10 10 10 @3",
        "layer.set 2 output off",
        "layer.move 3 top",
    ]);
    let colors: Vec<u8> = p.layers.iter().map(|l| l.color).collect();
    assert_eq!(colors, [3, 1]);
    assert!(p.moves.iter().all(|m| m.layer != 2));
    let first_burn = p.moves.iter().find(|m| m.kind == MoveKind::Burn).unwrap();
    assert_eq!(first_burn.layer, 3);
}

#[test]
fn power_is_clamped_and_only_burn_moves_carry_it() {
    let lines = [
        "shape.add_rect 10 10 10 10 @1",
        "layer.set 1 max_power 90",
        "layer.set 1 mode fill",
        "layer.set 1 overscan on",
    ];
    let limited = Machine {
        max_power: 50.0,
        ..Machine::default()
    };
    let p = plan(&project(&lines), &limited).unwrap();
    assert!(!p.moves.is_empty());
    for m in &p.moves {
        match m.kind {
            MoveKind::Burn => assert_eq!(m.power, 50.0),
            MoveKind::Unlit | MoveKind::Travel => assert_eq!(m.power, 0.0),
        }
    }
    assert_eq!(p.layers[0].power, 50.0);
    let p = planned(&lines);
    assert!(
        p.moves
            .iter()
            .filter(|m| m.kind == MoveKind::Burn)
            .all(|m| m.power == 90.0)
    );
}

#[test]
fn overscan_past_the_bed_edge_is_refused_with_details() {
    let lines = [
        "shape.add_rect 1 100 20 10 @4",
        "layer.set 4 mode fill",
        "layer.set 4 speed 100",
        "layer.set 4 interval 1",
    ];
    let area = *Project::default().work_area();
    // Without overscan the fill sits inside the bed.
    assert!(planned(&lines).check_bounds(&area).is_ok());
    let mut with = lines.to_vec();
    with.push("layer.set 4 overscan on");
    let p = planned(&with);
    // v²/2a = 100² / 2000 = 5 mm past x = 1.
    assert_eq!(p.layers[0].overscan, 5.0);
    assert!((p.stats.bounds.unwrap().min.x + 4.0).abs() < 1e-9);
    let err = p.check_bounds(&area).unwrap_err();
    assert_eq!(err.edge, Edge::Left);
    assert_eq!(err.edge.axis(), 'x');
    assert!((err.distance - 4.0).abs() < 1e-9);
    assert_eq!(
        (err.layer, err.shape, err.kind),
        (4, Some(1), MoveKind::Unlit)
    );
    let text = err.to_string();
    assert!(text.contains("shape 1 on layer 4"), "{text}");
    assert!(text.contains("4.000 mm past the left edge"), "{text}");
    assert!(text.contains("overscan"), "{text}");
}

#[test]
fn shapes_off_the_bed_name_the_axis_and_edge() {
    let area = *project(&["project.set_work_area 100 50"]).work_area();
    let p = plan(
        &project(&["project.set_work_area 100 50", "shape.add_rect 10 40 20 15"]),
        &Machine::default(),
    )
    .unwrap();
    let err = p.check_bounds(&area).unwrap_err();
    assert_eq!(
        (err.edge, err.kind, err.shape),
        (Edge::Bottom, MoveKind::Burn, Some(1))
    );
    assert!((err.distance - 5.0).abs() < 1e-9);
    assert!(err.to_string().contains("(y = 55.000 mm)"), "{err}");
}

#[test]
fn planning_is_deterministic() {
    let lines = [
        "shape.add_ellipse 50 50 20 10 @1",
        "shape.add_polyline closed 40 45 60 45 50 55 @1",
        "shape.add_rect 100 100 30 30 @2",
        "layer.set 1 kerf 0.2",
        "layer.set 1 tabs 3",
        "layer.set 2 mode fill",
        "layer.set 2 angle 30",
        "layer.set 2 crosshatch on",
        "layer.set 2 interval 0.5",
    ];
    let project = project(&lines);
    let a = plan(&project, &Machine::default()).unwrap();
    let b = plan(&project, &Machine::default()).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.dump(), b.dump());
}

#[test]
fn time_estimate_follows_speed_and_acceleration() {
    // One 100 mm line at 10 mm/s with a = 1000: ramps of 0.05 mm.
    let p = planned(&["shape.add_line 0 400 100 400"]);
    let burn: Vec<_> = p
        .moves
        .iter()
        .filter(|m| m.kind == MoveKind::Burn)
        .collect();
    assert_eq!(burn.len(), 1);
    assert!((burn[0].time - (0.01 + 0.01 + 99.9 / 10.0)).abs() < 1e-9);
    assert!((p.stats.burn_time - burn[0].time).abs() < 1e-12);
    assert_eq!(p.stats.travel_length, 0.0, "starts at the origin corner");
    // A gentler machine takes longer.
    let slow = Machine {
        acceleration: 10.0,
        ..Machine::default()
    };
    let p2 = plan(&project(&["shape.add_line 0 400 100 400"]), &slow).unwrap();
    assert!(p2.stats.total_time > p.stats.total_time + 0.5);
}

#[test]
fn warnings_name_layer_and_shape() {
    let p = planned(&[
        "shape.add_rect 10 10 4 4 @5",
        "layer.set 5 speed 500",
        "layer.set 5 tabs 8",
        "layer.set 5 tab_width 2",
        "shape.add_line 0 0 10 0 @6",
        "layer.set 6 mode fill",
    ]);
    let text: Vec<String> = p.warnings.iter().map(ToString::to_string).collect();
    assert!(
        text.iter().any(|w| w.starts_with("layer 5: speed 500")),
        "{text:?}"
    );
    assert!(
        text.iter()
            .any(|w| w.starts_with("layer 5, shape 1: the tabs")),
        "{text:?}"
    );
    assert!(
        text.iter()
            .any(|w| w == "layer 6: fill needs closed shapes; 1 open outline(s) skipped")
    );
    assert!(text.iter().any(|w| w == "layer 6: nothing to fill"));
}

#[test]
fn bad_machine_parameters_are_rejected() {
    let bad = Machine {
        acceleration: 0.0,
        ..Machine::default()
    };
    let err = plan(&Project::default(), &bad).unwrap_err();
    assert!(err.to_string().contains("acceleration"));
}

#[test]
fn scrubbing_follows_the_moves() {
    let p = planned(&["shape.add_line 0 400 100 400"]);
    let (at_start, started) = p.position_at(0.0);
    assert_eq!((at_start, started), (Point::new(0.0, 400.0), 1));
    let (mid, _) = p.position_at(p.stats.total_time / 2.0);
    assert!(mid.x > 0.0 && mid.x < 100.0);
    let (end, all) = p.position_at(p.stats.total_time + 1.0);
    assert_eq!((end, all), (Point::new(100.0, 400.0), p.moves.len()));
}

#[test]
fn closed_shapes_start_at_the_vertex_nearest_the_head() {
    // From the front-left origin (0, 400), the square's nearest corner is
    // its bottom-left one.
    let p = planned(&["shape.add_rect 100 100 50 50"]);
    let first = &p.moves[0];
    assert_eq!(first.kind, MoveKind::Travel);
    assert_eq!(first.to, Point::new(100.0, 150.0));
    let last = p.moves.last().unwrap();
    assert_eq!(last.to, Point::new(100.0, 150.0));
}
