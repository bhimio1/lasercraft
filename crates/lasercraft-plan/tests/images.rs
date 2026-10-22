//! Image layers (`docs/spec/images.md`, "Planning").

use lasercraft_geom::{Affine, Point};
use lasercraft_plan::{Machine, MoveKind, Plan, plan};
use lasercraft_project::raster::Gray;
use lasercraft_project::{Geometry, Image, Project};

/// A 4 × 2 mm image at (10, 20): left half black, right half white.
fn project(dither: &str) -> Project {
    let mut p = Project::with_work_area(100.0, 100.0).unwrap();
    let px = vec![0, 0, 255, 255, 0, 0, 255, 255];
    let img = Image::new(4.0, 2.0, Gray::new(4, 2, px).unwrap());
    p.add_shape_on(Geometry::Image(img), Affine::translate(10.0, 20.0), 2)
        .unwrap();
    for (k, v) in [
        ("image.dither", dither),
        ("image.interval", "0.5"),
        ("speed", "50"),
        ("max_power", "60"),
        ("min_power", "10"),
    ] {
        p.set_layer_setting(2, k, v).unwrap();
    }
    p
}

fn burns(plan: &Plan) -> Vec<(Point, Point, f64)> {
    plan.moves
        .iter()
        .filter(|m| m.kind == MoveKind::Burn)
        .map(|m| (m.from, m.to, m.power))
        .collect()
}

#[test]
fn dithered_rows_burn_the_dark_half_bottom_up_both_ways() {
    let plan = plan(&project("threshold"), &Machine::default()).unwrap();
    let b = burns(&plan);
    // Four rows of 0.5 mm; each burns x 10–12 at max power.
    assert_eq!(b.len(), 4, "{b:?}");
    assert_eq!(
        b[0],
        (Point::new(10.0, 21.75), Point::new(12.0, 21.75), 60.0)
    );
    assert_eq!(
        b[1],
        (Point::new(12.0, 21.25), Point::new(10.0, 21.25), 60.0),
        "back"
    );
    assert_eq!(b[3].0.y, 20.25);
    assert!((plan.stats.burn_length - 8.0).abs() < 1e-9);
}

#[test]
fn grayscale_maps_levels_between_min_and_max_power() {
    let mut p = project("grayscale");
    let px = vec![0, 128, 255, 255];
    let img = Image::new(2.0, 0.5, Gray::new(4, 1, px).unwrap());
    let mut q = Project::with_work_area(100.0, 100.0).unwrap();
    q.add_shape_on(Geometry::Image(img), Affine::translate(0.0, 0.0), 2)
        .unwrap();
    for k in [
        "image.dither",
        "image.interval",
        "speed",
        "max_power",
        "min_power",
    ] {
        q.set_layer_setting(2, k, &p.layer(2).unwrap().get(k).unwrap())
            .unwrap();
    }
    p = q;
    let b = burns(&plan(&p, &Machine::default()).unwrap());
    assert_eq!(b.len(), 2, "white is not burned: {b:?}");
    assert_eq!(b[0].2, 60.0);
    let mid = 10.0 + 127.0 / 255.0 * 50.0;
    assert!((b[1].2 - mid).abs() < 0.3, "{}", b[1].2);
}

#[test]
fn overscan_dot_correction_one_way_and_passes() {
    let mut p = project("threshold");
    p.set_layer_setting(2, "overscan", "on").unwrap();
    p.set_layer_setting(2, "bidirectional", "off").unwrap();
    p.set_layer_setting(2, "image.dot_correction", "0.2")
        .unwrap();
    p.set_layer_setting(2, "passes", "2").unwrap();
    let m = Machine::default();
    let plan = plan(&p, &m).unwrap();
    let b = burns(&plan);
    assert_eq!(b.len(), 8);
    for (from, to, _) in &b {
        assert!(
            (from.x - 10.1).abs() < 1e-9 && (to.x - 11.9).abs() < 1e-9,
            "{from:?} {to:?}"
        );
    }
    let run_on = m.overscan(50.0);
    let unlit_max = plan
        .moves
        .iter()
        .filter(|mv| mv.kind == MoveKind::Unlit)
        .map(|mv| mv.to.x)
        .fold(0.0, f64::max);
    assert!((unlit_max - (11.9 + run_on)).abs() < 1e-9);
}

#[test]
fn images_are_skipped_when_blank_too_fine_or_not_output() {
    let mut p = project("threshold");
    p.set_layer_setting(2, "image.negative", "on").unwrap();
    let plan_neg = plan(&p, &Machine::default()).unwrap();
    assert!(
        burns(&plan_neg)
            .iter()
            .all(|(f, t, _)| f.x.min(t.x) >= 12.0)
    );
    p.set_layer_setting(2, "output", "off").unwrap();
    assert!(plan(&p, &Machine::default()).unwrap().moves.is_empty());
    let mut fine = project("threshold");
    let id = fine.order()[0];
    fine.set_shape_transform(id, Affine::scale(50.0, 50.0))
        .unwrap();
    fine.set_layer_setting(2, "image.interval", "0.001")
        .unwrap();
    let plan = plan(&fine, &Machine::default()).unwrap();
    assert!(plan.moves.is_empty());
    assert!(
        plan.warnings[0].message.contains("image skipped"),
        "{:?}",
        plan.warnings
    );
}
