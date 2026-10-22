//! Picture settings: the controls a camera offers (`docs/spec/camera.md`,
//! "Picture settings"), by our own keys.

use crate::Picture;

/// Every control key LaserCraft knows, in the order they are shown.
pub const KEYS: [&str; 15] = [
    "brightness",
    "contrast",
    "hue",
    "saturation",
    "sharpness",
    "gamma",
    "white_balance",
    "backlight",
    "gain",
    "pan",
    "tilt",
    "zoom",
    "exposure",
    "iris",
    "focus",
];

/// The words shown for a control key.
#[must_use]
pub fn title(key: &str) -> String {
    match key {
        "white_balance" => "White balance".to_owned(),
        "backlight" => "Backlight compensation".to_owned(),
        _ => {
            let mut c = key.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default()
        }
    }
}

/// What kind of value a control takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// A number in a range.
    Range {
        /// Smallest value.
        min: f64,
        /// Largest value.
        max: f64,
        /// Step between values; 0 when any value goes.
        step: f64,
    },
    /// On (1) or off (0).
    Switch,
}

/// One control as the camera reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    /// One of [`KEYS`].
    pub key: &'static str,
    /// Its kind.
    pub kind: Kind,
    /// The camera's default.
    pub default: f64,
    /// The current value.
    pub value: f64,
}

impl Control {
    /// `v` clamped to the range and rounded to the step (a switch: 0 or 1).
    #[must_use]
    pub fn fit(&self, v: f64) -> f64 {
        match self.kind {
            Kind::Switch => {
                if v >= 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            Kind::Range { min, max, step } => {
                let v = v.clamp(min, max);
                if step > 0.0 {
                    (min + ((v - min) / step).round() * step).clamp(min, max)
                } else {
                    v
                }
            }
        }
    }
}

/// The key LaserCraft uses for a control name, if it knows it.
#[must_use]
pub fn known(key: &str) -> Option<&'static str> {
    let key = key.trim().to_lowercase().replace([' ', '-'], "_");
    KEYS.iter().copied().find(|k| *k == key)
}

/// Sorts controls into the order of [`KEYS`].
pub fn sort(controls: &mut [Control]) {
    controls.sort_by_key(|c| KEYS.iter().position(|k| *k == c.key));
}

/// The simulated camera's controls with the given values.
#[must_use]
pub fn simulated(brightness: f64, contrast: f64) -> Vec<Control> {
    let range = Kind::Range {
        min: -100.0,
        max: 100.0,
        step: 1.0,
    };
    vec![
        Control {
            key: "brightness",
            kind: range,
            default: 0.0,
            value: brightness,
        },
        Control {
            key: "contrast",
            kind: range,
            default: 0.0,
            value: contrast,
        },
    ]
}

/// Brightness and contrast (each −100…100) applied to a picture, as the
/// simulated camera does: contrast scales about mid-gray, then
/// brightness adds up to ±128.
#[must_use]
pub fn adjust(picture: &Picture, brightness: f64, contrast: f64) -> Picture {
    if brightness == 0.0 && contrast == 0.0 {
        return picture.clone();
    }
    let gain = (100.0 + contrast.clamp(-100.0, 100.0)) / 100.0;
    let shift = brightness.clamp(-100.0, 100.0) * 1.28;
    let rgb = picture
        .rgb()
        .iter()
        .map(|&v| {
            ((f64::from(v) - 128.0) * gain + 128.0 + shift)
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect();
    Picture::new(picture.width(), picture.height(), rgb).unwrap_or_else(|_| picture.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_fit_the_range_and_step() {
        let c = Control {
            key: "focus",
            kind: Kind::Range {
                min: 0.0,
                max: 250.0,
                step: 5.0,
            },
            default: 0.0,
            value: 0.0,
        };
        assert_eq!(c.fit(12.0), 10.0);
        assert_eq!(c.fit(13.0), 15.0);
        assert_eq!(c.fit(900.0), 250.0);
        let s = Control {
            kind: Kind::Switch,
            ..c
        };
        assert_eq!(s.fit(0.7), 1.0);
    }

    #[test]
    fn names_titles_and_order() {
        assert_eq!(known("White Balance"), Some("white_balance"));
        assert_eq!(known("aperture"), None);
        assert_eq!(title("brightness"), "Brightness");
        assert_eq!(title("backlight"), "Backlight compensation");
        let mut c = simulated(0.0, 0.0);
        c.reverse();
        sort(&mut c);
        assert_eq!(c[0].key, "brightness");
    }

    #[test]
    fn brightness_and_contrast_change_the_picture() {
        let p = Picture::filled(2, 1, [100, 128, 200]);
        assert_eq!(adjust(&p, 0.0, 0.0), p);
        let brighter = adjust(&p, 50.0, 0.0);
        assert_eq!(brighter.rgb()[0], 164);
        let flat = adjust(&p, 0.0, -100.0);
        assert!(flat.rgb().iter().all(|v| *v == 128));
    }
}
