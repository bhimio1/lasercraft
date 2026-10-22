//! The LaserCraft document model and its `.lcproj` file format (JSON).
//!
//! Only `lasercraft-commands` mutates a [`Project`]; every setter validates its
//! input and leaves the project unchanged on error. The format is described in
//! `docs/spec/project-file.md`.

mod compact;
mod error;
mod file;
mod image;
pub mod layer;
mod layers;
mod shape;
mod shapes;
mod work_area;

pub use error::ProjectError;
pub use file::{FORMAT, FORMAT_VERSION};
pub use image::Image;
pub use lasercraft_raster as raster;
pub use layer::{Color, Dither, Follows, ImageSettings, LAYER_COUNT, Layer, Mode};
pub use shape::{
    Geometry, MAX_COORD_MM, MAX_POINTS, Shape, snap_points, validate_groups, validate_transform,
};
pub use shapes::{BoxMode, ObjectKey, ViewState};
pub use work_area::{MAX_WORK_AREA_MM, MIN_WORK_AREA_MM, Origin, WorkArea};

use serde::{Deserialize, Serialize};

/// Longest allowed project name, in characters.
pub const MAX_NAME_CHARS: usize = 200;
/// Longest device profile name (`docs/spec/devices.md`).
pub const MAX_DEVICE_CHARS: usize = 60;

/// Longest allowed notes text, in characters.
pub const MAX_NOTES_CHARS: usize = 20_000;

/// A LaserCraft document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    format: String,
    version: u32,
    name: String,
    #[serde(default)]
    notes: String,
    work_area: WorkArea,
    /// The device profile the project is for (M8, `docs/spec/devices.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device: Option<String>,
    next_id: u32,
    /// Settings for all 30 layers, in output order.
    #[serde(default)]
    layers: layer::LayerTable,
    /// Shapes in drawing order: later shapes are drawn on top.
    #[serde(default)]
    shapes: Vec<Shape>,
    /// Selection and clipboard. Not saved, not undone, not compared.
    #[serde(skip)]
    view: ViewState,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            format: FORMAT.to_owned(),
            version: FORMAT_VERSION,
            name: "Untitled".to_owned(),
            notes: String::new(),
            work_area: WorkArea::default(),
            device: None,
            next_id: 1,
            layers: layer::LayerTable::default(),
            shapes: Vec::new(),
            view: ViewState::default(),
        }
    }
}

impl Project {
    /// A new, empty project with the given work area.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an out-of-range size.
    pub fn with_work_area(width_mm: f64, height_mm: f64) -> Result<Self, ProjectError> {
        let mut project = Self::default();
        project.set_work_area_size(width_mm, height_mm)?;
        Ok(project)
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Renames the project.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an empty or overlong name.
    pub fn set_name(&mut self, name: &str) -> Result<(), ProjectError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProjectError::invalid("name", "must not be empty"));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(ProjectError::invalid(
                "name",
                format!("must be at most {MAX_NAME_CHARS} characters"),
            ));
        }
        name.clone_into(&mut self.name);
        Ok(())
    }

    /// Free-form notes stored with the project.
    #[must_use]
    pub fn notes(&self) -> &str {
        &self.notes
    }

    /// Replaces the notes.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for overlong text.
    pub fn set_notes(&mut self, notes: &str) -> Result<(), ProjectError> {
        if notes.chars().count() > MAX_NOTES_CHARS {
            return Err(ProjectError::invalid(
                "notes",
                format!("must be at most {MAX_NOTES_CHARS} characters"),
            ));
        }
        notes.clone_into(&mut self.notes);
        Ok(())
    }

    /// The machine's work area that jobs must fit in.
    #[must_use]
    pub fn work_area(&self) -> &WorkArea {
        &self.work_area
    }

    /// Resizes the work area.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an out-of-range size.
    pub fn set_work_area_size(
        &mut self,
        width_mm: f64,
        height_mm: f64,
    ) -> Result<(), ProjectError> {
        self.work_area = WorkArea::new(width_mm, height_mm, self.work_area.origin)?;
        Ok(())
    }

    /// Sets the bed corner that is the machine's origin.
    pub fn set_origin(&mut self, origin: Origin) {
        self.work_area.origin = origin;
    }

    /// The device profile the project is for, by name.
    #[must_use]
    pub fn device(&self) -> Option<&str> {
        self.device.as_deref()
    }

    /// Names the project's device profile, or none.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an empty or overlong name.
    pub fn set_device(&mut self, name: Option<&str>) -> Result<(), ProjectError> {
        if let Some(name) = name {
            let name = name.trim();
            if name.is_empty() || name.chars().count() > MAX_DEVICE_CHARS {
                return Err(ProjectError::invalid(
                    "device",
                    format!("must be 1–{MAX_DEVICE_CHARS} characters"),
                ));
            }
        }
        self.device = name.map(|n| n.trim().to_owned());
        Ok(())
    }

    /// The identifier the next created object will get.
    #[must_use]
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// Moves the identifier allocator. Used by redo so recreated objects get
    /// the identifiers later history entries refer to.
    pub fn set_next_id(&mut self, next_id: u32) {
        self.next_id = next_id.max(1);
    }

    /// Hands out a fresh identifier.
    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Checks invariants that a hand-edited or damaged file might break.
    fn validate(&self) -> Result<(), ProjectError> {
        let mut copy = Self::default();
        copy.set_name(&self.name)?;
        copy.set_notes(&self.notes)?;
        copy.set_device(self.device.as_deref())?;
        WorkArea::new(
            self.work_area.width_mm,
            self.work_area.height_mm,
            self.work_area.origin,
        )?;
        if self.next_id == 0 {
            return Err(ProjectError::invalid("next_id", "must be at least 1"));
        }
        self.validate_layers()?;
        self.validate_shapes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_trims_and_validates() {
        let mut p = Project::default();
        p.set_name("  Coasters  ").unwrap();
        assert_eq!(p.name(), "Coasters");
        assert!(p.set_name("   ").is_err());
        assert!(p.set_name(&"x".repeat(MAX_NAME_CHARS + 1)).is_err());
        assert_eq!(p.name(), "Coasters");
    }

    #[test]
    fn the_device_name_round_trips_and_is_left_out_when_none() {
        let mut p = Project::default();
        assert!(!p.to_json().contains("device"));
        p.set_device(Some(" Shop diode ")).unwrap();
        assert_eq!(p.device(), Some("Shop diode"));
        let back = Project::from_json(&p.to_json()).unwrap();
        assert_eq!(back.device(), Some("Shop diode"));
        assert!(p.set_device(Some("  ")).is_err());
        p.set_device(None).unwrap();
        assert_eq!(p.device(), None);
    }

    #[test]
    fn notes_have_a_length_limit() {
        let mut p = Project::default();
        p.set_notes("3 mm birch").unwrap();
        assert!(p.set_notes(&"x".repeat(MAX_NOTES_CHARS + 1)).is_err());
        assert_eq!(p.notes(), "3 mm birch");
    }

    #[test]
    fn work_area_resize_keeps_origin() {
        let mut p = Project::default();
        p.set_origin(Origin::RearRight);
        p.set_work_area_size(600.0, 400.0).unwrap();
        assert_eq!(p.work_area().origin, Origin::RearRight);
        assert!(p.set_work_area_size(0.0, 400.0).is_err());
        assert_eq!(p.work_area().width_mm, 600.0);
    }

    #[test]
    fn ids_count_up_and_never_reach_zero() {
        let mut p = Project::default();
        assert_eq!(p.allocate_id(), 1);
        assert_eq!(p.allocate_id(), 2);
        p.set_next_id(0);
        assert_eq!(p.next_id(), 1);
    }
}
