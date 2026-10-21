//! Reading and writing `.lcproj` files.

use std::fs;
use std::path::Path;

use crate::{Project, ProjectError};

/// The `format` tag every project file carries.
pub const FORMAT: &str = "lasercraft-project";

/// The newest format version this build reads and the one it writes.
pub const FORMAT_VERSION: u32 = 3;

/// Just enough of a file to decide whether we can read the rest.
#[derive(serde::Deserialize)]
struct Header {
    format: Option<String>,
    version: Option<u32>,
}

impl Project {
    /// Serializes to compact, indented JSON ending in a newline, so files
    /// diff cleanly and golden tests compare exact text.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_value(self)
            .map(|v| crate::compact::to_string(&v))
            .unwrap_or_default()
    }

    /// Parses and validates project JSON.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::Format`] for anything that is not a valid
    /// project, [`ProjectError::UnsupportedVersion`] for files from a newer
    /// LaserCraft and [`ProjectError::InvalidValue`] for out-of-range values.
    pub fn from_json(text: &str) -> Result<Self, ProjectError> {
        let header: Header =
            serde_json::from_str(text).map_err(|e| ProjectError::Format(e.to_string()))?;
        if header.format.as_deref() != Some(FORMAT) {
            return Err(ProjectError::Format(format!(
                "missing `\"format\": \"{FORMAT}\"`"
            )));
        }
        match header.version {
            None | Some(0) => return Err(ProjectError::Format("missing version".to_owned())),
            Some(v) if v > FORMAT_VERSION => return Err(ProjectError::UnsupportedVersion(v)),
            Some(_) => {}
        }
        let mut project: Self =
            serde_json::from_str(text).map_err(|e| ProjectError::Format(e.to_string()))?;
        project.validate()?;
        // Older versions are upgraded in memory and saved as the current one.
        project.version = FORMAT_VERSION;
        Ok(project)
    }

    /// Reads a project file.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::Io`] if the file cannot be read, otherwise as
    /// [`Project::from_json`].
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let text = fs::read_to_string(path)
            .map_err(|e| ProjectError::Io(format!("cannot read {}: {e}", path.display())))?;
        Self::from_json(&text)
    }

    /// Writes a project file. The text goes to a temporary file next to the
    /// target that is then renamed over it, so a crash never leaves a
    /// half-written project.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::Io`] if writing fails.
    pub fn save(&self, path: &Path) -> Result<(), ProjectError> {
        let io =
            |e: std::io::Error| ProjectError::Io(format!("cannot write {}: {e}", path.display()));
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = std::path::PathBuf::from(tmp);
        fs::write(&tmp, self.to_json()).map_err(io)?;
        fs::rename(&tmp, path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            io(e)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Origin;

    const GOLDEN: &str = include_str!("../tests/golden/default.lcproj");
    const GOLDEN_SHAPES: &str = include_str!("../tests/golden/shapes.lcproj");
    const V1: &str = include_str!("../tests/golden/v1-default.lcproj");
    const V2_SHAPES: &str = include_str!("../tests/golden/v2-shapes.lcproj");

    #[test]
    fn default_project_matches_golden_file() {
        assert_eq!(Project::default().to_json(), GOLDEN);
    }

    #[test]
    fn shapes_match_golden_file() {
        let mut p = Project::default();
        p.add_shape(
            crate::Geometry::Rect {
                width: 20.0,
                height: 10.0,
            },
            lasercraft_geom::Affine::translate(5.0, 5.0),
        )
        .unwrap();
        p.add_shape(
            crate::Geometry::Ellipse { rx: 8.0, ry: 4.0 },
            lasercraft_geom::Affine::rotate_about(30.0, lasercraft_geom::Point::new(50.0, 50.0))
                * lasercraft_geom::Affine::translate(50.0, 50.0),
        )
        .unwrap();
        p.add_shape(
            crate::Geometry::Polyline {
                points: vec![
                    lasercraft_geom::Point::new(0.0, 0.0),
                    lasercraft_geom::Point::new(30.0, 0.0),
                    lasercraft_geom::Point::new(15.0, 20.0),
                ],
                closed: true,
            },
            lasercraft_geom::Affine::translate(100.0, 100.0),
        )
        .unwrap();
        assert_eq!(p.to_json(), GOLDEN_SHAPES);
        assert_eq!(Project::from_json(GOLDEN_SHAPES).unwrap(), p);
    }

    #[test]
    fn floats_round_trip_bit_exactly() {
        let mut p = Project::default();
        for degrees in [-12.0, 33.0, 0.1, 1.5, 89.99] {
            let t = lasercraft_geom::Affine::rotate_about(
                degrees,
                lasercraft_geom::Point::new(0.1, 0.7),
            ) * lasercraft_geom::Affine::scale(1.0 / 3.0, 0.3);
            p.add_shape(
                crate::Geometry::Ellipse {
                    rx: 0.1 + 0.2,
                    ry: 7.0 / 9.0,
                },
                t,
            )
            .unwrap();
        }
        assert_eq!(Project::from_json(&p.to_json()).unwrap(), p);
    }

    #[test]
    fn version_1_files_load_and_upgrade() {
        let p = Project::from_json(V1).unwrap();
        assert_eq!(p, Project::default());
        assert_eq!(p.to_json(), GOLDEN);
    }

    #[test]
    fn version_2_shapes_load_onto_layer_0() {
        let p = Project::from_json(V2_SHAPES).unwrap();
        assert_eq!(p.to_json(), GOLDEN_SHAPES);
        assert!(p.shapes().iter().all(|s| s.layer() == 0));
    }

    #[test]
    fn changed_layers_and_order_round_trip() {
        let mut p = Project::default();
        p.set_layer_setting(4, "mode", "fill").unwrap();
        p.set_layer_setting(4, "name", "Engrave").unwrap();
        let mut order = p.layer_order();
        order.swap(0, 4);
        p.set_layer_order(&order).unwrap();
        let text = p.to_json();
        assert_eq!(
            text.matches("\"color\":").count(),
            1,
            "only changed layers: {text}"
        );
        assert_eq!(Project::from_json(&text).unwrap(), p);
    }

    #[test]
    fn rejects_bad_layer_tables() {
        let order = "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,";
        for (from, to) in [
            (
                order,
                "0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,",
            ),
            (
                order,
                "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,",
            ),
            (
                order,
                "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 99,",
            ),
            (
                "\"changed\": []",
                "\"changed\": [{\"color\": 1, \"name\": \"x\", \"mode\": \"line\", \"speed\": -1.0}]",
            ),
        ] {
            let bad = GOLDEN.replace(from, to);
            assert_ne!(bad, GOLDEN);
            assert!(Project::from_json(&bad).is_err(), "{to}");
        }
    }

    #[test]
    fn rejects_duplicate_ids_and_stale_next_id() {
        let dup = GOLDEN_SHAPES.replace("\"id\": 2", "\"id\": 1");
        assert!(matches!(
            Project::from_json(&dup),
            Err(ProjectError::InvalidValue { .. })
        ));
        let stale = GOLDEN_SHAPES.replace("\"next_id\": 4", "\"next_id\": 3");
        assert!(matches!(
            Project::from_json(&stale),
            Err(ProjectError::InvalidValue { .. })
        ));
    }

    #[test]
    fn json_round_trips() {
        let mut p = Project::default();
        p.set_name("Coasters \"v2\"").unwrap();
        p.set_notes("line one\nline two").unwrap();
        p.set_work_area_size(600.5, 300.0).unwrap();
        p.set_origin(Origin::RearLeft);
        p.allocate_id();
        assert_eq!(Project::from_json(&p.to_json()).unwrap(), p);
    }

    #[test]
    fn rejects_other_formats_and_newer_versions() {
        assert!(matches!(
            Project::from_json("{}"),
            Err(ProjectError::Format(_))
        ));
        assert!(matches!(
            Project::from_json("not json"),
            Err(ProjectError::Format(_))
        ));
        let newer = GOLDEN.replace("\"version\": 3", "\"version\": 99");
        assert_eq!(
            Project::from_json(&newer),
            Err(ProjectError::UnsupportedVersion(99))
        );
        let unknown = GOLDEN.replace("\"name\"", "\"surprise\": 1,\n  \"name\"");
        assert!(matches!(
            Project::from_json(&unknown),
            Err(ProjectError::Format(_))
        ));
    }

    #[test]
    fn rejects_out_of_range_values() {
        let bad = GOLDEN.replace("\"width_mm\": 400.0", "\"width_mm\": -5.0");
        assert!(matches!(
            Project::from_json(&bad),
            Err(ProjectError::InvalidValue { .. })
        ));
        let empty_name = GOLDEN.replace("\"Untitled\"", "\"\"");
        assert!(Project::from_json(&empty_name).is_err());
    }

    #[test]
    fn save_and_load_through_a_file() {
        let dir = std::env::temp_dir().join(format!("lasercraft-project-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("job.lcproj");
        let mut p = Project::default();
        p.set_name("Saved").unwrap();
        p.save(&path).unwrap();
        assert_eq!(Project::load(&path).unwrap(), p);
        assert!(!dir.join("job.lcproj.tmp").exists());
        assert!(matches!(
            Project::load(&dir.join("missing.lcproj")),
            Err(ProjectError::Io(_))
        ));
        fs::remove_dir_all(&dir).unwrap();
    }
}
