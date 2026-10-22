//! Brightness, contrast and gamma (`docs/spec/images.md`, "Adjustments").

/// Per-image adjustments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Adjust {
    /// −100–100, % of full scale added.
    pub brightness: f64,
    /// −99–99; levels stretch around mid-gray by (100 + c) / (100 − c).
    pub contrast: f64,
    /// 0.1–10; level ↦ level^(1/gamma).
    pub gamma: f64,
}

impl Default for Adjust {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 0.0,
            gamma: 1.0,
        }
    }
}

impl Adjust {
    /// Valid ranges: brightness, contrast, gamma.
    pub const RANGES: [(&'static str, f64, f64); 3] = [
        ("brightness", -100.0, 100.0),
        ("contrast", -99.0, 99.0),
        ("gamma", 0.1, 10.0),
    ];

    /// Whether every value is in range.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        [self.brightness, self.contrast, self.gamma]
            .iter()
            .zip(Self::RANGES)
            .all(|(v, (_, lo, hi))| v.is_finite() && (lo..=hi).contains(v))
    }

    /// Whether this changes nothing.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// Adjusts a level in 0–1.
    #[must_use]
    pub fn apply(&self, level: f64) -> f64 {
        let mut v = (level + self.brightness / 100.0).clamp(0.0, 1.0);
        let factor = (100.0 + self.contrast) / (100.0 - self.contrast);
        v = ((v - 0.5) * factor + 0.5).clamp(0.0, 1.0);
        v.powf(1.0 / self.gamma).clamp(0.0, 1.0)
    }

    /// The adjusted byte for every byte, for drawing.
    #[must_use]
    pub fn table(&self) -> [u8; 256] {
        let mut out = [0u8; 256];
        for (i, slot) in out.iter_mut().enumerate() {
            // In 0..=255 after clamping.
            *slot = (self.apply(i as f64 / 255.0) * 255.0).round() as u8;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_brightness_contrast_gamma() {
        let id = Adjust::default();
        assert!(id.is_identity() && id.is_valid());
        assert_eq!(id.table()[37], 37);
        let bright = Adjust {
            brightness: 20.0,
            ..id
        };
        assert!((bright.apply(0.5) - 0.7).abs() < 1e-12);
        assert_eq!(bright.apply(0.9), 1.0);
        let contrast = Adjust {
            contrast: 50.0,
            ..id
        };
        assert!((contrast.apply(0.6) - 0.8).abs() < 1e-12, "factor 3");
        assert_eq!(contrast.apply(0.5), 0.5);
        let flat = Adjust {
            contrast: -99.0,
            ..id
        };
        assert!((flat.apply(1.0) - 0.5).abs() < 0.01);
        let gamma = Adjust { gamma: 2.0, ..id };
        assert!((gamma.apply(0.25) - 0.5).abs() < 1e-12);
        assert!(!Adjust { gamma: 0.0, ..id }.is_valid());
        assert!(
            !Adjust {
                contrast: 100.0,
                ..id
            }
            .is_valid()
        );
    }
}
