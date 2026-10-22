//! `camera`: the calibration sheet, lens fitting and overlays from picture
//! files, without a device profile (`docs/spec/camera.md`, CLI).

use std::path::Path;

use lasercraft_camera::align::Alignment;
use lasercraft_camera::calibrate::{View, calibrate};
use lasercraft_camera::dots::find_sheet;
use lasercraft_camera::lens::{Lens, Model};
use lasercraft_camera::overlay::{Fit, Overlay};
use lasercraft_camera::{Picture, sheet};

use crate::{CliError, failed, usage};

/// Runs `camera pattern|calibrate|overlay …`.
pub(crate) fn camera(args: &[String]) -> Result<String, CliError> {
    match args {
        [what, out] if what == "pattern" => {
            std::fs::write(out, sheet::svg())
                .map_err(|e| failed(format!("cannot write {out}: {e}")))?;
            Ok(format!("wrote the calibration sheet to {out}\n"))
        }
        [what, model, pictures @ ..] if what == "calibrate" => {
            let model = Model::parse(model)
                .ok_or_else(|| usage("the model is standard, wide or fisheye"))?;
            let mut views: Vec<View> = Vec::new();
            let mut size = (1, 1);
            let mut report = String::new();
            for path in pictures {
                let pic = Picture::load(Path::new(path)).map_err(|e| failed(e.0))?;
                size = (pic.width(), pic.height());
                match find_sheet(&pic) {
                    Ok(dots) => {
                        report.push_str(&format!("{path}: {} dots\n", dots.len()));
                        views.push(
                            dots.into_iter()
                                .map(|((i, j), p)| (sheet::point(i, j), p))
                                .collect(),
                        );
                    }
                    Err(e) => report.push_str(&format!("{path}: left out, {e}\n")),
                }
            }
            let lens = calibrate(model, size.0, size.1, &views)
                .map_err(|e| failed(format!("{report}{e}")))?;
            let json = serde_json::to_string_pretty(&lens).map_err(|e| failed(e.to_string()))?;
            Ok(format!(
                "{report}fit {:.0}% ({:.2} pixels)\n{json}\n",
                lens.score(),
                lens.rms
            ))
        }
        [what, lens, alignment, picture, out, rest @ ..] if what == "overlay" => {
            let read = |p: &String| {
                std::fs::read_to_string(p).map_err(|e| failed(format!("cannot read {p}: {e}")))
            };
            let lens: Lens = serde_json::from_str(&read(lens)?)
                .map_err(|e| failed(format!("not a lens correction: {e}")))?;
            let alignment: Alignment = serde_json::from_str(&read(alignment)?)
                .map_err(|e| failed(format!("not an alignment: {e}")))?;
            let (w, h) = match rest {
                [w, h] => (
                    w.parse().map_err(|_| usage("the bed width is a number"))?,
                    h.parse().map_err(|_| usage("the bed height is a number"))?,
                ),
                [] => (400.0, 400.0),
                _ => {
                    return Err(usage(
                        "camera overlay takes the bed width and height, or neither",
                    ));
                }
            };
            let pic = Picture::load(Path::new(picture)).map_err(|e| failed(e.0))?;
            let mut overlay = Overlay::empty(w, h);
            overlay.paint(&pic, &lens, &alignment, &Fit::default(), None);
            std::fs::write(out, overlay.encode_png())
                .map_err(|e| failed(format!("cannot write {out}: {e}")))?;
            Ok(format!(
                "wrote the overlay to {out}: {} × {} pixels\n",
                overlay.width, overlay.height
            ))
        }
        _ => Err(usage(
            "camera pattern <out.svg> | camera calibrate <model> <pictures…> | camera overlay <lens.json> <alignment.json> <picture> <out.png> [<bed_w> <bed_h>]",
        )),
    }
}
