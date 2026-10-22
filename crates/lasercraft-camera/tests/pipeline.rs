//! The whole camera setup on a synthetic camera: five pictures of the
//! calibration sheet lying on the bed, the lens fit, the burned alignment
//! marks, and an overlay that puts bed features where they are.

use lasercraft_camera::align::{Alignment, fit};
use lasercraft_camera::calibrate::{View, calibrate};
use lasercraft_camera::dots::find_sheet;
use lasercraft_camera::lens::{Lens, Model};
use lasercraft_camera::marks::{Pattern, burned, find_marks};
use lasercraft_camera::overlay::{Fit, Overlay};
use lasercraft_camera::sheet;
use lasercraft_camera::synth::{Pose, Renderer};
use lasercraft_geom::Point;

const BED: (f64, f64) = (300.0, 200.0);

fn camera() -> (Lens, Pose) {
    let mut lens = Lens::ideal(Model::Standard, 640, 480, 430.0);
    lens.k = vec![-0.32, 0.11];
    lens.p = vec![0.0006, -0.0003];
    lens.cx = 323.0;
    lens.cy = 236.0;
    (
        lens,
        Pose::overhead(150.0, 100.0, 300.0, [0.02, -0.015, 0.03]),
    )
}

fn on_bed(p: Point) -> bool {
    p.x >= 0.0 && p.y >= 0.0 && p.x <= BED.0 && p.y <= BED.1
}

/// A bed with 20 mm stripes, so the overlay can be checked anywhere.
fn stripes(p: Point) -> [u8; 3] {
    if (p.x / 20.0).floor() as i64 % 2 == 0 {
        [230, 210, 170]
    } else {
        [170, 200, 230]
    }
}

#[test]
fn calibrate_align_and_overlay_end_to_end() {
    let (truth, pose) = camera();
    let camera = Renderer::new(&truth, &pose);
    // The sheet in the four corners and the center of the picture.
    let mut views: Vec<View> = Vec::new();
    for (at, angle) in [
        (Point::new(-15.0, -25.0), 0.05),
        (Point::new(155.0, -25.0), -0.05),
        (Point::new(155.0, 105.0), 0.0),
        (Point::new(-15.0, 105.0), 0.1),
        (Point::new(70.0, 40.0), -0.1),
    ] {
        let picture = camera.render([70, 70, 70], |p| {
            let d = p - at;
            let (s, c) = f64::sin_cos(angle);
            let local = Point::new(c * d.x + s * d.y, -s * d.x + c * d.y);
            match sheet::dark_at(local) {
                Some(true) => Some([25, 25, 25]),
                Some(false) => Some([240, 240, 236]),
                None => on_bed(p).then(|| stripes(p)),
            }
        });
        let dots = find_sheet(&picture).unwrap();
        assert!(dots.len() >= 30, "{}", dots.len());
        views.push(
            dots.into_iter()
                .map(|((i, j), px)| (sheet::point(i, j), px))
                .collect(),
        );
    }
    let lens = calibrate(Model::Standard, 640, 480, &views).unwrap();
    assert!(lens.rms < 0.5, "{}", lens.rms);

    // Burn the marks, find them, align.
    let pattern = Pattern::default_for(BED.0, BED.1);
    let picture = camera.render([70, 70, 70], |p| {
        on_bed(p).then(|| {
            if burned(&pattern, BED.0, BED.1, p) {
                [30, 20, 10]
            } else {
                stripes(p)
            }
        })
    });
    let raw = find_marks(&picture).unwrap();
    let matrix = fit(&lens, &raw, &pattern.centers(BED.0, BED.1)).unwrap();
    let alignment = Alignment {
        name: "bed".into(),
        height: 0.0,
        matrix,
        lens: lens.created.clone(),
        width: 640,
        height_px: 480,
        created: String::new(),
    };
    // Every bed point inside the marks maps back to itself within 0.5 mm.
    let mut worst: f64 = 0.0;
    for y in (40..=160).step_by(20) {
        for x in (50..=250).step_by(25) {
            let p = Point::new(f64::from(x), f64::from(y));
            let px = truth.project(pose.ray(p).unwrap());
            worst = worst.max(alignment.to_bed(&lens, px).unwrap().distance(p));
        }
    }
    assert!(worst < 0.5, "worst error {worst} mm");

    // The overlay shows each stripe edge where it is on the bed.
    let plain = camera.render([70, 70, 70], |p| on_bed(p).then(|| stripes(p)));
    let mut overlay = Overlay::empty(BED.0, BED.1);
    overlay.paint(&plain, &lens, &alignment, &Fit::default(), None);
    for x in [60.0, 100.0, 140.0, 200.0, 240.0] {
        for y in [50.0, 100.0, 150.0] {
            let before = overlay.at(Point::new(x - 1.5, y)).unwrap();
            let after = overlay.at(Point::new(x + 1.5, y)).unwrap();
            assert!(
                before != after,
                "edge at x {x}, y {y}: {before:?} {after:?}"
            );
        }
    }
}
