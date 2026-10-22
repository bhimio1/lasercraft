//! Cameras of a device profile (`docs/spec/camera.md`, "Camera data").

use lasercraft_camera::align::Alignment;
use lasercraft_camera::lens::Lens;
use lasercraft_camera::overlay::Fit;
use serde::{Deserialize, Serialize};

use crate::WorkshopError;
use crate::profile::check_name;

/// Where pictures come from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A camera on a USB cable.
    Usb {
        /// The device's name as the system lists it.
        #[serde(default)]
        device: String,
        /// Its index in the system's list.
        #[serde(default)]
        index: u32,
        /// Picture size, `WxH`; empty for the camera's default.
        #[serde(default)]
        format: String,
    },
    /// A camera that serves pictures over HTTP.
    Network {
        /// Still-picture or motion-JPEG URL.
        url: String,
        /// Pictures per second to ask for.
        #[serde(default = "default_rate")]
        rate: f64,
    },
    /// LaserCraft's simulated camera.
    Simulated,
}

fn default_rate() -> f64 {
    2.0
}

impl Source {
    /// "USB", "Network" or "Simulated".
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Usb { .. } => "USB",
            Self::Network { .. } => "Network",
            Self::Simulated => "Simulated",
        }
    }

    /// One line: kind and device or address.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Usb { device, index, .. } if device.is_empty() => format!("USB · camera {index}"),
            Self::Usb { device, .. } => format!("USB · {device}"),
            Self::Network { url, .. } => format!("Network · {url}"),
            Self::Simulated => "Simulated camera".to_owned(),
        }
    }
}

/// How the camera is mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mount {
    /// Fixed above the bed.
    #[default]
    Bed,
}

/// One camera.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    /// Unique within the profile.
    pub name: String,
    /// Where pictures come from.
    pub source: Source,
    /// Mounting.
    #[serde(default)]
    pub mount: Mount,
    /// The lens correction.
    #[serde(default)]
    pub lens: Option<Lens>,
    /// Bed alignments; the first is in use.
    #[serde(default)]
    pub alignments: Vec<Alignment>,
    /// Included in the overlay.
    #[serde(rename = "use", default = "yes")]
    pub used: bool,
    /// Overlay opacity, %.
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    /// Overlay fine adjustment.
    #[serde(default)]
    pub fit: Fit,
    /// Pictures dropped after the source starts.
    #[serde(default = "default_skip")]
    pub skip: u32,
}

fn yes() -> bool {
    true
}

fn default_opacity() -> f64 {
    70.0
}

fn default_skip() -> u32 {
    3
}

/// The keys `camera.set` accepts.
pub const KEYS: [&str; 12] = [
    "use",
    "opacity",
    "fit_width",
    "fit_height",
    "fit_dx",
    "fit_dy",
    "skip",
    "device",
    "index",
    "format",
    "url",
    "rate",
];

fn number(key: &str, value: &str, (min, max): (f64, f64)) -> Result<f64, WorkshopError> {
    let v: f64 = value
        .trim()
        .parse()
        .map_err(|_| WorkshopError::new(format!("{key} must be a number, not `{value}`")))?;
    if v.is_finite() && (min..=max).contains(&v) {
        Ok(v)
    } else {
        Err(WorkshopError::new(format!(
            "{key} must be between {min} and {max}, got {v}"
        )))
    }
}

/// Checks a picture URL.
///
/// # Errors
///
/// Not an `http://` or `https://` address.
pub fn check_url(url: &str) -> Result<String, WorkshopError> {
    let url = url.trim();
    if (url.starts_with("http://") || url.starts_with("https://")) && url.len() > 8 {
        Ok(url.to_owned())
    } else {
        Err(WorkshopError::new(
            "a network camera's address starts with http:// or https://",
        ))
    }
}

impl Camera {
    /// A camera with the defaults of `docs/spec/camera.md`.
    ///
    /// # Errors
    ///
    /// An invalid name.
    pub fn new(name: &str, source: Source) -> Result<Self, WorkshopError> {
        Ok(Self {
            name: check_name(name)?,
            source,
            mount: Mount::Bed,
            lens: None,
            alignments: Vec::new(),
            used: true,
            opacity: default_opacity(),
            fit: Fit::default(),
            skip: default_skip(),
        })
    }

    /// The alignment in use.
    #[must_use]
    pub fn alignment(&self) -> Option<&Alignment> {
        self.alignments.first()
    }

    /// Whether the alignment in use was made with another lens correction.
    #[must_use]
    pub fn alignment_outdated(&self) -> bool {
        match (&self.lens, self.alignment()) {
            (Some(lens), Some(a)) => a.lens != lens.created,
            _ => false,
        }
    }

    /// Changes one setting.
    ///
    /// # Errors
    ///
    /// An unknown key, a key that does not fit the source, or a bad value.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), WorkshopError> {
        let on = |v: &str| match v {
            "on" | "true" => Ok(true),
            "off" | "false" => Ok(false),
            _ => Err(WorkshopError::new(format!("{key} must be on or off"))),
        };
        match (key, &mut self.source) {
            ("use", _) => self.used = on(value)?,
            ("opacity", _) => self.opacity = number(key, value, (0.0, 100.0))?,
            ("fit_width", _) => self.fit.width = number(key, value, (10.0, 1000.0))?,
            ("fit_height", _) => self.fit.height = number(key, value, (10.0, 1000.0))?,
            ("fit_dx", _) => self.fit.dx = number(key, value, (-1000.0, 1000.0))?,
            ("fit_dy", _) => self.fit.dy = number(key, value, (-1000.0, 1000.0))?,
            ("skip", _) => self.skip = number(key, value, (0.0, 100.0))?.round() as u32,
            ("device", Source::Usb { device, .. }) => value.trim().clone_into(device),
            ("index", Source::Usb { index, .. }) => {
                *index = number(key, value, (0.0, 64.0))?.round() as u32;
            }
            ("format", Source::Usb { format, .. }) => {
                let v = value.trim();
                let ok = v.is_empty()
                    || v.split_once('x').is_some_and(|(w, h)| {
                        w.parse::<u32>().is_ok_and(|w| w > 0)
                            && h.parse::<u32>().is_ok_and(|h| h > 0)
                    });
                if !ok {
                    return Err(WorkshopError::new("format is WxH, for example 1280x720"));
                }
                v.clone_into(format);
            }
            ("url", Source::Network { url, .. }) => *url = check_url(value)?,
            ("rate", Source::Network { rate, .. }) => *rate = number(key, value, (0.1, 30.0))?,
            _ if KEYS.contains(&key) => {
                return Err(WorkshopError::new(format!(
                    "{key} does not apply to a {} camera",
                    self.source.kind().to_lowercase()
                )));
            }
            _ => {
                return Err(WorkshopError::new(format!(
                    "`{key}` is not a camera setting ({})",
                    KEYS.join(", ")
                )));
            }
        }
        Ok(())
    }

    /// Stores an alignment first; one made at the same height is replaced.
    pub fn add_alignment(&mut self, alignment: Alignment) {
        self.alignments
            .retain(|a| (a.height - alignment.height).abs() > 1e-9);
        self.alignments.insert(0, alignment);
    }

    /// Checks names, sources and stored numbers (for files).
    ///
    /// # Errors
    ///
    /// The first problem found.
    pub fn validate(&self) -> Result<(), WorkshopError> {
        check_name(&self.name)?;
        if let Source::Network { url, .. } = &self.source {
            check_url(url)?;
        }
        if let Some(lens) = &self.lens {
            lens.validate()
                .map_err(|e| WorkshopError::new(format!("camera \"{}\": {e}", self.name)))?;
        }
        Ok(())
    }

    /// One line for lists: name, source, correction and alignment state.
    #[must_use]
    pub fn summary(&self) -> String {
        let lens = self.lens.as_ref().map_or_else(
            || "lens not corrected".to_owned(),
            |l| format!("{} lens correction, fit {:.0}%", l.model.title(), l.score()),
        );
        let alignment = match self.alignment() {
            None => "not aligned".to_owned(),
            Some(a) if self.alignment_outdated() => {
                format!("aligned ({}), made with another lens correction", a.name)
            }
            Some(a) => format!("aligned ({})", a.name),
        };
        format!(
            "{}: {}; {lens}; {alignment}{}",
            self.name,
            self.source.summary(),
            if self.used {
                ""
            } else {
                "; not in the overlay"
            }
        )
    }
}

/// The exchange file of one camera (`.lccam`).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CameraFile {
    pub format: String,
    pub version: u32,
    pub camera: Camera,
}

/// The `format` of a camera file.
pub(crate) const FORMAT: &str = "lasercraft-camera";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_follow_the_source_and_alignments_replace_by_height() {
        let mut usb = Camera::new(
            "Top",
            Source::Usb {
                device: String::new(),
                index: 0,
                format: String::new(),
            },
        )
        .unwrap();
        usb.set("format", "1280x720").unwrap();
        assert!(usb.set("format", "big").is_err());
        assert!(usb.set("url", "http://x/y.jpg").is_err(), "not for USB");
        usb.set("opacity", "40").unwrap();
        usb.set("use", "off").unwrap();
        assert!(!usb.used && usb.opacity == 40.0);
        assert!(usb.summary().contains("not in the overlay"));
        let mut net = Camera::new(
            "Lid",
            Source::Network {
                url: "http://cam.local/still.jpg".into(),
                rate: 2.0,
            },
        )
        .unwrap();
        assert!(net.set("url", "ftp://x").is_err());
        net.set("rate", "5").unwrap();
        let align = |height: f64, name: &str| Alignment {
            name: name.into(),
            height,
            matrix: lasercraft_camera::Homography::IDENTITY,
            lens: String::new(),
            width: 640,
            height_px: 480,
            created: String::new(),
        };
        net.add_alignment(align(0.0, "flat"));
        net.add_alignment(align(3.0, "3 mm"));
        net.add_alignment(align(0.0, "flat again"));
        let names: Vec<&str> = net.alignments.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["flat again", "3 mm"]);
        // Round trip through JSON with the `use` spelling.
        let text = serde_json::to_string(&net).unwrap();
        assert!(text.contains("\"use\":true"));
        let back: Camera = serde_json::from_str(&text).unwrap();
        assert_eq!(back, net);
        back.validate().unwrap();
    }
}
