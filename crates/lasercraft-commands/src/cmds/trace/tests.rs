use lasercraft_geom::{Point, Rect};
use lasercraft_project::raster::{Gray, encode_png};
use lasercraft_project::{Geometry, Project};

use super::*;
use crate::{Invocation, Registry, Session};

/// 60 × 40 pixels: a dark 20 × 10 block with a 6 × 4 hole, a dark ring of
/// radius 8 to its right, and a one-pixel speck.
fn drawing() -> Gray {
    let (w, h) = (60u32, 40u32);
    let mut px = vec![240u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
            let block = (5..25).contains(&x) && (5..15).contains(&y);
            let hole = (12..18).contains(&x) && (8..12).contains(&y);
            let r = (fx - 42.0).hypot(fy - 25.0);
            let ring = (6.0..9.0).contains(&r);
            if (block && !hole) || ring {
                px[(y * w + x) as usize] = 20;
            }
        }
    }
    px[(35 * w + 2) as usize] = 0;
    Gray::new(w, h, px).unwrap()
}

#[test]
fn the_trace_follows_the_drawing() {
    let traced = trace(&drawing(), &Adjust::default(), &TraceSettings::default()).unwrap();
    // The block (with its hole) and the ring (with its inside); no speck.
    assert_eq!(traced.paths.len(), 2);
    let block = &traced.paths[0];
    assert_eq!(block.subpaths.len(), 2);
    assert_eq!(
        block.bounds(),
        Some(Rect::from_corners(
            Point::new(5.0, 5.0),
            Point::new(25.0, 15.0)
        ))
    );
    let ring = traced.paths[1].bounds().unwrap();
    // The ring's outer diameter is 18 pixels.
    assert!((ring.width() - 18.0).abs() < 0.5, "{ring:?}");
    let all = TraceSettings {
        speck: 0,
        ..TraceSettings::default()
    };
    assert_eq!(
        trace(&drawing(), &Adjust::default(), &all)
            .unwrap()
            .paths
            .len(),
        3
    );
    let only_ring = TraceSettings {
        area: Some([30, 0, 60, 40]),
        ..TraceSettings::default()
    };
    assert_eq!(
        trace(&drawing(), &Adjust::default(), &only_ring)
            .unwrap()
            .paths
            .len(),
        1
    );
    let none = TraceSettings {
        to: 10,
        ..TraceSettings::default()
    };
    assert!(trace(&drawing(), &Adjust::default(), &none).is_err());
    assert!(TraceSettings::parse(r#"{"from": 200, "to": 100}"#).is_err());
    assert!(TraceSettings::parse(r#"{"smooth": 1}"#).is_err());
    assert_eq!(
        TraceSettings::parse(r#"{"speck": 5}"#).unwrap().speck,
        5,
        "missing keys take their defaults"
    );
}

#[test]
fn tracing_an_image_adds_one_group_and_undoes() {
    let mut project = Project::default();
    let image = lasercraft_project::Image::new(30.0, 20.0, drawing());
    let id = project
        .add_shape_on(Geometry::Image(image), Affine::translate(100.0, 50.0), 2)
        .unwrap();
    let mut s = Session::new(project, Registry::with_builtin());
    let out = s.run_line(&format!("image.trace {id} @5")).unwrap();
    assert!(out.message.contains("2 path(s)"), "{}", out.message);
    let traced: Vec<_> = s
        .project()
        .shapes()
        .iter()
        .filter(|sh| sh.id() != id)
        .collect();
    assert_eq!(traced.len(), 2);
    assert!(traced.iter().all(|sh| sh.layer() == 5));
    assert_eq!(traced[0].groups().len(), 1);
    assert_eq!(traced[0].groups(), traced[1].groups());
    // 60 pixels across 30 mm: the block's corner (5, 5) px is at 102.5, 52.5.
    let b = traced[0].bounds();
    assert!(
        (b.min.x - 102.5).abs() < 1e-9 && (b.min.y - 52.5).abs() < 1e-9,
        "{b:?}"
    );
    s.undo().unwrap();
    assert_eq!(s.project().shapes().len(), 1);
    // Deleting the image in the same step, undone in one step.
    s.run_line(&format!("image.trace {id} \"{{\\\"delete\\\": true}}\""))
        .unwrap();
    assert!(s.project().shape(id).is_none());
    s.undo().unwrap();
    assert!(s.project().shape(id).is_some());
    assert_eq!(s.project().shapes().len(), 1);
    assert!(s.run_line("image.trace 999").is_err());
}

#[test]
fn a_picture_file_traces_onto_its_bed_rectangle() {
    let dir = std::env::temp_dir().join(format!("lasercraft-trace-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("camera.png");
    std::fs::write(&path, encode_png(&drawing())).unwrap();
    let mut s = Session::new(Project::default(), Registry::with_builtin());
    s.run(&Invocation::new(
        "image.trace_picture",
        [
            path.display().to_string(),
            "10".into(),
            "20".into(),
            "120".into(),
            "80".into(),
        ],
    ))
    .unwrap();
    // 2 mm per pixel: the block's corner (5, 5) px is at 20, 30.
    let b = s.project().shapes()[0].bounds();
    assert!(
        (b.min.x - 20.0).abs() < 1e-9 && (b.min.y - 30.0).abs() < 1e-9,
        "{b:?}"
    );
    let err = s
        .run(&Invocation::new(
            "image.trace_picture",
            [
                path.display().to_string(),
                "0".into(),
                "0".into(),
                "10".into(),
                "10".into(),
                r#"{"delete": true}"#.into(),
            ],
        ))
        .unwrap_err();
    assert!(err.to_string().contains("no image to delete"), "{err}");
    let _ = std::fs::remove_file(path);
}
