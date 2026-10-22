use lasercraft_camera::lens::{Lens, Model};
use lasercraft_camera::marks::{Pattern, burned};
use lasercraft_camera::sheet;
use lasercraft_camera::synth::{Pose, Renderer};
use lasercraft_device::{Request, StreamKind};
use lasercraft_geom::Point;
use lasercraft_project::Project;

use crate::{Invocation, Registry, Session};

fn session() -> Session {
    let mut s = Session::new(Project::default(), Registry::with_builtin());
    s.run_line("profile.add Shop width=300 height=200").unwrap();
    s.run_line("project.set_device Shop").unwrap();
    s
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("lasercraft-camera-cmds-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn run(
    s: &mut Session,
    name: &str,
    args: &[String],
) -> Result<crate::Outcome, crate::CommandError> {
    s.run(&Invocation::new(name, args.iter().cloned()))
}

#[test]
fn cameras_need_a_device_profile_and_check_their_settings() {
    let mut s = Session::new(Project::default(), Registry::with_builtin());
    let err = s.run_line("camera.add Top usb").unwrap_err();
    assert!(
        err.to_string().contains("Choose one in the Machine panel"),
        "{err}"
    );
    let mut s = session();
    let depth = s.undo_depth();
    s.run_line("camera.add Top usb index=1 format=1280x720")
        .unwrap();
    assert!(
        s.run_line("camera.add top simulated").is_err(),
        "names are unique"
    );
    assert!(
        s.run_line("camera.add Lid network").is_err(),
        "a URL is needed"
    );
    s.run_line("camera.add Lid network url=http://cam.local/still.jpg rate=1")
        .unwrap();
    s.run_line("camera.set Top opacity 55").unwrap();
    assert!(s.run_line("camera.set Top url http://x/y").is_err());
    s.run_line("camera.rename Lid \"Lid camera\"").unwrap();
    let list = s.run_line("camera.list").unwrap().message;
    assert!(
        list.contains("Top: USB · camera 1; lens not corrected; not aligned")
            && list.contains("Lid camera: Network · http://cam.local/still.jpg"),
        "{list}"
    );
    let path = temp("top.lccam");
    run(
        &mut s,
        "camera.export",
        &["Top".into(), path.display().to_string()],
    )
    .unwrap();
    s.run_line("camera.remove Top").unwrap();
    run(&mut s, "camera.import", &[path.display().to_string()]).unwrap();
    assert_eq!(s.workshop().cameras("Shop").unwrap().len(), 2);
    assert_eq!(s.undo_depth(), depth, "camera changes are not undoable");
    let _ = std::fs::remove_file(path);
}

#[test]
fn calibrate_align_and_write_an_overlay_from_picture_files() {
    let mut s = session();
    s.run_line("camera.add Top simulated").unwrap();
    let mut truth = Lens::ideal(Model::Standard, 480, 360, 330.0);
    truth.k = vec![-0.3, 0.1];
    let pose = Pose::overhead(150.0, 100.0, 300.0, [0.01, -0.02, 0.02]);
    let camera = Renderer::new(&truth, &pose);
    let mut pictures = Vec::new();
    for (k, at) in [
        Point::new(-15.0, -25.0),
        Point::new(155.0, -25.0),
        Point::new(155.0, 105.0),
        Point::new(-15.0, 105.0),
        Point::new(70.0, 40.0),
    ]
    .into_iter()
    .enumerate()
    {
        let pic = camera.render([60, 60, 60], |p| match sheet::dark_at(p - at) {
            Some(true) => Some([20, 20, 20]),
            Some(false) => Some([240, 240, 240]),
            None => Some([180, 160, 130]),
        });
        let path = temp(&format!("sheet{k}.png"));
        std::fs::write(&path, pic.encode_png()).unwrap();
        pictures.push(path.display().to_string());
    }
    let mut args = vec!["Top".to_owned(), "standard".to_owned()];
    args.extend(pictures.iter().cloned());
    let out = run(&mut s, "camera.calibrate", &args).unwrap();
    assert!(
        out.message
            .contains("Standard lens correction for \"Top\": fit"),
        "{}",
        out.message
    );
    // The pattern burned on the bed, as the camera sees it.
    let pattern = Pattern::default_for(300.0, 200.0);
    let burned_pic = camera.render([60, 60, 60], |p| {
        (p.x >= 0.0 && p.y >= 0.0 && p.x <= 300.0 && p.y <= 200.0).then(|| {
            if burned(&pattern, 300.0, 200.0, p) {
                [30, 20, 10]
            } else {
                [220, 200, 160]
            }
        })
    });
    let marks = temp("marks.png");
    std::fs::write(&marks, burned_pic.encode_png()).unwrap();
    run(
        &mut s,
        "camera.align_find",
        &["Top".into(), marks.display().to_string()],
    )
    .unwrap();
    let cam = s.workshop().camera("Shop", "Top").unwrap().clone();
    let alignment = cam.alignment().unwrap();
    let lens = cam.lens.as_ref().unwrap();
    for p in [
        Point::new(80.0, 60.0),
        Point::new(220.0, 140.0),
        Point::new(150.0, 100.0),
    ] {
        let px = truth.project(pose.ray(p).unwrap());
        let back = alignment.to_bed(lens, px).unwrap();
        assert!(back.distance(p) < 0.6, "{p:?} {back:?}");
    }
    let png = temp("overlay.png");
    let out = run(
        &mut s,
        "camera.overlay",
        &[
            "Top".into(),
            marks.display().to_string(),
            png.display().to_string(),
        ],
    )
    .unwrap();
    assert!(out.message.contains("600 × 400 pixels"), "{}", out.message);
    // Clicked points work too, and the order is checked.
    let raw = pattern
        .centers(300.0, 200.0)
        .map(|p| truth.project(pose.ray(p).unwrap()));
    let json = |pts: [Point; 4]| {
        format!(
            r#"{{"picture": [{}], "bed": [[45,30],[255,30],[255,170],[45,170]], "height": 3}}"#,
            pts.iter()
                .map(|p| format!("[{},{}]", p.x, p.y))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    run(&mut s, "camera.align", &["Top".into(), json(raw)]).unwrap();
    let names: Vec<String> = s
        .workshop()
        .camera("Shop", "Top")
        .unwrap()
        .alignments
        .iter()
        .map(|a| a.name.clone())
        .collect();
    assert_eq!(names, ["3 mm", "Bed"]);
    let swapped = [raw[1], raw[0], raw[2], raw[3]];
    let err = run(&mut s, "camera.align", &["Top".into(), json(swapped)]).unwrap_err();
    assert!(err.to_string().contains("not in order"), "{err}");
    s.run_line("camera.use_alignment Top Bed").unwrap();
    s.run_line("camera.remove_alignment Top \"3 mm\"").unwrap();
    assert_eq!(
        s.workshop().camera("Shop", "Top").unwrap().alignments.len(),
        1
    );
    // Another camera takes the lens correction from an exported file.
    let file = temp("top-lens.lccam");
    run(
        &mut s,
        "camera.export",
        &["Top".into(), file.display().to_string()],
    )
    .unwrap();
    s.run_line("camera.add Spare simulated").unwrap();
    run(
        &mut s,
        "camera.set_lens",
        &["Spare".into(), file.display().to_string()],
    )
    .unwrap();
    assert_eq!(
        s.workshop().camera("Shop", "Spare").unwrap().lens,
        s.workshop().camera("Shop", "Top").unwrap().lens
    );
    let _ = std::fs::remove_file(file);
    for p in pictures
        .iter()
        .chain([&marks.display().to_string(), &png.display().to_string()])
    {
        let _ = std::fs::remove_file(p);
    }
}

#[test]
fn a_camera_without_lens_correction_aligns_on_the_picture_as_it_is() {
    let mut s = session();
    s.run_line("camera.add Plain simulated").unwrap();
    let lens = Lens::ideal(Model::Standard, 400, 300, 330.0);
    let pose = Pose::overhead(150.0, 100.0, 300.0, [0.0, 0.0, 0.0]);
    let camera = Renderer::new(&lens, &pose);
    let pattern = Pattern::default_for(300.0, 200.0);
    let pic = camera.render([60, 60, 60], |p| {
        (p.x >= 0.0 && p.y >= 0.0 && p.x <= 300.0 && p.y <= 200.0).then(|| {
            if burned(&pattern, 300.0, 200.0, p) {
                [30, 20, 10]
            } else {
                [220, 200, 160]
            }
        })
    });
    let path = temp("plain.png");
    std::fs::write(&path, pic.encode_png()).unwrap();
    let file = path.display().to_string();
    run(&mut s, "camera.align_find", &["Plain".into(), file.clone()]).unwrap();
    let cam = s.workshop().camera("Shop", "Plain").unwrap().clone();
    assert!(cam.lens.is_none());
    let plain = cam.lens_or_plain(400, 300);
    let alignment = cam.alignment().unwrap();
    let p = Point::new(200.0, 50.0);
    let back = alignment
        .to_bed(&plain, lens.project(pose.ray(p).unwrap()))
        .unwrap();
    assert!(back.distance(p) < 0.6, "{back:?}");
    let png = temp("plain-overlay.png");
    run(
        &mut s,
        "camera.overlay",
        &["Plain".into(), file, png.display().to_string()],
    )
    .unwrap();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(png);
}

#[test]
fn the_pattern_burns_as_an_absolute_fill_job_and_frames_laser_off() {
    let mut s = session();
    let svg = temp("sheet.svg");
    run(&mut s, "camera.pattern", &[svg.display().to_string()]).unwrap();
    assert!(std::fs::read_to_string(&svg).unwrap().contains("<circle"));
    let burn = s.run_line("device.camera_pattern power=30").unwrap();
    assert!(
        burn.message.contains("210 × 140 mm, marks 12 mm"),
        "{}",
        burn.message
    );
    match burn.device {
        Some(Request::Stream { kind, gcode }) => {
            assert_eq!(kind, StreamKind::Job);
            assert!(
                gcode.contains("M4") && gcode.contains("S300"),
                "fill at 30%"
            );
        }
        other => panic!("{other:?}"),
    }
    match s.run_line("device.camera_pattern_frame").unwrap().device {
        Some(Request::Stream { kind, gcode }) => {
            assert_eq!(kind, StreamKind::Frame);
            assert!(!gcode.contains("S300"));
        }
        other => panic!("{other:?}"),
    }
    assert!(
        s.run_line(r#"device.camera_pattern "{\"width\": 500, \"height\": 100, \"mark\": 10}""#)
            .is_err()
    );
    let _ = std::fs::remove_file(svg);
}
