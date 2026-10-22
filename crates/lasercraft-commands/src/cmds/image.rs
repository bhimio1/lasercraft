//! Images: PNG/JPEG import and `image.adjust` (`docs/spec/images.md`).

use std::path::Path as FsPath;

use lasercraft_geom::Affine;
use lasercraft_project::raster::{Adjust, decode};
use lasercraft_project::{Geometry, Image, Project};

use super::export::{key_value, number};
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

/// Resolution assumed when a file states none: 254 DPI, 0.1 mm per pixel.
pub const DEFAULT_MM_PER_PIXEL: f64 = 0.1;

/// File extensions imported as images.
pub const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "image.adjust",
            "image.adjust <id> [brightness=<-100..100>] [contrast=<-99..99>] [gamma=<0.1..10>]",
            "Change an image's brightness, contrast or gamma.",
            adjust,
        ),
        cmd(
            "image.insert_picture",
            "image.insert_picture <path> <x> <y> <w> <h> [@layer]",
            "Add a picture file as an image covering a bed rectangle (camera Insert picture).",
            insert_picture,
        ),
    ]
}

/// Adds a picture covering the rectangle `x, y, w, h` (document mm).
fn insert_picture(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(5, 6)?;
    let file = args.str(0)?;
    let at = args.point(1)?;
    let (w, h) = (args.f64(3, "width")?, args.f64(4, "height")?);
    if !(w > 0.0 && h > 0.0) {
        return Err(args.invalid("the picture needs a positive size"));
    }
    let bytes =
        std::fs::read(file).map_err(|e| args.invalid(format!("cannot read {file}: {e}")))?;
    let gray = decode(&bytes)
        .map_err(|e| args.invalid(format!("{file}: {e}")))?
        .gray;
    let color = args.color()?.unwrap_or(0);
    let id = project.add_shape_on(
        Geometry::Image(Image::new(w, h, gray)),
        Affine::translate(at.x, at.y),
        color,
    )?;
    project.replace_selection([id])?;
    Ok(Outcome::undoable(
        format!(
            "inserted the picture as image {id}: {} × {} mm",
            lasercraft_plan::num(w),
            lasercraft_plan::num(h)
        ),
        Invocation::new("shape.delete", [id.to_string()]),
    ))
}

/// Whether `path` names a PNG or JPEG file.
pub(super) fn is_image_file(path: &str) -> bool {
    FsPath::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

/// The image's size in mm: from the file's resolution or the default, shrunk
/// (keeping its proportions) to fit the work area.
fn size_mm(pixels: [u32; 2], per_mm: Option<[f64; 2]>, area: [f64; 2]) -> [f64; 2] {
    let [px, py] = per_mm.unwrap_or([1.0 / DEFAULT_MM_PER_PIXEL; 2]);
    let w = f64::from(pixels[0]) / px;
    let h = f64::from(pixels[1]) / py;
    let fit = (area[0] / w).min(area[1] / h).min(1.0);
    [w * fit, h * fit]
}

/// Imports a PNG or JPEG as one image shape at the document origin.
pub(super) fn import_image(
    project: &mut Project,
    args: &Args<'_>,
    file: &str,
) -> Result<Outcome, CommandError> {
    let bytes =
        std::fs::read(file).map_err(|e| args.invalid(format!("cannot read {file}: {e}")))?;
    let decoded = decode(&bytes).map_err(|e| args.invalid(format!("{file}: {e}")))?;
    let gray = decoded.gray;
    let area = project.work_area();
    let [w, h] = size_mm(
        [gray.width(), gray.height()],
        decoded.pixels_per_mm,
        [area.width_mm, area.height_mm],
    );
    let (px_w, px_h) = (gray.width(), gray.height());
    let color = args.color()?.unwrap_or(0);
    let geometry = Geometry::Image(Image::new(w, h, gray));
    let id = project.add_shape_on(geometry, Affine::IDENTITY, color)?;
    project.replace_selection([id])?;
    Ok(Outcome::undoable(
        format!(
            "imported {file} as image {id}: {px_w} × {px_h} pixels, {} × {} mm",
            lasercraft_plan::num(w),
            lasercraft_plan::num(h)
        ),
        Invocation::new("shape.delete", [id.to_string()]),
    ))
}

fn adjust(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(2, 4)?;
    let id = args.parse::<u32>(0, "shape id")?;
    let image = project
        .shape(id)
        .ok_or_else(|| args.invalid(format!("shape {id} does not exist")))?
        .geometry()
        .image()
        .ok_or_else(|| args.invalid(format!("shape {id} is not an image")))?
        .clone();
    let old = image.adjust;
    let mut new = old;
    for i in 1..args.len() {
        let (key, value) = key_value(args, args.str(i)?)?;
        let v = number(args, key, value)?;
        match key {
            "brightness" => new.brightness = v,
            "contrast" => new.contrast = v,
            "gamma" => new.gamma = v,
            _ => return Err(args.invalid(format!("unknown adjustment `{key}`"))),
        }
    }
    project.set_shape_geometry(id, Geometry::Image(image.with_adjust(new)))?;
    let text = |a: Adjust| {
        [
            id.to_string(),
            format!("brightness={}", a.brightness),
            format!("contrast={}", a.contrast),
            format!("gamma={}", a.gamma),
        ]
    };
    Ok(Outcome::undoable(
        format!(
            "image {id}: brightness {}, contrast {}, gamma {}",
            new.brightness, new.contrast, new.gamma
        ),
        Invocation::new("image.adjust", text(old)),
    ))
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::Point;
    use lasercraft_project::raster::{Gray, encode_png};

    use crate::{Registry, Session};

    use super::*;

    fn png_file(tag: &str, w: u32, h: u32) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lasercraft-image-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pic.png");
        let px = (0..w * h).map(|i| (i * 255 / (w * h)) as u8).collect();
        std::fs::write(&path, encode_png(&Gray::new(w, h, px).unwrap())).unwrap();
        path
    }

    #[test]
    fn sizes_come_from_the_file_or_the_default_and_fit_the_bed() {
        assert_eq!(size_mm([100, 50], None, [400.0, 400.0]), [10.0, 5.0]);
        assert_eq!(
            size_mm([300, 300], Some([300.0 / 25.4; 2]), [400.0, 400.0]),
            [25.4, 25.4]
        );
        assert_eq!(size_mm([8000, 2000], None, [400.0, 400.0]), [400.0, 100.0]);
        assert!(is_image_file("a/b.PNG") && is_image_file("x.jpeg") && !is_image_file("x.svg"));
    }

    #[test]
    fn import_adjust_undo_and_vector_refusals() {
        let path = png_file("import", 40, 20);
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        let out = s
            .run(&Invocation::new(
                "project.import",
                [path.display().to_string(), "@4".into()],
            ))
            .unwrap();
        assert!(
            out.message.contains("40 × 20 pixels, 4 × 2 mm"),
            "{}",
            out.message
        );
        let id = s.project().selection()[0];
        let shape = s.project().shape(id).unwrap();
        assert_eq!((shape.layer(), shape.geometry().kind()), (4, "image"));

        s.run_line(&format!("image.adjust {id} brightness=10 gamma=2"))
            .unwrap();
        let adjust = |s: &Session| {
            s.project()
                .shape(id)
                .unwrap()
                .geometry()
                .image()
                .unwrap()
                .adjust
        };
        assert_eq!((adjust(&s).brightness, adjust(&s).gamma), (10.0, 2.0));
        s.run_line("edit.undo").unwrap();
        assert!(adjust(&s).is_identity());
        for bad in [
            format!("image.adjust {id} gamma=0"),
            format!("image.adjust {id} contrast=100"),
            format!("image.adjust {id} sharpness=1"),
            format!("image.adjust {id}"),
            "image.adjust 99 gamma=1".to_owned(),
        ] {
            assert!(s.run_line(&bad).is_err(), "{bad}");
        }
        s.run_line("shape.add_rect 0 0 5 5").unwrap();
        let rect = s.project().selection()[0];
        assert!(s.run_line(&format!("image.adjust {rect} gamma=1")).is_err());
        for line in [
            format!("path.union {id} {rect}"),
            format!("path.weld {id} {rect}"),
            format!("path.offset 1 {id}"),
            format!("path.join {id} {rect}"),
        ] {
            let err = s.run_line(&line).unwrap_err().to_string();
            assert!(err.contains("is an image"), "{line}: {err}");
        }
        assert!(s.run_line(&format!("shape.to_path {id}")).is_err());
        // Undo of a delete brings the pixels back.
        s.run_line(&format!("shape.delete {id}")).unwrap();
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project().shape(id).unwrap().geometry().kind(), "image");
        let mut empty = Session::new(Project::default(), Registry::with_builtin());
        let missing = path.with_file_name("missing.png");
        assert!(
            empty
                .run(&Invocation::new(
                    "project.import",
                    [missing.display().to_string()]
                ))
                .is_err()
        );
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn origin_placement() {
        let path = png_file("origin", 10, 10);
        let mut s = Session::new(Project::default(), Registry::with_builtin());
        s.run(&Invocation::new(
            "project.import",
            [path.display().to_string()],
        ))
        .unwrap();
        let id = s.project().selection()[0];
        let b = s.project().shape(id).unwrap().bounds();
        assert_eq!((b.min, b.max), (Point::ZERO, Point::new(1.0, 1.0)));
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
