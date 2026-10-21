//! Layers: one per palette color, with cut settings (`docs/spec/layers.md`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ProjectError;

/// Number of output colors (and layers).
pub const LAYER_COUNT: usize = 30;

/// Longest layer name, in characters.
pub const MAX_LAYER_NAME_CHARS: usize = 60;

/// Highest allowed speed, mm/s.
pub const MAX_SPEED: f64 = 10_000.0;

/// Most passes per layer.
pub const MAX_PASSES: u32 = 1000;

/// Allowed line interval range, mm.
pub const INTERVAL_RANGE: (f64, f64) = (0.001, 10.0);

/// Largest kerf offset either way, mm.
pub const MAX_KERF: f64 = 5.0;

/// Most tabs per shape.
pub const MAX_TABS: u32 = 100;

/// Allowed tab width range, mm.
pub const TAB_WIDTH_RANGE: (f64, f64) = (0.01, 50.0);

/// Default tab width, mm.
pub const DEFAULT_TAB_WIDTH: f64 = 1.0;

/// A palette color index, 0–29.
pub type Color = u8;

/// The palette as sRGB triples: our own colors. Index 0 is black; the rest
/// walk the hue wheel, alternating brightness so neighbors differ.
#[must_use]
pub fn palette() -> [[u8; 3]; LAYER_COUNT] {
    let mut colors = [[0, 0, 0]; LAYER_COUNT];
    for (i, c) in colors.iter_mut().enumerate().skip(1) {
        let hue = (i - 1) as f64 * 360.0 / (LAYER_COUNT - 1) as f64;
        let value = if i % 2 == 0 { 0.65 } else { 0.95 };
        *c = hsv(hue, 0.85, value);
    }
    colors
}

/// The palette color closest to `rgb` (smallest squared RGB distance; ties
/// go to the lower number), used to put imported art on layers
/// (`docs/spec/import.md`).
#[must_use]
pub fn nearest_color(rgb: [u8; 3]) -> Color {
    let distance = |c: &[u8; 3]| -> u32 {
        c.iter()
            .zip(rgb)
            .map(|(a, b)| u32::from(a.abs_diff(b)).pow(2))
            .sum()
    };
    let mut best = (0, u32::MAX);
    for (i, c) in palette().iter().enumerate() {
        let d = distance(c);
        if d < best.1 {
            // At most 30 colors, so the index fits.
            best = (i as Color, d);
        }
    }
    best.0
}

fn hsv(hue: f64, saturation: f64, value: f64) -> [u8; 3] {
    let c = value * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = value - c;
    // Each channel is in 0..=1, so the cast cannot overflow.
    let byte = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    [byte(r), byte(g), byte(b)]
}

/// Parses a color index argument.
///
/// # Errors
///
/// Returns [`ProjectError::InvalidValue`] unless it is 0–29.
pub fn parse_color(text: &str) -> Result<Color, ProjectError> {
    text.parse::<u8>()
        .ok()
        .filter(|c| usize::from(*c) < LAYER_COUNT)
        .ok_or_else(|| {
            ProjectError::invalid(
                "color",
                format!("`{text}` is not a color number 0–{}", LAYER_COUNT - 1),
            )
        })
}

/// What a layer does with vector shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Trace outlines.
    #[default]
    Line,
    /// Scan lines inside closed shapes.
    Fill,
}

/// How images on a layer are turned into dots (M7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dither {
    /// Power follows the gray level.
    Grayscale,
    /// On above a fixed level.
    Threshold,
    /// Floyd–Steinberg error diffusion.
    #[default]
    FloydSteinberg,
    /// Jarvis–Judice–Ninke error diffusion.
    Jarvis,
    /// Stucki error diffusion.
    Stucki,
    /// Ordered (Bayer) dithering.
    Ordered,
    /// Halftone dots.
    Halftone,
}

macro_rules! names {
    ($ty:ty { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl $ty {
            /// The argument spelling.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $name),+ }
            }
        }
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
        impl FromStr for $ty {
            type Err = ProjectError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($name => Ok(Self::$variant),)+
                    other => Err(ProjectError::invalid(
                        stringify!($ty),
                        format!("`{other}` is not one of {}", [$($name),+].join(", ")),
                    )),
                }
            }
        }
    };
}

names!(Mode { Line => "line", Fill => "fill" });
names!(Dither {
    Grayscale => "grayscale",
    Threshold => "threshold",
    FloydSteinberg => "floyd_steinberg",
    Jarvis => "jarvis",
    Stucki => "stucki",
    Ordered => "ordered",
    Halftone => "halftone",
});

/// Settings for images on a layer (used from M7).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSettings {
    /// How gray levels become dots.
    pub dither: Dither,
    /// Line spacing, mm.
    pub interval: f64,
    /// Invert the image.
    pub negative: bool,
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            dither: Dither::default(),
            interval: 0.1,
            negative: false,
        }
    }
}

/// One layer's settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    /// Palette color, 0–29.
    pub color: Color,
    /// Label in the list.
    pub name: String,
    /// Line or fill.
    pub mode: Mode,
    /// Top speed, mm/s.
    pub speed: f64,
    /// Peak power, %.
    pub max_power: f64,
    /// Low-speed / lightest-shade power, %.
    pub min_power: f64,
    /// Times the layer is run.
    pub passes: u32,
    /// Fill line spacing, mm.
    pub interval: f64,
    /// Fill scan angle, degrees.
    pub angle: f64,
    /// Fill burns in both directions.
    pub bidirectional: bool,
    /// Fill runs on past line ends with the laser off.
    pub overscan: bool,
    /// Fill adds a second pass at 90°.
    pub crosshatch: bool,
    /// Included in preview, export and jobs.
    pub output: bool,
    /// Drawn on the canvas.
    pub show: bool,
    /// Air assist.
    pub air: bool,
    /// Image settings.
    pub image: ImageSettings,
    /// Line mode: kerf offset for closed shapes, mm; positive moves the cut
    /// outward from the part (M4).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub kerf: f64,
    /// Line mode: tabs left uncut per closed outer shape; 0 is off (M4).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub tabs: u32,
    /// Line mode: width of each tab, mm (M4).
    #[serde(
        default = "default_tab_width",
        skip_serializing_if = "is_default_tab_width"
    )]
    pub tab_width: f64,
    /// G-code: constant power (`M3`) instead of the default dynamic power
    /// (`M4`), whose laser turns off whenever the head stops (M5). Pause is
    /// then motion only (`docs/spec/safety.md`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub constant_power: bool,
}

// Planning settings (M4) are left out of files while at their defaults, so
// older files and goldens stay as they were.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(v: &bool) -> bool {
    !*v
}

fn default_tab_width() -> f64 {
    DEFAULT_TAB_WIDTH
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_tab_width(v: &f64) -> bool {
    *v == DEFAULT_TAB_WIDTH
}

impl Layer {
    /// Default settings for a color.
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            name: format!("Layer {color:02}"),
            mode: Mode::Line,
            speed: 10.0,
            max_power: 20.0,
            min_power: 0.0,
            passes: 1,
            interval: 0.1,
            angle: 0.0,
            bidirectional: true,
            overscan: false,
            crosshatch: false,
            output: true,
            show: true,
            air: false,
            image: ImageSettings::default(),
            kerf: 0.0,
            tabs: 0,
            tab_width: DEFAULT_TAB_WIDTH,
            constant_power: false,
        }
    }

    /// Checks every setting.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for the first bad setting.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if usize::from(self.color) >= LAYER_COUNT {
            return Err(ProjectError::invalid("color", "must be 0–29"));
        }
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > MAX_LAYER_NAME_CHARS {
            return Err(ProjectError::invalid(
                "name",
                format!("must be 1–{MAX_LAYER_NAME_CHARS} characters"),
            ));
        }
        in_range("speed", self.speed, f64::MIN_POSITIVE, MAX_SPEED)?;
        in_range("max_power", self.max_power, 0.0, 100.0)?;
        in_range("min_power", self.min_power, 0.0, 100.0)?;
        if self.min_power > self.max_power {
            return Err(ProjectError::invalid(
                "min_power",
                format!(
                    "{} must not be above max_power {}",
                    self.min_power, self.max_power
                ),
            ));
        }
        if !(1..=MAX_PASSES).contains(&self.passes) {
            return Err(ProjectError::invalid(
                "passes",
                format!("must be 1–{MAX_PASSES}"),
            ));
        }
        in_range(
            "interval",
            self.interval,
            INTERVAL_RANGE.0,
            INTERVAL_RANGE.1,
        )?;
        in_range("angle", self.angle, -360.0, 360.0)?;
        in_range(
            "image.interval",
            self.image.interval,
            INTERVAL_RANGE.0,
            INTERVAL_RANGE.1,
        )?;
        in_range("kerf", self.kerf, -MAX_KERF, MAX_KERF)?;
        if self.tabs > MAX_TABS {
            return Err(ProjectError::invalid(
                "tabs",
                format!("must be 0–{MAX_TABS}"),
            ));
        }
        in_range(
            "tab_width",
            self.tab_width,
            TAB_WIDTH_RANGE.0,
            TAB_WIDTH_RANGE.1,
        )
    }

    /// The value of a setting, formatted as `layer.set` takes it.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an unknown setting.
    pub fn get(&self, key: &str) -> Result<String, ProjectError> {
        let on = |b: bool| if b { "on" } else { "off" }.to_owned();
        Ok(match key {
            "name" => self.name.clone(),
            "mode" => self.mode.to_string(),
            "speed" => self.speed.to_string(),
            "max_power" => self.max_power.to_string(),
            "min_power" => self.min_power.to_string(),
            "passes" => self.passes.to_string(),
            "interval" => self.interval.to_string(),
            "angle" => self.angle.to_string(),
            "bidirectional" => on(self.bidirectional),
            "overscan" => on(self.overscan),
            "crosshatch" => on(self.crosshatch),
            "output" => on(self.output),
            "show" => on(self.show),
            "air" => on(self.air),
            "image.dither" => self.image.dither.to_string(),
            "image.interval" => self.image.interval.to_string(),
            "image.negative" => on(self.image.negative),
            "kerf" => self.kerf.to_string(),
            "tabs" => self.tabs.to_string(),
            "tab_width" => self.tab_width.to_string(),
            "constant_power" => on(self.constant_power),
            other => return Err(unknown_setting(other)),
        })
    }

    /// Sets one setting from text, validating the whole layer. On error the
    /// layer is unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for an unknown setting or a bad
    /// value.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), ProjectError> {
        let mut next = self.clone();
        match key {
            "name" => value.trim().clone_into(&mut next.name),
            "mode" => next.mode = value.parse()?,
            "speed" => next.speed = number(key, value)?,
            "max_power" => next.max_power = number(key, value)?,
            "min_power" => next.min_power = number(key, value)?,
            "passes" => next.passes = whole(key, value)?,
            "interval" => next.interval = interval(key, value)?,
            "angle" => next.angle = number(key, value)?,
            "bidirectional" => next.bidirectional = switch(key, value)?,
            "overscan" => next.overscan = switch(key, value)?,
            "crosshatch" => next.crosshatch = switch(key, value)?,
            "output" => next.output = switch(key, value)?,
            "show" => next.show = switch(key, value)?,
            "air" => next.air = switch(key, value)?,
            "image.dither" => next.image.dither = value.parse()?,
            "image.interval" => next.image.interval = interval(key, value)?,
            "image.negative" => next.image.negative = switch(key, value)?,
            "kerf" => next.kerf = number(key, value)?,
            "tabs" => next.tabs = whole(key, value)?,
            "tab_width" => next.tab_width = number(key, value)?,
            "constant_power" => next.constant_power = switch(key, value)?,
            other => return Err(unknown_setting(other)),
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
}

/// Every setting name `layer.set` accepts.
pub const SETTINGS: [&str; 21] = [
    "name",
    "mode",
    "speed",
    "max_power",
    "min_power",
    "passes",
    "interval",
    "angle",
    "bidirectional",
    "overscan",
    "crosshatch",
    "output",
    "show",
    "air",
    "image.dither",
    "image.interval",
    "image.negative",
    "kerf",
    "tabs",
    "tab_width",
    "constant_power",
];

fn unknown_setting(key: &str) -> ProjectError {
    ProjectError::invalid(
        "setting",
        format!("`{key}` is not one of {}", SETTINGS.join(", ")),
    )
}

fn in_range(field: &str, value: f64, min: f64, max: f64) -> Result<(), ProjectError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ProjectError::invalid(
            field,
            format!("must be between {min} and {max}, got {value}"),
        ))
    }
}

fn whole(field: &str, value: &str) -> Result<u32, ProjectError> {
    value
        .parse()
        .map_err(|_| ProjectError::invalid(field, format!("`{value}` is not a whole number")))
}

fn number(field: &str, value: &str) -> Result<f64, ProjectError> {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| ProjectError::invalid(field, format!("`{value}` is not a number")))
}

/// Millimetres, or `<n>lpi` / `<n>dpi` meaning 25.4 / n mm (to the nearest
/// nanometre).
fn interval(field: &str, value: &str) -> Result<f64, ProjectError> {
    let per_inch = value
        .strip_suffix("lpi")
        .or_else(|| value.strip_suffix("dpi"));
    match per_inch {
        Some(n) => {
            let n = number(field, n)?;
            if n <= 0.0 {
                return Err(ProjectError::invalid(
                    field,
                    "lines per inch must be positive",
                ));
            }
            // Rounded to the nanometre so 254lpi reads back as 0.1 mm.
            Ok((25.4 / n * 1e9).round() / 1e9)
        }
        None => number(field, value),
    }
}

fn switch(field: &str, value: &str) -> Result<bool, ProjectError> {
    match value {
        "on" | "true" | "yes" => Ok(true),
        "off" | "false" | "no" => Ok(false),
        other => Err(ProjectError::invalid(
            field,
            format!("`{other}` is not on or off"),
        )),
    }
}

/// Default layers for a new project, in color order.
#[must_use]
pub fn default_layers() -> Vec<Layer> {
    (0..LAYER_COUNT)
        .map(|c| Layer::new(u8::try_from(c).unwrap_or(0)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_color_matches_exactly_or_closest() {
        for (i, c) in palette().iter().enumerate() {
            assert_eq!(usize::from(nearest_color(*c)), i);
        }
        assert_eq!(nearest_color([10, 10, 10]), 0);
        let near_one = palette()[1].map(|v| v.saturating_sub(3));
        assert_eq!(nearest_color(near_one), 1);
    }

    #[test]
    fn palette_has_distinct_colors() {
        let p = palette();
        assert_eq!(p[0], [0, 0, 0]);
        for i in 0..LAYER_COUNT {
            for j in i + 1..LAYER_COUNT {
                assert_ne!(p[i], p[j], "{i} and {j}");
            }
        }
    }

    #[test]
    fn defaults_are_valid_and_named() {
        for layer in default_layers() {
            layer.validate().unwrap();
        }
        assert_eq!(Layer::new(7).name, "Layer 07");
    }

    #[test]
    fn set_and_get_every_setting() {
        let mut l = Layer::new(1);
        for (key, value) in [
            ("name", "Cut"),
            ("mode", "fill"),
            ("speed", "120.5"),
            ("max_power", "80"),
            ("min_power", "10"),
            ("passes", "3"),
            ("interval", "0.08"),
            ("angle", "45"),
            ("bidirectional", "off"),
            ("overscan", "on"),
            ("crosshatch", "on"),
            ("output", "off"),
            ("show", "off"),
            ("air", "on"),
            ("image.dither", "stucki"),
            ("image.interval", "0.05"),
            ("image.negative", "on"),
            ("kerf", "-0.08"),
            ("tabs", "4"),
            ("tab_width", "0.5"),
            ("constant_power", "on"),
        ] {
            l.set(key, value).unwrap();
            assert_eq!(l.get(key).unwrap(), value, "{key}");
        }
        assert_eq!(SETTINGS.len(), 21);
        for key in SETTINGS {
            assert!(l.get(key).is_ok(), "{key}");
        }
    }

    #[test]
    fn interval_accepts_lines_per_inch() {
        let mut l = Layer::new(0);
        l.set("interval", "254lpi").unwrap();
        assert_eq!(l.interval, 0.1);
        l.set("image.interval", "508dpi").unwrap();
        assert_eq!(l.image.interval, 0.05);
        l.set("interval", "300lpi").unwrap();
        assert_eq!(l.interval, 0.084_666_667);
        assert!(l.set("interval", "0lpi").is_err());
    }

    #[test]
    fn bad_values_leave_the_layer_alone() {
        let mut l = Layer::new(0);
        let before = l.clone();
        for (key, value) in [
            ("speed", "0"),
            ("speed", "nan"),
            ("max_power", "101"),
            ("min_power", "-1"),
            ("min_power", "50"),
            ("passes", "0"),
            ("passes", "1.5"),
            ("interval", "20"),
            ("mode", "engrave"),
            ("output", "maybe"),
            ("name", "  "),
            ("bogus", "1"),
            ("kerf", "5.5"),
            ("kerf", "-6"),
            ("tabs", "101"),
            ("tabs", "-1"),
            ("tab_width", "0"),
            ("tab_width", "51"),
        ] {
            assert!(l.set(key, value).is_err(), "{key} {value}");
        }
        assert_eq!(l, before);
        // Lowering max below min fails; lower min first.
        l.set("min_power", "15").unwrap();
        assert!(l.set("max_power", "10").is_err());
    }

    #[test]
    fn planning_settings_default_and_stay_out_of_files() {
        let l = Layer::new(4);
        assert_eq!((l.kerf, l.tabs, l.tab_width), (0.0, 0, DEFAULT_TAB_WIDTH));
        let json = serde_json::to_string(&l).unwrap();
        assert!(!json.contains("kerf") && !json.contains("tab"), "{json}");
        // Older layer JSON without the fields still reads.
        assert_eq!(serde_json::from_str::<Layer>(&json).unwrap(), l);
        let mut changed = l.clone();
        changed.set("kerf", "0.1").unwrap();
        changed.set("tabs", "2").unwrap();
        changed.set("tab_width", "0.4").unwrap();
        let json = serde_json::to_string(&changed).unwrap();
        assert!(json.contains("\"kerf\":0.1") && json.contains("\"tab_width\":0.4"));
        assert_eq!(serde_json::from_str::<Layer>(&json).unwrap(), changed);
    }

    #[test]
    fn colors_parse_in_range() {
        assert_eq!(parse_color("0").unwrap(), 0);
        assert_eq!(parse_color("29").unwrap(), 29);
        assert!(parse_color("30").is_err());
        assert!(parse_color("-1").is_err());
    }
}

/// All 30 layers in output order. In files it is written as the order plus
/// only the layers whose settings differ from the defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "LayerFile", into = "LayerFile")]
pub(crate) struct LayerTable(pub(crate) Vec<Layer>);

impl Default for LayerTable {
    fn default() -> Self {
        Self(default_layers())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerFile {
    order: Vec<Color>,
    #[serde(default)]
    changed: Vec<Layer>,
}

impl From<LayerTable> for LayerFile {
    fn from(table: LayerTable) -> Self {
        let order = table.0.iter().map(|l| l.color).collect();
        let mut changed: Vec<Layer> = table
            .0
            .into_iter()
            .filter(|l| *l != Layer::new(l.color))
            .collect();
        changed.sort_by_key(|l| l.color);
        Self { order, changed }
    }
}

impl TryFrom<LayerFile> for LayerTable {
    type Error = String;

    fn try_from(file: LayerFile) -> Result<Self, String> {
        let mut by_color: Vec<Option<Layer>> = (0..LAYER_COUNT).map(|_| None).collect();
        for layer in file.changed {
            layer.validate().map_err(|e| e.to_string())?;
            let slot = by_color
                .get_mut(usize::from(layer.color))
                .ok_or("layer color out of range")?;
            if slot.replace(layer).is_some() {
                return Err("a layer is listed twice".to_owned());
            }
        }
        let mut layers = Vec::with_capacity(LAYER_COUNT);
        for color in &file.order {
            let slot = by_color
                .get_mut(usize::from(*color))
                .ok_or("layer order holds a color out of range")?;
            let layer = slot.take().unwrap_or_else(|| Layer::new(*color));
            if layers.iter().any(|l: &Layer| l.color == *color) {
                return Err("layer order lists a color twice".to_owned());
            }
            layers.push(layer);
        }
        if layers.len() != LAYER_COUNT {
            return Err("layer order must list all 30 colors".to_owned());
        }
        Ok(Self(layers))
    }
}
