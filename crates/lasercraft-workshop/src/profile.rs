//! Device profiles (`docs/spec/devices.md`).

use std::fmt::Write as _;

use lasercraft_project::{Origin, WorkArea};
use serde::{Deserialize, Serialize};

use crate::WorkshopError;

/// Longest profile name.
pub const MAX_NAME_CHARS: usize = lasercraft_project::MAX_DEVICE_CHARS;
/// Valid top speeds, mm/s.
pub const MAX_SPEED_RANGE: (f64, f64) = (1.0, 10_000.0);
/// Valid accelerations, mm/s².
pub const ACCELERATION_RANGE: (f64, f64) = (1.0, 100_000.0);
/// Valid full-power S values (GRBL's `$30`).
pub const S_MAX_RANGE: (f64, f64) = (1.0, 100_000.0);
/// Valid baud rates.
pub const BAUD_RANGE: (u32, u32) = (1, 10_000_000);

/// One machine: its bed, its connection and its limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceProfile {
    /// Shown in lists; unique, case-insensitive.
    pub name: String,
    /// Work area width, mm.
    pub width: f64,
    /// Work area height, mm.
    pub height: f64,
    /// Machine origin corner.
    pub origin: Origin,
    /// Serial port; empty means "choose in the Machine panel".
    #[serde(default)]
    pub port: String,
    /// Baud rate.
    pub baud: u32,
    /// Top speed, mm/s.
    pub max_speed: f64,
    /// Acceleration, mm/s².
    pub acceleration: f64,
    /// Device power ceiling, %.
    pub max_power: f64,
    /// Full-power S value; must equal GRBL's `$30`.
    pub s_max: f64,
    /// Air assist code, `m7` or `m8`.
    pub air: String,
}

/// The keys `profile.set` accepts, in display order.
pub const KEYS: [&str; 10] = [
    "width",
    "height",
    "origin",
    "port",
    "baud",
    "max_speed",
    "acceleration",
    "max_power",
    "s_max",
    "air",
];

fn range(key: &str, value: f64, (min, max): (f64, f64)) -> Result<(), WorkshopError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(WorkshopError::new(format!(
            "{key} must be between {min} and {max}, got {value}"
        )))
    }
}

fn number(key: &str, value: &str) -> Result<f64, WorkshopError> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| WorkshopError::new(format!("{key} must be a number, not `{value}`")))
}

/// Checks a profile name.
///
/// # Errors
///
/// An empty or overlong name.
pub fn check_name(name: &str) -> Result<String, WorkshopError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(WorkshopError::new(format!(
            "a device profile name must be 1–{MAX_NAME_CHARS} characters"
        )));
    }
    Ok(name.to_owned())
}

impl DeviceProfile {
    /// A profile with the default values of `docs/spec/devices.md`.
    ///
    /// # Errors
    ///
    /// An invalid name.
    pub fn new(name: &str) -> Result<Self, WorkshopError> {
        let area = WorkArea::default();
        Ok(Self {
            name: check_name(name)?,
            width: area.width_mm,
            height: area.height_mm,
            origin: area.origin,
            port: String::new(),
            baud: 115_200,
            max_speed: 100.0,
            acceleration: 1000.0,
            max_power: 100.0,
            s_max: 1000.0,
            air: "m8".to_owned(),
        })
    }

    /// The work area the profile describes.
    #[must_use]
    pub fn work_area(&self) -> WorkArea {
        WorkArea {
            width_mm: self.width,
            height_mm: self.height,
            origin: self.origin,
        }
    }

    /// Checks every field.
    ///
    /// # Errors
    ///
    /// Names the first field out of range.
    pub fn validate(&self) -> Result<(), WorkshopError> {
        check_name(&self.name)?;
        WorkArea::new(self.width, self.height, self.origin)
            .map_err(|e| WorkshopError::new(e.to_string()))?;
        if !(BAUD_RANGE.0..=BAUD_RANGE.1).contains(&self.baud) {
            return Err(WorkshopError::new(format!(
                "baud must be between {} and {}",
                BAUD_RANGE.0, BAUD_RANGE.1
            )));
        }
        range("max_speed", self.max_speed, MAX_SPEED_RANGE)?;
        range("acceleration", self.acceleration, ACCELERATION_RANGE)?;
        range("max_power", self.max_power, (0.0, 100.0))?;
        range("s_max", self.s_max, S_MAX_RANGE)?;
        if self.air != "m7" && self.air != "m8" {
            return Err(WorkshopError::new(format!(
                "air must be m7 or m8, not `{}`",
                self.air
            )));
        }
        Ok(())
    }

    /// Changes one field; the profile is unchanged on error.
    ///
    /// # Errors
    ///
    /// An unknown key or an invalid value.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), WorkshopError> {
        let mut next = self.clone();
        match key {
            "width" => next.width = number(key, value)?,
            "height" => next.height = number(key, value)?,
            "origin" => {
                next.origin = value
                    .parse()
                    .map_err(|e: lasercraft_project::ProjectError| {
                        WorkshopError::new(e.to_string())
                    })?;
            }
            "port" => value.trim().clone_into(&mut next.port),
            "baud" => {
                next.baud = value.trim().parse().map_err(|_| {
                    WorkshopError::new(format!("baud must be a whole number, not `{value}`"))
                })?;
            }
            "max_speed" => next.max_speed = number(key, value)?,
            "acceleration" => next.acceleration = number(key, value)?,
            "max_power" => next.max_power = number(key, value)?,
            "s_max" => next.s_max = number(key, value)?,
            "air" => next.air = value.trim().to_ascii_lowercase(),
            other => {
                return Err(WorkshopError::new(format!(
                    "unknown device setting `{other}`; one of {}",
                    KEYS.join(", ")
                )));
            }
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// The value of a field as `set` takes it.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        Some(match key {
            "width" => self.width.to_string(),
            "height" => self.height.to_string(),
            "origin" => self.origin.as_str().to_owned(),
            "port" => self.port.clone(),
            "baud" => self.baud.to_string(),
            "max_speed" => self.max_speed.to_string(),
            "acceleration" => self.acceleration.to_string(),
            "max_power" => self.max_power.to_string(),
            "s_max" => self.s_max.to_string(),
            "air" => self.air.clone(),
            _ => return None,
        })
    }

    /// One summary line: size, origin and port.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut line = format!(
            "{} × {} mm, origin {}",
            self.width,
            self.height,
            self.origin.as_str().replace('-', " ")
        );
        if !self.port.is_empty() {
            let _ = write!(line, ", {} at {} baud", self.port, self.baud);
        }
        line
    }
}

/// The exchange file of one profile (`.lcdevice`).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProfileFile {
    pub format: String,
    pub version: u32,
    pub profile: DeviceProfile,
}

/// The profiles file in the configuration folder (`devices.json`).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProfilesFile {
    pub format: String,
    pub version: u32,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub profiles: Vec<DeviceProfile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_validates_and_leaves_the_profile_alone_on_error() {
        let mut p = DeviceProfile::new("Diode").unwrap();
        p.set("width", "300").unwrap();
        p.set("origin", "rear-left").unwrap();
        p.set("air", "M7").unwrap();
        assert_eq!(
            (p.width, p.origin, p.air.as_str()),
            (300.0, Origin::RearLeft, "m7")
        );
        for (key, value) in [
            ("width", "0"),
            ("max_power", "120"),
            ("air", "m9"),
            ("baud", "fast"),
            ("color", "red"),
        ] {
            let before = p.clone();
            assert!(p.set(key, value).is_err(), "{key}={value}");
            assert_eq!(p, before);
        }
        for key in KEYS {
            let v = p.get(key).unwrap();
            p.set(key, &v).unwrap();
        }
        assert!(DeviceProfile::new("  ").is_err());
    }
}
