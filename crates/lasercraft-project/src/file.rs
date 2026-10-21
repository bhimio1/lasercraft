//! Reading and writing `.lcproj` files.

use std::fs;
use std::path::Path;

use crate::{Project, ProjectError};

/// The `format` tag every project file carries.
pub const FORMAT: &str = "lasercraft-project";

/// The newest format version this build reads and the one it writes.
pub const FORMAT_VERSION: u32 = 1;

/// Just enough of a file to decide whether we can read the rest.
#[derive(serde::Deserialize)]
struct Header {
    format: Option<String>,
    version: Option<u32>,
}

impl Project {
    /// Serializes to pretty-printed JSON ending in a newline, so files diff
    /// cleanly and golden tests compare exact text.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
        text.push('\n');
        text
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
        let project: Self =
            serde_json::from_str(text).map_err(|e| ProjectError::Format(e.to_string()))?;
        project.validate()?;
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

    #[test]
    fn default_project_matches_golden_file() {
        assert_eq!(Project::default().to_json(), GOLDEN);
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
        let newer = GOLDEN.replace("\"version\": 1", "\"version\": 99");
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
