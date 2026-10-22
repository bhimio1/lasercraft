//! Test pattern presets (`docs/spec/test-patterns.md`). Settings are kept
//! as JSON objects; the planner reads them, so this crate never needs to
//! know their fields.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::WorkshopError;

/// The pattern kinds.
pub const KINDS: [&str; 2] = ["speed_power", "interval"];

/// A named set of test settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    /// Shown in the Preset dropdown.
    pub name: String,
    /// `speed_power` or `interval`.
    pub kind: String,
    /// The settings object (missing keys take the pattern's defaults).
    pub settings: Value,
}

/// Checks a pattern kind.
///
/// # Errors
///
/// An unknown kind.
pub fn check_kind(kind: &str) -> Result<(), WorkshopError> {
    if KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(WorkshopError::new(format!(
            "unknown test `{kind}`; one of {}",
            KINDS.join(", ")
        )))
    }
}

/// LaserCraft's own built-in presets.
#[must_use]
pub fn built_in() -> Vec<Preset> {
    vec![
        Preset {
            name: "Diode engraving".to_owned(),
            kind: "speed_power".to_owned(),
            settings: json!({
                "rows": {"count": 10, "setting": "speed", "from": 20, "to": 200},
                "columns": {"count": 10, "setting": "power", "from": 10, "to": 60},
                "box": {"mode": "fill", "interval": 0.1}
            }),
        },
        Preset {
            name: "Diode cutting".to_owned(),
            kind: "speed_power".to_owned(),
            settings: json!({
                "rows": {"count": 10, "setting": "speed", "from": 1, "to": 10},
                "columns": {"count": 5, "setting": "power", "from": 60, "to": 100},
                "box": {"mode": "line", "passes": 2}
            }),
        },
    ]
}

/// The presets file in the configuration folder (`test-presets.json`) and
/// the `.lctests` exchange file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct PresetsFile {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub last: serde_json::Map<String, Value>,
}

/// `format` of the presets files.
pub(crate) const FORMAT: &str = "lasercraft-test-presets";
