//! Finding fonts: by file path, or by family name among the system's fonts
//! (`docs/spec/text.md`).

use std::sync::OnceLock;

use fontdb::{Database, Family, Query, Stretch, Style, Weight};

use crate::ImportError;

/// Families tried, in order, for the text tool's default font.
pub const PREFERRED_FAMILIES: [&str; 4] = ["DejaVu Sans", "Liberation Sans", "Arial", "Helvetica"];

fn database() -> &'static Database {
    static DB: OnceLock<Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        db
    })
}

/// Whether `font` names a file rather than a family.
#[must_use]
pub fn is_path(font: &str) -> bool {
    let lower = font.to_ascii_lowercase();
    font.contains('/')
        || font.contains('\\')
        || [".ttf", ".otf", ".ttc"].iter().any(|e| lower.ends_with(e))
}

/// The font data and face index for a family name or a font file path.
///
/// # Errors
///
/// Fails when the file cannot be read or the family is not installed.
pub fn load(font: &str) -> Result<(Vec<u8>, u32), ImportError> {
    if is_path(font) {
        let data = std::fs::read(font)
            .map_err(|e| ImportError::new(format!("cannot read font {font}: {e}")))?;
        return Ok((data, 0));
    }
    let db = database();
    let wanted = font.trim().to_lowercase();
    let name = db
        .faces()
        .flat_map(|f| f.families.iter().map(|(n, _)| n))
        .find(|n| n.to_lowercase() == wanted)
        .ok_or_else(|| ImportError::new(format!("font `{font}` is not installed")))?;
    let families = [Family::Name(name)];
    let id = db
        .query(&Query {
            families: &families,
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            style: Style::Normal,
        })
        .ok_or_else(|| ImportError::new(format!("font `{font}` is not installed")))?;
    db.with_face_data(id, |data, index| (data.to_vec(), index))
        .ok_or_else(|| ImportError::new(format!("cannot read font `{font}`")))
}

/// Installed family names, sorted and without duplicates.
#[must_use]
pub fn families() -> Vec<String> {
    let mut names: Vec<String> = database()
        .faces()
        .filter_map(|f| f.families.first().map(|(n, _)| n.clone()))
        .collect();
    names.sort_unstable_by_key(|n| n.to_lowercase());
    names.dedup();
    names
}

/// The text tool's default family: the first installed of
/// [`PREFERRED_FAMILIES`], else the first installed family.
#[must_use]
pub fn default_family() -> Option<String> {
    let installed = families();
    PREFERRED_FAMILIES
        .iter()
        .find_map(|p| {
            installed
                .iter()
                .find(|n| n.eq_ignore_ascii_case(p))
                .cloned()
        })
        .or_else(|| installed.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_recognized_and_read() {
        assert!(is_path("fonts/a.ttf") && is_path("A.OTF") && !is_path("DejaVu Sans"));
        let fixture = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/fixtures/lctest.ttf"
        );
        let (data, index) = load(fixture).unwrap();
        assert_eq!((data.len(), index), (1420, 0));
        assert!(load("/nonexistent/font.ttf").is_err());
    }

    #[test]
    fn families_resolve_when_fonts_are_installed() {
        assert!(load("No Such Family 12345").is_err());
        let Some(family) = default_family() else {
            eprintln!("skipped: no fonts installed on this machine");
            return;
        };
        let (data, _) = load(&family).unwrap();
        assert!(!data.is_empty());
        assert!(load(&family.to_uppercase()).is_ok(), "case-insensitive");
    }
}
