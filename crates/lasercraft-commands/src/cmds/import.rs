//! `project.import` (`docs/spec/import.md`) and `shape.add_text`
//! (`docs/spec/text.md`).

use std::path::Path as FsPath;

use lasercraft_geom::Affine;
use lasercraft_import::{fonts, import_file, text};
use lasercraft_project::layer::nearest_color;
use lasercraft_project::{Geometry, Project};

use super::id_args;
use super::shape::add;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Invocation, Outcome};

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "project.import",
            "project.import <path> [@color]",
            "Import the vector art of an SVG or DXF file as paths.",
            import,
        ),
        cmd(
            "shape.add_text",
            "shape.add_text <x> <y> <height> <font> <text> [@color]",
            "Add text as outlines, its baseline starting at (x, y); height is the capital height in mm.",
            add_text,
        ),
    ]
}

fn import(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 1)?;
    let file = args.str(0)?;
    let forced = args.color()?;
    let imported = import_file(FsPath::new(file), project.work_area().height_mm)
        .map_err(|e| args.invalid(e.0))?;
    if imported.is_empty() {
        return Err(args.invalid(format!("nothing to import from {file}")));
    }
    let shapes: Vec<(Geometry, u8)> = imported
        .into_iter()
        .map(|i| {
            let color = forced.unwrap_or_else(|| i.color.map_or(0, nearest_color));
            (Geometry::Path(i.path), color)
        })
        .collect();
    for (g, _) in &shapes {
        g.validate()?;
    }
    let mut ids = Vec::with_capacity(shapes.len());
    for (g, color) in shapes {
        match project.add_shape_on(g, Affine::IDENTITY, color) {
            Ok(id) => ids.push(id),
            Err(err) => {
                for id in &ids {
                    let _ = project.remove_shape(*id);
                }
                return Err(err.into());
            }
        }
    }
    if ids.len() > 1 {
        let group = project.allocate_id();
        for id in &ids {
            project.set_shape_groups(*id, vec![group])?;
        }
    }
    project.set_selection(ids.iter().copied())?;
    Ok(Outcome::undoable(
        format!("imported {} shape(s) from {file}", ids.len()),
        Invocation::new("shape.delete", id_args(&ids)),
    ))
}

fn add_text(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(5, 5)?;
    let at = args.point(0)?;
    let height = args.f64(2, "height")?;
    let (data, index) = fonts::load(args.str(3)?).map_err(|e| args.invalid(e.0))?;
    let path = text::outline(&data, index, args.str(4)?, height).map_err(|e| args.invalid(e.0))?;
    add(
        project,
        args,
        Geometry::Path(path),
        Affine::translate(at.x, at.y),
    )
}

#[cfg(test)]
mod tests {
    use lasercraft_project::Project;

    use crate::{Invocation, Registry, Session};

    pub(crate) const FONT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/fixtures/lctest.ttf"
    );
    const SVG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/fixtures/m3-art.svg"
    );
    const DXF: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/fixtures/m3-plate.dxf"
    );

    fn session() -> Session {
        Session::new(Project::default(), Registry::with_builtin())
    }

    #[test]
    fn import_groups_selects_and_maps_colors() {
        let mut s = session();
        let out = s.run(&Invocation::new("project.import", [SVG])).unwrap();
        assert!(
            out.message.starts_with("imported 4 shape(s)"),
            "{}",
            out.message
        );
        let shapes = s.project().shapes();
        assert!(shapes.iter().all(|sh| sh.geometry().kind() == "path"));
        assert!(shapes.iter().all(|sh| sh.groups() == [5]));
        assert_eq!(s.project().selection(), [1, 2, 3, 4]);
        let layers: Vec<u8> = shapes
            .iter()
            .map(lasercraft_project::Shape::layer)
            .collect();
        assert_eq!(layers[0], 0, "black outline on layer 0");
        assert!(layers.iter().any(|l| *l != 0));
        s.run_line("edit.undo").unwrap();
        assert_eq!(s.project(), &Project::default());
        s.run(&Invocation::new("project.import", [SVG, "@7"]))
            .unwrap();
        assert!(s.project().shapes().iter().all(|sh| sh.layer() == 7));
        s.run(&Invocation::new("project.import", [DXF])).unwrap();
        assert!(s.project().shapes().len() > 4);
    }

    #[test]
    fn import_errors_change_nothing() {
        let mut s = session();
        for bad in ["/nonexistent.svg", "/tmp/whatever.pdf"] {
            assert!(s.run(&Invocation::new("project.import", [bad])).is_err());
        }
        let empty = std::env::temp_dir().join(format!("lc-empty-{}.svg", std::process::id()));
        std::fs::write(&empty, r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#).unwrap();
        let err = s
            .run(&Invocation::new(
                "project.import",
                [empty.to_str().unwrap()],
            ))
            .unwrap_err();
        assert!(err.to_string().contains("nothing to import"));
        std::fs::remove_file(&empty).unwrap();
        assert!(s.run_line("project.import").is_err());
        assert_eq!(s.project(), &Project::default());
        assert_eq!(s.undo_depth(), 0);
    }

    #[test]
    fn text_is_added_as_an_outline_path() {
        let mut s = session();
        s.run(&Invocation::new(
            "shape.add_text",
            ["10", "50", "7", FONT, "HOLA", "@3"],
        ))
        .unwrap();
        let shape = &s.project().shapes()[0];
        assert_eq!((shape.geometry().kind(), shape.layer()), ("path", 3));
        let b = shape.bounds();
        assert!(
            (b.max.y - 50.0).abs() < 1e-9 && (b.min.y - 43.0).abs() < 1e-9,
            "{b:?}"
        );
        s.run_line("edit.undo").unwrap();
        assert!(s.project().shapes().is_empty());
        for bad in [
            vec!["0", "0", "7", FONT, "   "],
            vec!["0", "0", "0", FONT, "H"],
            vec!["0", "0", "7", "No Such Font 987", "H"],
            vec!["0", "0", "7", FONT],
        ] {
            assert!(
                s.run(&Invocation::new("shape.add_text", bad.clone()))
                    .is_err(),
                "{bad:?}"
            );
        }
        assert!(s.project().shapes().is_empty());
    }
}
