use lasercraft_device::{Request, StreamKind};
use lasercraft_project::{Fit, Project};

use crate::{DeviceContext, Invocation, Registry, Session, with_context};

fn session(lines: &[&str]) -> Session {
    let mut s = Session::new(Project::default(), Registry::with_builtin());
    for line in lines {
        s.run_line(line).unwrap();
    }
    s
}

#[test]
fn rotary_settings_are_checked_and_undone() {
    let mut s = session(&[]);
    let out = s
        .run_line("rotary.set enabled on kind=chuck object_diameter=60")
        .unwrap();
    assert!(out.message.contains("soft limits"), "{}", out.message);
    let r = s.project().rotary();
    assert!(r.enabled && r.object_diameter == 60.0);
    assert!(s.run_line("rotary.set mm_per_turn -3").is_err());
    assert_eq!(s.project().rotary(), r, "a failed change changes nothing");
    s.run_line("rotary.set mirror on").unwrap();
    s.run_line("edit.undo").unwrap();
    assert!(!s.project().rotary().mirror);
    s.run_line("rotary.clear").unwrap();
    assert!(!s.project().rotary_on());
    s.run_line("edit.undo").unwrap();
    assert!(s.project().rotary_on());
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project().rotary(), lasercraft_project::Rotary::default());
}

#[test]
fn the_rotary_test_turns_once_and_back_with_the_laser_off() {
    let mut s = session(&["rotary.set mm_per_turn 72"]);
    match s.run_line("device.rotary_test speed=20").unwrap().device {
        Some(Request::Stream { kind, gcode }) => {
            assert_eq!(kind, StreamKind::Frame);
            assert!(gcode.contains("G91") && gcode.contains("Y72 F1200") && gcode.contains("Y-72"));
            assert!(!gcode.contains("M3") && !gcode.contains("M4"));
            assert!(gcode.trim_end().ends_with("G90"));
        }
        other => panic!("{other:?}"),
    }
}

fn ctx(x: f64, y: f64) -> DeviceContext {
    DeviceContext {
        head: Some(lasercraft_geom::Point::new(x, y)),
        user_origin: None,
        s_max: None,
    }
}

fn run_at(
    s: &mut Session,
    line: &str,
    x: f64,
    y: f64,
) -> Result<crate::Outcome, crate::CommandError> {
    s.run(&with_context(&Invocation::parse(line).unwrap(), &ctx(x, y)))
}

#[test]
fn targets_fit_the_job_and_errors_name_the_problem() {
    // Two crosshair-ish targets 100 mm apart and a part between them.
    let mut s = session(&[
        "shape.add_rect 5 5 10 10",
        "shape.add_rect 105 5 10 10",
        "shape.add_rect 40 20 20 20",
    ]);
    assert!(run_at(&mut s, "registration.set_target 2 2", 0.0, 0.0).is_err());
    let none = s.run_line("registration.set_target 1 1").unwrap_err();
    assert!(none.to_string().contains("did not report"), "{none}");
    // Front-left bed 400 deep: document (10, 10) is machine (10, 390).
    run_at(&mut s, "registration.set_target 1 1", 13.0, 390.0).unwrap();
    let same = run_at(&mut s, "registration.set_target 2 1", 113.0, 390.0).unwrap_err();
    assert!(same.to_string().contains("same object"), "{same}");
    let still = run_at(&mut s, "registration.set_target 2 2", 13.2, 390.0).unwrap_err();
    assert!(still.to_string().contains("still where"), "{still}");
    let two = run_at(&mut s, "registration.set_target 2 1 2", 113.0, 390.0).unwrap_err();
    assert!(two.to_string().contains("2 objects"), "{two}");
    // The expected position of target 2 is target 1 plus the design offset.
    match s.run_line("registration.expected 2").unwrap().device {
        Some(Request::Jog { x, y, relative, .. }) => {
            assert!(!relative && (x - 113.0).abs() < 1e-9 && (y - 390.0).abs() < 1e-9);
        }
        other => panic!("{other:?}"),
    }
    run_at(&mut s, "registration.set_target 2 2", 113.0, 390.0).unwrap();
    assert!(s.run_line("registration.fit sideways").is_err());
    s.run_line("registration.fit scaled").unwrap();
    assert_eq!(s.project().registration().fit, Fit::Scaled);
    // Planning moves the part 3 mm right; the shapes do not change.
    let p = s.project();
    let plan = lasercraft_plan::plan(p, &lasercraft_plan::Machine::default()).unwrap();
    let b = plan.stats.bounds.unwrap();
    assert!((b.min.x - 8.0).abs() < 0.2, "{b:?}");
    assert_eq!(p.shape(3).unwrap().bounds().min.x, 40.0);
    // Undo the fit, then the second target, then the first.
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project().registration().fit, Fit::None);
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project().registration().targets.len(), 1);
    s.run_line("edit.redo").unwrap();
    assert_eq!(s.project().registration().targets.len(), 2);
    s.run_line("registration.clear").unwrap();
    assert!(s.project().registration().targets.is_empty());
    s.run_line("edit.undo").unwrap();
    assert_eq!(s.project().registration().targets.len(), 2);
}

#[test]
fn a_badly_scaled_fit_is_refused_and_jobs_run_absolute() {
    let mut s = session(&["shape.add_rect 5 5 10 10", "shape.add_rect 105 5 10 10"]);
    run_at(&mut s, "registration.set_target 1 1", 10.0, 390.0).unwrap();
    run_at(&mut s, "registration.set_target 2 2", 120.0, 390.0).unwrap();
    let err = s.run_line("registration.fit scaled").unwrap_err();
    assert!(err.to_string().contains("10.0% farther"), "{err}");
    s.run_line("registration.fit rigid").unwrap();
    let start = run_at(&mut s, "device.start from=current", 50.0, 50.0).unwrap();
    assert!(
        start
            .message
            .contains("Print registration places the job by its targets"),
        "{}",
        start.message
    );
}

#[test]
fn taper_narrows_the_selection_and_refuses_images() {
    let mut s = session(&["shape.add_rect 0 0 40 100"]);
    s.run_line("shape.taper top 80 100 60 1").unwrap();
    let b = s.project().shape(1).unwrap().bounds();
    assert!((b.width() - 40.0).abs() < 1e-6 && (b.height() - 100.0).abs() < 1e-6);
    // The bottom edge is now 30 mm wide, centered.
    let outline = &s.project().shape(1).unwrap().outlines(0.01)[0].0;
    let bottom: Vec<_> = outline
        .iter()
        .filter(|p| (p.y - 100.0).abs() < 1e-9)
        .collect();
    let xs: Vec<f64> = bottom.iter().map(|p| p.x).collect();
    let (lo, hi) = xs
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
    assert!(
        (lo - 5.0).abs() < 1e-6 && (hi - 35.0).abs() < 1e-6,
        "{xs:?}"
    );
    s.run_line("edit.undo").unwrap();
    assert!(matches!(
        s.project().shape(1).unwrap().geometry(),
        lasercraft_project::Geometry::Rect { .. }
    ));
    assert!(s.run_line("shape.taper middle 80 100 60 1").is_err());
    assert!(s.run_line("shape.taper top 0 100 60 1").is_err());
}
