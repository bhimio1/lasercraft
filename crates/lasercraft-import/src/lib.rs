//! SVG and DXF import and font outlines for LaserCraft
//! (`docs/spec/import.md`, `docs/spec/text.md`).
//!
//! Importers produce geometry only, in document millimetres (x right, y
//! down); adding it to a project goes through a command in
//! `lasercraft-commands`. Everything here is deterministic for a given
//! input file.

pub mod dxf;
pub mod fonts;
pub mod svg;
pub mod text;

use std::fmt;
use std::path::Path as FsPath;

use lasercraft_geom::Path;

/// Why an import or font operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(pub String);

impl ImportError {
    pub(crate) fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ImportError {}

/// One imported element: its outline in document millimetres and its
/// color in the file, if it had a plain one.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    /// The outline.
    pub path: Path,
    /// sRGB color from the file.
    pub color: Option<[u8; 3]>,
}

/// Reads an SVG or DXF file, chosen by extension. `bed_height_mm` places
/// DXF drawings (their y axis points up from the front of the bed).
///
/// # Errors
///
/// Returns an [`ImportError`] for unreadable, unsupported or malformed files.
pub fn import_file(path: &FsPath, bed_height_mm: f64) -> Result<Vec<Imported>, ImportError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let read = || {
        std::fs::read(path)
            .map_err(|e| ImportError::new(format!("cannot read {}: {e}", path.display())))
    };
    match ext.as_str() {
        "svg" => svg::parse(&read()?),
        "dxf" => dxf::parse(&read()?, bed_height_mm),
        _ => Err(ImportError::new(format!(
            "unsupported file type `{}` (SVG and DXF can be imported)",
            path.display()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_extensions_and_missing_files_fail() {
        let err = import_file(FsPath::new("art.pdf"), 400.0).unwrap_err();
        assert!(err.to_string().contains("unsupported file type"));
        let err = import_file(FsPath::new("/nonexistent/art.svg"), 400.0).unwrap_err();
        assert!(err.to_string().contains("cannot read"));
    }
}
