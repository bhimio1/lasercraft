//! Test patterns (`docs/spec/test-patterns.md`): a speed and power grid and
//! a line interval row, built from settings and planned like a job. A
//! pattern never touches the user's project: it comes with its own scratch
//! project, which holds its layers and shapes for G-code and previews.

use lasercraft_geom::{Affine, Point, Rect};
use lasercraft_project::raster::Gray;
use lasercraft_project::{Color, Dither, Geometry, Image, Layer, Mode, Project, WorkArea};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::emit::Emitter;
use crate::order::Outline;
use crate::{
    Machine, Plan, PlanError, Stats, Warning, font, num, origin_point, plan_layer, timing,
};

/// Layer of the boxes or patches.
const BOX: Color = 0;
/// Layer of the labels.
const LABELS: Color = 1;
/// Layer of the border.
const BORDER: Color = 2;
/// Label text height, mm, before shrinking to fit.
const TEXT: f64 = 2.5;
/// Smallest label text height, mm.
const MIN_TEXT: f64 = 1.0;
/// Gap between labels and what they label, mm.
const GAP: f64 = 1.0;
/// Border margin, mm.
const MARGIN: f64 = 2.0;

/// What a grid axis varies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Setting {
    /// Speed, mm/s.
    Speed,
    /// Max power, %.
    Power,
    /// Fill line interval, mm.
    Interval,
    /// Passes.
    Passes,
}

impl Setting {
    /// The axis title burned beside the grid.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Speed => "SPEED MM/S",
            Self::Power => "POWER %",
            Self::Interval => "INTERVAL MM",
            Self::Passes => "PASSES",
        }
    }

    /// The setting's unit in a label, for the test windows.
    #[must_use]
    pub fn unit(self) -> &'static str {
        match self {
            Self::Speed => " (mm/s)",
            Self::Power => " (%)",
            Self::Interval => " (mm)",
            Self::Passes => "",
        }
    }

    fn range(self) -> (f64, f64) {
        match self {
            Self::Speed => (0.1, lasercraft_project::layer::MAX_SPEED),
            Self::Power => (0.0, 100.0),
            Self::Interval => lasercraft_project::layer::INTERVAL_RANGE,
            Self::Passes => (1.0, f64::from(lasercraft_project::layer::MAX_PASSES)),
        }
    }

    fn apply(self, layer: &mut Layer, value: f64) {
        match self {
            Self::Speed => layer.speed = value,
            Self::Power => {
                layer.max_power = value;
                layer.min_power = layer.min_power.min(value);
            }
            Self::Interval => layer.interval = value,
            Self::Passes => layer.passes = value.round() as u32,
        }
    }

    fn label(self, value: f64) -> String {
        match self {
            Self::Passes => format!("{}", value.round()),
            Self::Interval => num(value),
            _ => num((value * 10.0).round() / 10.0),
        }
    }
}

/// One axis of the speed and power grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Axis {
    /// Rows or columns.
    pub count: u32,
    /// What varies along the axis.
    pub setting: Setting,
    /// First value.
    pub from: f64,
    /// Last value.
    pub to: f64,
    /// Box height (rows) or width (columns), mm.
    pub size: f64,
}

impl Axis {
    /// The axis values, evenly spaced from `from` to `to`.
    #[must_use]
    pub fn values(&self) -> Vec<f64> {
        let n = self.count.max(2);
        (0..n)
            .map(|i| self.from + (self.to - self.from) * f64::from(i) / f64::from(n - 1))
            .map(|v| {
                if self.setting == Setting::Passes {
                    v.round()
                } else {
                    v
                }
            })
            .collect()
    }

    fn validate(&self, which: &str) -> Result<(), PlanError> {
        if !(2..=20).contains(&self.count) {
            return Err(PlanError(format!("{which} count must be 2–20")));
        }
        let (min, max) = self.setting.range();
        for (name, v) in [("from", self.from), ("to", self.to)] {
            if !(v.is_finite() && (min..=max).contains(&v)) {
                return Err(PlanError(format!(
                    "{which} {name} must be between {} and {} for {}",
                    num(min),
                    num(max),
                    self.setting.title().to_lowercase()
                )));
            }
        }
        range(&format!("{which} size"), self.size, 1.0, 100.0)
    }
}

fn range(what: &str, v: f64, min: f64, max: f64) -> Result<(), PlanError> {
    if v.is_finite() && (min..=max).contains(&v) {
        Ok(())
    } else {
        Err(PlanError(format!(
            "{what} must be between {} and {}",
            num(min),
            num(max)
        )))
    }
}

fn layer(mode: Mode, speed: f64, power: f64, name: &str, color: Color) -> Layer {
    let mut l = Layer::new(color);
    l.mode = mode;
    l.speed = speed;
    l.max_power = power;
    name.clone_into(&mut l.name);
    l
}

/// Speed and power test settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeedPower {
    /// Down the rows.
    pub rows: Axis,
    /// Across the columns.
    pub columns: Axis,
    /// Gap between boxes, mm.
    pub spacing: f64,
    /// Pattern center in document mm; `None` is the bed center.
    pub center: Option<[f64; 2]>,
    /// The boxes' settings.
    #[serde(rename = "box")]
    pub boxes: Layer,
    /// The labels' settings.
    pub labels: Layer,
    /// The border's settings.
    pub border: Layer,
    /// Burn the labels.
    pub draw_labels: bool,
    /// Burn the border.
    pub draw_border: bool,
}

impl Default for SpeedPower {
    fn default() -> Self {
        let mut boxes = layer(Mode::Fill, 100.0, 30.0, "Boxes", BOX);
        boxes.interval = 0.1;
        Self {
            rows: Axis {
                count: 10,
                setting: Setting::Speed,
                from: 10.0,
                to: 100.0,
                size: 5.0,
            },
            columns: Axis {
                count: 10,
                setting: Setting::Power,
                from: 10.0,
                to: 100.0,
                size: 5.0,
            },
            spacing: 1.5,
            center: None,
            boxes,
            labels: layer(Mode::Line, 50.0, 20.0, "Labels", LABELS),
            border: layer(Mode::Line, 50.0, 20.0, "Border", BORDER),
            draw_labels: true,
            draw_border: true,
        }
    }
}

/// How interval test patches are filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchFill {
    /// A fill scan.
    Solid,
    /// A white-to-black ramp engraved as an image.
    Gradient,
}

/// Line interval test settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalTest {
    /// mm/s.
    pub speed: f64,
    /// Max power, %.
    pub power: f64,
    /// Number of patches.
    pub patches: u32,
    /// Smallest interval, mm.
    pub smallest: f64,
    /// Largest interval, mm.
    pub largest: f64,
    /// Patch side, mm.
    pub size: f64,
    /// Gap between patches, mm.
    pub spacing: f64,
    /// Solid or gradient.
    pub fill: PatchFill,
    /// Image method for gradients.
    pub dither: Dither,
    /// Pattern center; `None` is the bed center.
    pub center: Option<[f64; 2]>,
    /// Burn the labels.
    pub draw_labels: bool,
}

impl Default for IntervalTest {
    fn default() -> Self {
        Self {
            speed: 100.0,
            power: 30.0,
            patches: 8,
            smallest: 0.05,
            largest: 0.3,
            size: 10.0,
            spacing: 3.0,
            fill: PatchFill::Solid,
            dither: Dither::FloydSteinberg,
            center: None,
            draw_labels: true,
        }
    }
}

impl IntervalTest {
    /// The patch intervals, smallest first.
    #[must_use]
    pub fn intervals(&self) -> Vec<f64> {
        let n = self.patches.max(2);
        (0..n)
            .map(|i| {
                self.smallest + (self.largest - self.smallest) * f64::from(i) / f64::from(n - 1)
            })
            .collect()
    }
}

/// A pattern's settings.
#[derive(Debug, Clone, PartialEq)]
pub enum TestSettings {
    /// Speed and power test.
    SpeedPower(Box<SpeedPower>),
    /// Line interval test.
    Interval(IntervalTest),
}

/// Deep-merges `patch` into `base`: objects key by key, anything else
/// replaced.
fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                match b.get_mut(k) {
                    Some(slot) => merge(slot, v),
                    None => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (slot, v) => *slot = v.clone(),
    }
}

fn with_defaults<T: Serialize + DeserializeOwned + Default>(
    patch: Option<&Value>,
) -> Result<T, PlanError> {
    let mut value = serde_json::to_value(T::default()).map_err(|e| PlanError(e.to_string()))?;
    if let Some(p) = patch {
        if !p.is_object() {
            return Err(PlanError("test settings must be a JSON object".to_owned()));
        }
        merge(&mut value, p);
    }
    serde_json::from_value(value).map_err(|e| PlanError(format!("test settings: {e}")))
}

impl TestSettings {
    /// Settings of `kind` (`speed_power` or `interval`) from a partial JSON
    /// object; missing keys take the defaults.
    ///
    /// # Errors
    ///
    /// An unknown kind, unknown keys, wrong types or values out of range.
    pub fn parse(kind: &str, patch: Option<&Value>) -> Result<Self, PlanError> {
        let settings = match kind {
            "speed_power" => Self::SpeedPower(Box::new(with_defaults(patch)?)),
            "interval" => Self::Interval(with_defaults(patch)?),
            other => {
                return Err(PlanError(format!(
                    "unknown test `{other}`; speed_power or interval"
                )));
            }
        };
        settings.validate()?;
        Ok(settings)
    }

    /// The settings as a JSON object (complete).
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::SpeedPower(s) => serde_json::to_value(s),
            Self::Interval(s) => serde_json::to_value(s),
        }
        .unwrap_or(Value::Null)
    }

    /// The kind name.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::SpeedPower(_) => "speed_power",
            Self::Interval(_) => "interval",
        }
    }

    /// Checks every field.
    ///
    /// # Errors
    ///
    /// The first field out of range.
    pub fn validate(&self) -> Result<(), PlanError> {
        let layer_ok = |l: &Layer, what: &str| {
            l.validate()
                .map_err(|e| PlanError(format!("{what} settings: {e}")))
        };
        match self {
            Self::SpeedPower(s) => {
                s.rows.validate("rows")?;
                s.columns.validate("columns")?;
                if s.rows.setting == s.columns.setting {
                    return Err(PlanError(
                        "rows and columns must vary different settings".to_owned(),
                    ));
                }
                range("spacing", s.spacing, 0.0, 20.0)?;
                layer_ok(&s.boxes, "box")?;
                layer_ok(&s.labels, "label")?;
                layer_ok(&s.border, "border")
            }
            Self::Interval(t) => {
                range("speed", t.speed, 0.1, lasercraft_project::layer::MAX_SPEED)?;
                range("power", t.power, 0.0, 100.0)?;
                if !(2..=20).contains(&t.patches) {
                    return Err(PlanError("patches must be 2–20".to_owned()));
                }
                let (min, max) = lasercraft_project::layer::INTERVAL_RANGE;
                range("smallest interval", t.smallest, min, max)?;
                range("largest interval", t.largest, min, max)?;
                if t.smallest >= t.largest {
                    return Err(PlanError(
                        "the smallest interval must be below the largest".to_owned(),
                    ));
                }
                range("patch size", t.size, 2.0, 50.0)?;
                range("spacing", t.spacing, 0.0, 20.0)
            }
        }
    }
}

/// A planned pattern.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// The scratch project: the pattern's layers and shapes, on the target
    /// work area.
    pub project: Project,
    /// The plan.
    pub plan: Plan,
    /// The pattern's size, mm (border included).
    pub size: (f64, f64),
}

/// One thing to burn: its layer settings, outlines (or image shapes in the
/// scratch project).
struct Piece {
    layer: Layer,
    outlines: Vec<Outline>,
    /// Image shapes in the scratch project, with their place in the
    /// pattern's frame.
    images: Vec<(u32, Rect)>,
    energy: f64,
}

fn energy(layer: &Layer, machine: &Machine) -> f64 {
    let power = layer.max_power.min(machine.max_power);
    let interval = if layer.mode == Mode::Fill {
        layer.interval
    } else {
        1.0
    };
    power * f64::from(layer.passes) / (layer.speed.min(machine.max_speed) * interval)
}

fn rect(r: &Rect) -> Geometry {
    Geometry::Rect {
        width: r.width(),
        height: r.height(),
    }
}

fn rect_points(r: &Rect) -> Vec<Point> {
    vec![
        r.min,
        Point::new(r.max.x, r.min.y),
        r.max,
        Point::new(r.min.x, r.max.y),
    ]
}

/// Collects shapes and outlines in the pattern's own frame (top left at the
/// origin), then moves them into place.
struct Drawing {
    project: Project,
    labels: Vec<Vec<Point>>,
    pieces: Vec<Piece>,
}

impl Drawing {
    fn new(area: &WorkArea, name: &str) -> Result<Self, PlanError> {
        let mut project = Project::with_work_area(area.width_mm, area.height_mm)
            .map_err(|e| PlanError(e.to_string()))?;
        project.set_origin(area.origin);
        project
            .set_name(name)
            .map_err(|e| PlanError(e.to_string()))?;
        Ok(Self {
            project,
            labels: Vec::new(),
            pieces: Vec::new(),
        })
    }

    fn text(&mut self, text: &str, height: f64, place: impl Fn(Point) -> Point) {
        self.labels.extend(font::strokes(text, height, place));
    }

    fn bounds(&self) -> Option<Rect> {
        Rect::bounding(
            self.labels
                .iter()
                .flatten()
                .copied()
                .chain(self.pieces.iter().flat_map(|p| {
                    p.outlines
                        .iter()
                        .flat_map(|o| o.points.iter().copied())
                        .chain(p.images.iter().flat_map(|(_, r)| [r.min, r.max]))
                })),
        )
    }

    /// Moves everything by `d`, adds the shapes to the scratch project and
    /// plans it: labels and border first, then the pieces from the least
    /// energy up.
    fn finish(
        mut self,
        layers: [&Layer; 3],
        border: bool,
        center: Option<[f64; 2]>,
        machine: &Machine,
    ) -> Result<Pattern, PlanError> {
        machine.validate()?;
        let mut outer = self
            .bounds()
            .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO));
        if border {
            outer = Rect::from_corners(
                outer.min - Point::new(MARGIN, MARGIN),
                outer.max + Point::new(MARGIN, MARGIN),
            );
        }
        let area = *self.project.work_area();
        let [cx, cy] = center.unwrap_or([area.width_mm / 2.0, area.height_mm / 2.0]);
        let d = Point::new(cx, cy) - outer.center();
        let shift = |p: Point| p + d;
        for (color, l) in [BOX, LABELS, BORDER].into_iter().zip(layers) {
            self.project
                .replace_layer(color, l.clone())
                .map_err(|e| PlanError(e.to_string()))?;
        }
        let mut warnings = Vec::new();
        let mut out = Emitter::new(origin_point(&area), machine.max_speed);
        let mut summaries = Vec::new();
        // Labels and border.
        let mut lines = Vec::new();
        for stroke in std::mem::take(&mut self.labels) {
            let points: Vec<Point> = stroke.iter().copied().map(shift).collect();
            let id = self.add(
                Geometry::Polyline {
                    points: points.clone(),
                    closed: false,
                },
                Point::ZERO,
                LABELS,
            )?;
            lines.push(Outline {
                shape: id,
                points,
                closed: false,
            });
        }
        summaries.extend(plan_layer(
            layers[1],
            lines,
            &[],
            machine,
            &mut out,
            &mut warnings,
        ));
        if border {
            let r = Rect::from_corners(shift(outer.min), shift(outer.max));
            let id = self.add(rect(&r), r.min, BORDER)?;
            let border_line = Outline {
                shape: id,
                points: rect_points(&r),
                closed: true,
            };
            summaries.extend(plan_layer(
                layers[2],
                vec![border_line],
                &[],
                machine,
                &mut out,
                &mut warnings,
            ));
        }
        // Pieces, least energy first; ties keep their order.
        let mut pieces = std::mem::take(&mut self.pieces);
        pieces.sort_by(|a, b| a.energy.total_cmp(&b.energy));
        for piece in pieces {
            let mut outlines = Vec::new();
            for o in piece.outlines {
                let points: Vec<Point> = o.points.into_iter().map(shift).collect();
                let r = Rect::bounding(points.iter().copied())
                    .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO));
                let id = self.add(rect(&r), r.min, BOX)?;
                outlines.push(Outline {
                    shape: id,
                    points,
                    closed: true,
                });
            }
            for (id, r) in &piece.images {
                self.place(*id, shift(r.min))?;
            }
            let shapes: Vec<&lasercraft_project::Shape> = piece
                .images
                .iter()
                .filter_map(|(id, _)| self.project.shape(*id))
                .collect();
            summaries.extend(plan_layer(
                &piece.layer,
                outlines,
                &shapes,
                machine,
                &mut out,
                &mut warnings,
            ));
        }
        // One warning per message: every box repeats its layer's.
        let mut seen = std::collections::HashSet::new();
        warnings.retain(|w| seen.insert(w.message.clone()));
        let mut moves = out.moves;
        timing::estimate(&mut moves, machine);
        let stats = Stats::of(&moves);
        Ok(Pattern {
            size: (outer.width(), outer.height()),
            plan: Plan {
                start: origin_point(&area),
                layers: summaries,
                moves,
                warnings,
                stats,
            },
            project: self.project,
        })
    }

    /// Adds a shape whose geometry starts at the origin, moved to `at`.
    fn add(&mut self, geometry: Geometry, at: Point, color: Color) -> Result<u32, PlanError> {
        self.project
            .add_shape_on(geometry, Affine::translate(at.x, at.y), color)
            .map_err(|e| PlanError(e.to_string()))
    }

    /// Moves shape `id` (geometry at the origin) to `at`.
    fn place(&mut self, id: u32, at: Point) -> Result<(), PlanError> {
        self.project
            .set_shape_transform(id, Affine::translate(at.x, at.y))
            .map_err(|e| PlanError(e.to_string()))
    }
}

/// Warns when a power is above the device's maximum (safety rule 6).
fn clamp_warning(power: f64, machine: &Machine, warnings: &mut Vec<Warning>) {
    if power > machine.max_power {
        let m = num(machine.max_power);
        let message = format!("the device maximum is {m}%. {m}% is used.");
        if !warnings.iter().any(|w| w.message == message) {
            warnings.push(Warning {
                layer: BOX,
                shape: None,
                message,
            });
        }
    }
}

fn speed_power(s: &SpeedPower, area: &WorkArea, machine: &Machine) -> Result<Pattern, PlanError> {
    let mut d = Drawing::new(area, "Speed and power test")?;
    let (rows, cols) = (s.rows.values(), s.columns.values());
    let (rh, cw) = (s.rows.size, s.columns.size);
    // Label sizes: column values must fit their column, row values their row.
    let col_text = cols
        .iter()
        .map(|v| s.columns.setting.label(*v))
        .map(|t| TEXT.min((cw + s.spacing * 0.8) / (font::width(&t, 1.0)).max(1e-9)))
        .fold(TEXT, f64::min)
        .max(MIN_TEXT);
    let row_text = TEXT.min(rh).max(MIN_TEXT);
    let row_labels: Vec<String> = rows.iter().map(|v| s.rows.setting.label(*v)).collect();
    let label_w = row_labels
        .iter()
        .map(|t| font::width(t, row_text))
        .fold(0.0, f64::max);
    let (gx, gy) = if s.draw_labels {
        (TEXT + GAP + label_w + GAP, TEXT + 2.0 * GAP)
    } else {
        (0.0, 0.0)
    };
    let grid_w = cols.len() as f64 * cw + (cols.len() - 1) as f64 * s.spacing;
    let grid_h = rows.len() as f64 * rh + (rows.len() - 1) as f64 * s.spacing;
    let mut warnings = Vec::new();
    for (i, rv) in rows.iter().enumerate() {
        for (j, cv) in cols.iter().enumerate() {
            let mut l = s.boxes.clone();
            l.color = BOX;
            s.rows.setting.apply(&mut l, *rv);
            s.columns.setting.apply(&mut l, *cv);
            clamp_warning(l.max_power, machine, &mut warnings);
            let x = gx + j as f64 * (cw + s.spacing);
            let y = gy + i as f64 * (rh + s.spacing);
            let r = Rect::from_corners(Point::new(x, y), Point::new(x + cw, y + rh));
            d.pieces.push(Piece {
                energy: energy(&l, machine),
                layer: l,
                outlines: vec![Outline {
                    shape: 0,
                    points: rect_points(&r),
                    closed: true,
                }],
                images: Vec::new(),
            });
        }
    }
    if s.draw_labels {
        // Header: the box settings that do not vary.
        let varied = [s.rows.setting, s.columns.setting];
        let b = &s.boxes;
        let mut header = vec![if b.mode == Mode::Fill { "FILL" } else { "LINE" }.to_owned()];
        for (setting, text) in [
            (Setting::Speed, format!("SPEED {} MM/S", num(b.speed))),
            (Setting::Power, format!("POWER {}%", num(b.max_power))),
            (
                Setting::Interval,
                format!("INTERVAL {} MM", num(b.interval)),
            ),
            (Setting::Passes, format!("PASSES {}", b.passes)),
        ] {
            if !varied.contains(&setting) && (setting != Setting::Interval || b.mode == Mode::Fill)
            {
                header.push(text);
            }
        }
        d.text(&header.join("  "), TEXT, |p| Point::new(gx + p.x, p.y));
        for (i, t) in row_labels.iter().enumerate() {
            let y = gy + i as f64 * (rh + s.spacing) + (rh - row_text) / 2.0;
            let x = gx - GAP - font::width(t, row_text);
            d.text(t, row_text, |p| Point::new(x + p.x, y + p.y));
        }
        let vy = gy + grid_h + GAP;
        for (j, v) in cols.iter().enumerate() {
            let t = s.columns.setting.label(*v);
            let x = gx + j as f64 * (cw + s.spacing) + (cw - font::width(&t, col_text)) / 2.0;
            d.text(&t, col_text, |p| Point::new(x + p.x, vy + p.y));
        }
        let title = s.columns.setting.title();
        let tx = gx + (grid_w - font::width(title, TEXT)) / 2.0;
        let ty = vy + col_text + GAP;
        d.text(title, TEXT, |p| Point::new(tx + p.x, ty + p.y));
        // The row title reads upward at the far left.
        let title = s.rows.setting.title();
        let bottom = gy + (grid_h + font::width(title, TEXT)) / 2.0;
        d.text(title, TEXT, |p| Point::new(p.y, bottom - p.x));
    }
    let mut boxes = s.boxes.clone();
    boxes.color = BOX;
    let mut pattern = d.finish(
        [&boxes, &s.labels, &s.border],
        s.draw_border,
        s.center,
        machine,
    )?;
    pattern.plan.warnings.extend(warnings);
    Ok(pattern)
}

/// A ramp from white at the left to black at the right.
fn ramp() -> Result<Gray, PlanError> {
    let (w, h) = (64_u32, 4_u32);
    let pixels = (0..h)
        .flat_map(|_| (0..w).map(move |x| (255 - x * 255 / (w - 1)) as u8))
        .collect();
    Gray::new(w, h, pixels).map_err(|e| PlanError(e.to_string()))
}

fn interval(t: &IntervalTest, area: &WorkArea, machine: &Machine) -> Result<Pattern, PlanError> {
    let mut d = Drawing::new(area, "Line interval test")?;
    let intervals = t.intervals();
    let top = if t.draw_labels { TEXT + 2.0 * GAP } else { 0.0 };
    let mut warnings = Vec::new();
    clamp_warning(t.power, machine, &mut warnings);
    for (k, &i) in intervals.iter().enumerate() {
        let x = k as f64 * (t.size + t.spacing);
        let r = Rect::from_corners(Point::new(x, top), Point::new(x + t.size, top + t.size));
        let mut l = layer(Mode::Fill, t.speed, t.power, "Patches", BOX);
        l.interval = i;
        l.image.interval = i;
        l.image.dither = t.dither;
        let mut piece = Piece {
            energy: -i,
            layer: l,
            outlines: Vec::new(),
            images: Vec::new(),
        };
        match t.fill {
            PatchFill::Solid => piece.outlines.push(Outline {
                shape: 0,
                points: rect_points(&r),
                closed: true,
            }),
            PatchFill::Gradient => {
                let image = Geometry::Image(Image::new(t.size, t.size, ramp()?));
                let id = d.add(image, r.min, BOX)?;
                piece.images.push((id, r));
            }
        }
        d.pieces.push(piece);
        if t.draw_labels {
            let text = num(i);
            let h = TEXT
                .min((t.size + t.spacing * 0.8) / font::width(&text, 1.0))
                .max(MIN_TEXT);
            let lx = x + (t.size - font::width(&text, h)) / 2.0;
            let ly = top + t.size + GAP;
            d.text(&text, h, |p| Point::new(lx + p.x, ly + p.y));
        }
    }
    if t.draw_labels {
        let header = format!("SPEED {} MM/S  POWER {}%", num(t.speed), num(t.power));
        d.text(&header, TEXT, |p| p);
    }
    // Images keep their pattern-frame spot in the bounds.
    let mut base = layer(Mode::Fill, t.speed, t.power, "Patches", BOX);
    base.image.dither = t.dither;
    let labels = layer(
        Mode::Line,
        50.0,
        20.0_f64.min(t.power.max(1.0)),
        "Labels",
        LABELS,
    );
    let border = layer(Mode::Line, 50.0, 20.0, "Border", BORDER);
    let mut pattern = d.finish([&base, &labels, &border], false, t.center, machine)?;
    pattern.plan.warnings.extend(warnings);
    Ok(pattern)
}

/// Builds and plans a pattern on `area` for `machine`.
///
/// # Errors
///
/// Invalid settings or machine parameters.
pub fn build(
    settings: &TestSettings,
    area: &WorkArea,
    machine: &Machine,
) -> Result<Pattern, PlanError> {
    settings.validate()?;
    match settings {
        TestSettings::SpeedPower(s) => speed_power(s, area, machine),
        TestSettings::Interval(t) => interval(t, area, machine),
    }
}

/// Parses settings of `kind` from optional JSON text and builds the pattern.
///
/// # Errors
///
/// Bad JSON, invalid settings or machine parameters.
pub fn from_json(
    kind: &str,
    json: Option<&str>,
    area: &WorkArea,
    machine: &Machine,
) -> Result<Pattern, PlanError> {
    let value: Option<Value> = json
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| PlanError(format!("test settings are not JSON: {e}")))?;
    build(&TestSettings::parse(kind, value.as_ref())?, area, machine)
}

#[cfg(test)]
mod tests;
