//! The layer list, the layer settings editor and the palette strip
//! (`docs/spec/layers.md`, `docs/spec/style/layers-panel.md`). Every change
//! is emitted as a command; nothing here edits the project.

use egui::{DragValue, Sense, Stroke, StrokeKind, vec2};
use lasercraft_commands::Invocation;
use lasercraft_project::{Color, Dither, Layer, Mode, Project};

use crate::colors::{self, layer_color};
use crate::theme::{self, ROUNDED, Tokens, size, weight};
use crate::widgets::{self, Band};

/// Width of the list's fixed columns: color, mode, speed and power, output,
/// show.
const COLUMNS: [f32; 5] = [30.0, 64.0, 86.0, 46.0, 46.0];

/// List header: caption, tooltip.
const HEADER: [(&str, &str); 6] = [
    ("Color", "Layer color and number, as in the palette"),
    ("Name", "Layer name. Click a row to edit the layer below."),
    (
        "Mode",
        "How the layer runs: Line traces outlines, Fill scans inside closed shapes",
    ),
    ("Speed, power", "Speed (mm/s) and Max power (%)"),
    (
        "Output",
        "Include the layer in the job preview, G-code and jobs",
    ),
    ("Show", "Draw the layer on the canvas"),
];

/// Image methods in menu order, with their readable names.
const METHODS: [(Dither, &str); 8] = [
    (Dither::Grayscale, "Grayscale"),
    (Dither::Threshold, "Threshold"),
    (Dither::FloydSteinberg, "Floyd–Steinberg"),
    (Dither::Jarvis, "Jarvis"),
    (Dither::Stucki, "Stucki"),
    (Dither::Atkinson, "Atkinson"),
    (Dither::Ordered, "Ordered (Bayer)"),
    (Dither::Halftone, "Halftone"),
];

/// UI state of the layer list and editor.
#[derive(Default)]
pub(crate) struct LayersPanel {
    /// The layer shown in the editor.
    editing: Option<Color>,
    /// Edits in progress (a drag or typed text), committed as commands.
    draft: Option<Layer>,
    /// The layer whose settings Copy settings remembered.
    copied: Option<Color>,
}

fn set(color: Color, key: &str, value: impl Into<String>) -> Invocation {
    Invocation::new(
        "layer.set",
        [color.to_string(), key.to_owned(), value.into()],
    )
}

fn on_off(v: bool) -> &'static str {
    if v { "on" } else { "off" }
}

/// The readable name of a mode.
fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Line => "Line",
        Mode::Fill => "Fill",
    }
}

fn method_name(dither: Dither) -> &'static str {
    METHODS
        .iter()
        .find(|(d, _)| *d == dither)
        .map_or("", |(_, name)| name)
}

/// A number with at most one decimal, ".0" dropped.
fn short(v: f64) -> String {
    let text = format!("{v:.1}");
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}

/// How a swatch is drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// Plain.
    None,
    /// The current color for new shapes.
    Current,
    /// The layer is hidden on the canvas: outline only.
    Hollow,
    /// The layer is not output: faded.
    Faded,
}

/// A square swatch with its two-digit number, black or white by contrast.
fn swatch(ui: &mut egui::Ui, color: Color, side: f32, mark: Mark) -> egui::Response {
    let t = Tokens::of(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(side, side), Sense::click());
    let fill = layer_color(color);
    let painter = ui.painter();
    let number = format!("{color:02}");
    let font = egui::TextStyle::Small.resolve(ui.style());
    match mark {
        Mark::Hollow => {
            painter.rect_stroke(
                rect.shrink(1.0),
                ROUNDED,
                Stroke::new(weight::NORMAL, fill),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                number,
                font,
                t.text,
            );
        }
        _ => {
            let fill = if mark == Mark::Faded {
                fill.gamma_multiply(0.4)
            } else {
                fill
            };
            painter.rect_filled(rect, ROUNDED, fill);
            let label = theme::label_on(layer_color(color));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                number,
                font,
                label,
            );
        }
    }
    let outline = if mark == Mark::Current {
        painter.rect_stroke(
            rect.expand(1.0),
            ROUNDED,
            Stroke::new(weight::THIN, t.field),
            StrokeKind::Outside,
        );
        Stroke::new(weight::HEAVY, t.text)
    } else if response.hovered() {
        Stroke::new(weight::THIN, t.text)
    } else {
        Stroke::new(weight::THIN, t.border)
    };
    let gap = if mark == Mark::Current { 2.0 } else { 0.0 };
    painter.rect_stroke(rect.expand(gap), ROUNDED, outline, StrokeKind::Outside);
    response
}

/// The palette strip: one line of numbered swatches; those that do not fit
/// fold into a More button. A click moves the selection to that layer, or
/// with nothing selected picks the color for new shapes (returned).
pub(crate) fn palette_strip(
    ui: &mut egui::Ui,
    project: &Project,
    current: Color,
) -> (Vec<Invocation>, Option<Color>) {
    let mut out = Vec::new();
    let mut picked = None;
    let side = size::ROW - 2.0;
    let step = side + size::GAP_TIGHT + 2.0;
    let all: Vec<Color> = colors::all().map(|(c, _)| c).collect();
    let fits = ((ui.available_width() / step).floor() as usize).max(2);
    let (shown, folded) = if fits >= all.len() {
        (&all[..], &all[..0])
    } else {
        all.split_at(fits - 1)
    };
    let selection = !project.selection().is_empty();
    let mut one = |ui: &mut egui::Ui, color: Color| {
        let name = project
            .layer(color)
            .map(|l| l.name.clone())
            .unwrap_or_default();
        let action = if selection {
            "Click to put the selected shapes on this layer."
        } else {
            "Click to draw new shapes on this layer."
        };
        let mark = if color == current {
            Mark::Current
        } else {
            Mark::None
        };
        let response =
            swatch(ui, color, side, mark).on_hover_text(format!("{color:02}: {name}\n{action}"));
        if response.clicked() {
            if selection {
                out.push(Invocation::new("shape.set_layer", [color.to_string()]));
            } else {
                picked = Some(color);
            }
        }
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = size::GAP_TIGHT + 2.0;
        for color in shown {
            one(ui, *color);
        }
        if !folded.is_empty() {
            let more = ui
                .add(egui::Button::new("»").selected(folded.contains(&current)))
                .on_hover_text("More");
            egui::Popup::menu(&more).show(|ui| {
                ui.horizontal(|ui| {
                    for color in folded {
                        one(ui, *color);
                    }
                });
            });
        }
    });
    (out, picked)
}

/// One row of the layer list: a layer's vectors or its images.
struct Row<'a> {
    layer: &'a Layer,
    image: bool,
}

impl LayersPanel {
    /// Draws the list and the editor; returns the commands they produce.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, project: &Project) -> Vec<Invocation> {
        let mut out = Vec::new();
        let used = project.used_layers();
        if used.is_empty() {
            let t = Tokens::of(ui.ctx());
            ui.add_space(size::GAP);
            ui.label(egui::RichText::new("Draw a shape to start a layer.").color(t.text_muted));
            return out;
        }
        self.follow_selection(project, &used);
        let rows: Vec<Row<'_>> = used
            .iter()
            .filter_map(|c| project.layer(*c).ok())
            .flat_map(|layer| {
                let c = layer.color;
                let on = || project.shapes().iter().filter(move |s| s.layer() == c);
                let vectors = on().any(|s| !s.geometry().is_image());
                let images = on().any(|s| s.geometry().is_image());
                [
                    vectors.then_some(Row {
                        layer,
                        image: false,
                    }),
                    images.then_some(Row { layer, image: true }),
                ]
            })
            .flatten()
            .collect();
        ui.horizontal_top(|ui| {
            let list_width = ui.available_width() - size::ROW - size::GAP;
            ui.vertical(|ui| {
                ui.set_width(list_width);
                header(ui);
                for row in &rows {
                    self.list_row(ui, row, &mut out);
                }
            });
            self.side_buttons(ui, &used, &mut out);
        });
        self.settings_buttons(ui, &mut out);
        ui.separator();
        if let Some(color) = self.editing
            && let Ok(layer) = project.layer(color)
        {
            let has_images = rows.iter().any(|r| r.image && r.layer.color == color);
            let has_vectors = rows.iter().any(|r| !r.image && r.layer.color == color);
            self.editor(ui, layer, has_images, has_vectors, &mut out);
        }
        out
    }

    /// The edited layer follows the canvas: when every selected shape is on
    /// one layer, that layer is edited.
    fn follow_selection(&mut self, project: &Project, used: &[Color]) {
        let mut layers = project
            .selection()
            .into_iter()
            .filter_map(|id| project.shape(id).map(lasercraft_project::Shape::layer));
        if let Some(first) = layers.next()
            && layers.all(|c| c == first)
        {
            self.editing = Some(first);
        }
        if self.editing.is_none_or(|c| !used.contains(&c)) {
            self.editing = used.first().copied();
        }
        if self.copied.is_some_and(|c| !used.contains(&c)) {
            self.copied = None;
        }
    }

    fn list_row(&mut self, ui: &mut egui::Ui, row: &Row<'_>, out: &mut Vec<Invocation>) {
        let t = Tokens::of(ui.ctx());
        let layer = row.layer;
        let c = layer.color;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(vec2(width, size::ROW), Sense::click());
        let edited = self.editing == Some(c);
        let fill = if edited {
            t.accent_soft
        } else if response.hovered() {
            t.hover
        } else {
            t.field
        };
        ui.painter().rect_filled(rect, 0.0, fill);
        if response.clicked() {
            if ui.input(|i| i.modifiers.shift) {
                out.push(Invocation::new("layer.select", [c.to_string()]));
            } else {
                self.editing = Some(c);
            }
        }
        let text_color = if layer.output { t.text } else { t.text_muted };
        let name_width = (width - COLUMNS.iter().sum::<f32>()).max(40.0);
        let mut cells = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut cells;
        ui.spacing_mut().item_spacing.x = 0.0;
        let cell = |ui: &mut egui::Ui, w: f32, add: &mut dyn FnMut(&mut egui::Ui)| {
            ui.allocate_ui_with_layout(
                vec2(w, size::ROW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_width(w);
                    add(ui);
                },
            );
        };
        cell(ui, COLUMNS[0], &mut |ui| {
            ui.add_space(size::GAP_TIGHT);
            let mark = match (layer.show, layer.output) {
                (false, _) => Mark::Hollow,
                (true, false) => Mark::Faded,
                _ => Mark::None,
            };
            swatch(ui, c, size::ROW - 4.0, mark);
        });
        cell(ui, name_width, &mut |ui| {
            if row.image {
                ui.label(egui::RichText::new("▣ ").color(t.text_muted));
            }
            ui.add(
                egui::Label::new(egui::RichText::new(&layer.name).color(text_color))
                    .truncate()
                    .selectable(false),
            )
            .on_hover_text(format!("{}\nShift+click selects its shapes.", layer.name));
        });
        cell(ui, COLUMNS[1], &mut |ui| {
            if row.image {
                ui.label(egui::RichText::new("Image").color(text_color));
            } else {
                let mut mode = layer.mode;
                egui::ComboBox::from_id_salt(("row-mode", c))
                    .selected_text(mode_name(mode))
                    .width(COLUMNS[1] - size::GAP)
                    .show_ui(ui, |ui| {
                        for m in [Mode::Line, Mode::Fill] {
                            ui.selectable_value(&mut mode, m, mode_name(m));
                        }
                    });
                if mode != layer.mode {
                    out.push(set(c, "mode", mode.to_string()));
                }
            }
        });
        cell(ui, COLUMNS[2], &mut |ui| {
            let text = format!("{} / {}%", short(layer.speed), short(layer.max_power));
            let mut tip = format!(
                "Speed {} mm/s, Max power {}%",
                short(layer.speed),
                short(layer.max_power)
            );
            if layer.constant_power {
                tip.push_str("\nConstant power (M3): the laser may stay on while paused.");
            }
            let shown = ui.label(egui::RichText::new(text).color(text_color));
            shown.on_hover_text(&tip);
            if layer.constant_power {
                ui.label(
                    egui::RichText::new("M3")
                        .small()
                        .color(t.text)
                        .background_color(t.warning_soft),
                )
                .on_hover_text(&tip);
            }
        });
        cell(ui, COLUMNS[3], &mut |ui| {
            let mut output = layer.output;
            if widgets::switch(ui, &mut output, "")
                .on_hover_text("Output: include the layer in preview, G-code and jobs")
                .changed()
            {
                out.push(set(c, "output", on_off(output)));
            }
        });
        cell(ui, COLUMNS[4], &mut |ui| {
            let mut show = layer.show;
            if widgets::switch(ui, &mut show, "")
                .on_hover_text("Show: draw the layer on the canvas")
                .changed()
            {
                out.push(set(c, "show", on_off(show)));
            }
        });
    }

    /// Move up and Move down, beside the list.
    fn side_buttons(&self, ui: &mut egui::Ui, used: &[Color], out: &mut Vec<Invocation>) {
        let Some(color) = self.editing else {
            return;
        };
        let at = used.iter().position(|c| *c == color);
        ui.vertical(|ui| {
            ui.add_space(size::ROW);
            let square = vec2(size::ROW, size::ROW);
            for (glyph, to, tip, ok) in [
                ("⏶", "up", "Move up: run this layer earlier", at != Some(0)),
                (
                    "⏷",
                    "down",
                    "Move down: run this layer later",
                    at != Some(used.len() - 1),
                ),
            ] {
                if ui
                    .add_enabled(ok, egui::Button::new(glyph).min_size(square))
                    .on_hover_text(tip)
                    .clicked()
                {
                    out.push(Invocation::new(
                        "layer.move",
                        [color.to_string(), to.to_owned()],
                    ));
                }
            }
        });
    }

    /// Delete shapes, Copy settings and Paste settings for the edited layer.
    fn settings_buttons(&mut self, ui: &mut egui::Ui, out: &mut Vec<Invocation>) {
        let Some(color) = self.editing else {
            return;
        };
        ui.horizontal_wrapped(|ui| {
            if ui
                .button("Delete shapes")
                .on_hover_text(
                    "Remove every shape on this layer\nIts settings stay. Undo brings the shapes back.",
                )
                .clicked()
            {
                out.push(Invocation::new("layer.delete", [color.to_string()]));
            }
            if ui
                .button("Copy settings")
                .on_hover_text("Remember this layer's settings")
                .clicked()
            {
                self.copied = Some(color);
            }
            let paste = self.copied.filter(|from| *from != color);
            let tip = paste.map_or_else(
                || "Copy another layer's settings first.".to_owned(),
                |from| format!("Apply the settings of layer {from:02} here\nEvery setting except the name."),
            );
            if ui
                .add_enabled(paste.is_some(), egui::Button::new("Paste settings"))
                .on_hover_text(&tip)
                .on_disabled_hover_text(&tip)
                .clicked()
                && let Some(from) = paste
            {
                out.push(Invocation::new(
                    "layer.copy_settings",
                    [from.to_string(), color.to_string()],
                ));
            }
        });
    }

    fn editor(
        &mut self,
        ui: &mut egui::Ui,
        layer: &Layer,
        has_images: bool,
        has_vectors: bool,
        out: &mut Vec<Invocation>,
    ) {
        // Keep the draft in step with the project unless the user is in the
        // middle of an edit.
        let editing_now = ui.ctx().dragged_id().is_some() || ui.ctx().text_edit_focused();
        if !editing_now || self.draft.as_ref().is_none_or(|d| d.color != layer.color) {
            self.draft = Some(layer.clone());
        }
        let Some(draft) = self.draft.as_mut() else {
            return;
        };
        let c = draft.color;
        let mut edits: Vec<(&str, String)> = Vec::new();
        name_line(ui, draft, layer, &mut edits);
        let pairs = if ui.available_width() >= 380.0 { 2 } else { 1 };
        let mut form = Form::new("layer_editor", pairs);
        form.show(ui, |f, ui| {
            if has_vectors {
                f.pair(ui, "Mode", "Line traces outlines; Fill scans inside closed shapes\nOpen shapes on a fill layer are skipped.", |ui| {
                    let before = draft.mode;
                    egui::ComboBox::from_id_salt("mode")
                        .selected_text(mode_name(draft.mode))
                        .show_ui(ui, |ui| {
                            for mode in [Mode::Line, Mode::Fill] {
                                ui.selectable_value(&mut draft.mode, mode, mode_name(mode));
                            }
                        });
                    if draft.mode != before {
                        edits.push(("mode", draft.mode.to_string()));
                    }
                });
            } else {
                f.pair(ui, "Mode", "Images are engraved line by line.", |ui| {
                    ui.label("Image");
                });
            }
            number(f, ui, "Speed (mm/s)", "Top speed for this layer. Above 0 to 10 000.\nShort moves may not reach it.", &mut draft.speed, (0.1, 10_000.0), "speed", &mut edits);
            let mut passes = f64::from(draft.passes);
            number(f, ui, "Passes", "How many times the layer runs. 1–1000.", &mut passes, (1.0, 1000.0), "passes", &mut edits);
            number(f, ui, "Max power (%)", "Peak power for this layer. 0–100.\nNever above the device's Max power (%).", &mut draft.max_power, (0.0, 100.0), "max_power", &mut edits);
            switch(f, ui, "Air assist", "Turn air assist on while this layer runs.", &mut draft.air, "air", &mut edits);
            switch(f, ui, "Constant power (M3)", "Keep the set power while the head slows or stops (M3).\nOff uses dynamic power (M4), which turns the laser off when the head stops.", &mut draft.constant_power, "constant_power", &mut edits);
            if has_vectors && draft.mode == Mode::Line {
                line_settings(f, ui, draft, &mut edits);
            }
            if has_vectors && draft.mode == Mode::Fill {
                fill_settings(f, ui, draft, &mut edits);
            }
        });
        if draft.constant_power {
            widgets::band(
                ui,
                Band::Warning,
                "The laser may stay on while the job is paused. Stop turns it off.",
            );
        }
        egui::CollapsingHeader::new("Images on this layer")
            .default_open(has_images)
            .open(has_images.then_some(true))
            .show(ui, |ui| {
                let mut form = Form::new("layer_image", pairs);
                form.show(ui, |f, ui| {
                    image_settings(f, ui, draft, has_vectors, &mut edits)
                });
            });
        // Passes and tabs are whole numbers.
        for edit in &mut edits {
            if matches!(edit.0, "passes" | "tabs")
                && let Ok(v) = edit.1.parse::<f64>()
            {
                edit.1 = v.round().to_string();
            }
        }
        out.extend(edits.into_iter().map(|(key, value)| set(c, key, value)));
    }
}

fn header(ui: &mut egui::Ui) {
    let t = Tokens::of(ui.ctx());
    let width = ui.available_width();
    let name_width = (width - COLUMNS.iter().sum::<f32>()).max(40.0);
    let widths = [
        COLUMNS[0], name_width, COLUMNS[1], COLUMNS[2], COLUMNS[3], COLUMNS[4],
    ];
    let (rect, _) = ui.allocate_exact_size(vec2(width, size::ROW), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, t.field_alt);
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(weight::THIN, t.border),
    );
    let mut x = rect.left();
    for ((caption, tip), w) in HEADER.iter().zip(widths) {
        let cell = egui::Rect::from_min_size(egui::pos2(x, rect.top()), vec2(w, size::ROW));
        ui.painter().text(
            cell.left_center() + vec2(size::GAP_TIGHT, 0.0),
            egui::Align2::LEFT_CENTER,
            *caption,
            egui::TextStyle::Small.resolve(ui.style()),
            t.text_muted,
        );
        ui.interact(
            cell,
            ui.make_persistent_id(("layer-header", *caption)),
            Sense::hover(),
        )
        .on_hover_text(*tip);
        x += w;
    }
}

/// The header line: the edited layer's chip and its name.
fn name_line<'a>(
    ui: &mut egui::Ui,
    draft: &mut Layer,
    layer: &Layer,
    edits: &mut Vec<(&'a str, String)>,
) {
    ui.horizontal(|ui| {
        swatch(ui, draft.color, size::ROW - 2.0, Mark::None);
        ui.label("Name");
        let field = ui
            .add(egui::TextEdit::singleline(&mut draft.name).desired_width(f32::INFINITY))
            .on_hover_text("Name shown in the list and in messages");
        if field.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            draft.name.clone_from(&layer.name);
        }
        if field.lost_focus() && draft.name != layer.name {
            if draft.name.trim().is_empty() {
                draft.name.clone_from(&layer.name);
            } else {
                edits.push(("name", draft.name.clone()));
            }
        }
    });
}

/// A form grid with one or two label/field pairs per line; labels are
/// right-aligned and carry the unit.
struct Form {
    id: &'static str,
    pairs: usize,
    placed: usize,
}

impl Form {
    fn new(id: &'static str, pairs: usize) -> Self {
        Self {
            id,
            pairs,
            placed: 0,
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, add: impl FnOnce(&mut Self, &mut egui::Ui)) {
        egui::Grid::new(self.id)
            .num_columns(self.pairs * 2)
            .spacing([size::GAP, size::GAP_TIGHT])
            .show(ui, |ui| {
                add(self, ui);
                if self.placed % self.pairs != 0 {
                    ui.end_row();
                }
            });
    }

    /// One label/field pair.
    fn pair(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        tip: &str,
        field: impl FnOnce(&mut egui::Ui),
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(label).on_hover_text(tip);
        });
        ui.horizontal(|ui| {
            field(ui);
        })
        .response
        .on_hover_text(tip);
        self.placed += 1;
        if self.placed % self.pairs == 0 {
            ui.end_row();
        }
    }

    /// An empty pair, to keep a field in the right-hand column.
    fn skip(&mut self, ui: &mut egui::Ui) {
        if self.pairs > 1 {
            ui.label("");
            ui.label("");
            self.placed += 1;
            if self.placed % self.pairs == 0 {
                ui.end_row();
            }
        }
    }
}

/// The image method and its settings (M7, `docs/spec/images.md`).
fn image_settings(
    f: &mut Form,
    ui: &mut egui::Ui,
    draft: &mut Layer,
    has_vectors: bool,
    edits: &mut Vec<(&str, String)>,
) {
    f.pair(
        ui,
        "Method",
        "How the image is turned into dots or shades",
        |ui| {
            let before = draft.image.dither;
            egui::ComboBox::from_id_salt("dither")
                .selected_text(method_name(draft.image.dither))
                .show_ui(ui, |ui| {
                    for (d, name) in METHODS {
                        ui.selectable_value(&mut draft.image.dither, d, name);
                    }
                });
            if draft.image.dither != before {
                edits.push(("image.dither", draft.image.dither.to_string()));
            }
        },
    );
    let dpi = 25.4 / draft.image.interval;
    number(
        f,
        ui,
        "Interval (mm)",
        &format!("Distance between image lines. 0.001–10.\n{dpi:.0} dpi."),
        &mut draft.image.interval,
        (0.001, 10.0),
        "image.interval",
        edits,
    );
    switch(
        f,
        ui,
        "Negative",
        "Burn the light parts instead of the dark ones.",
        &mut draft.image.negative,
        "image.negative",
        edits,
    );
    let grayscale = draft.image.dither == Dither::Grayscale;
    let max_power = draft.max_power;
    number_enabled(
        f,
        ui,
        grayscale,
        "Min power (%)",
        "Power for the lightest shade of a grayscale image. 0 to Max power (%).",
        &mut draft.min_power,
        (0.0, max_power),
        "min_power",
        edits,
    );
    let interval = draft.image.interval;
    number(
        f,
        ui,
        "Beam compensation (mm)",
        "Shorten each burned dash so neighbouring dots do not merge.",
        &mut draft.image.dot_correction,
        (0.0, interval),
        "image.dot_correction",
        edits,
    );
    f.skip(ui);
    if draft.image.dither == Dither::Halftone {
        number(
            f,
            ui,
            "Halftone frequency (lpi)",
            "Halftone cells per inch.",
            &mut draft.image.halftone_cells,
            lasercraft_project::layer::HALFTONE_CELLS_RANGE,
            "image.halftone_cells",
            edits,
        );
        number(
            f,
            ui,
            "Halftone angle (°)",
            "Angle of the halftone screen.",
            &mut draft.image.halftone_angle,
            (-360.0, 360.0),
            "image.halftone_angle",
            edits,
        );
    }
    if !has_vectors || draft.mode != Mode::Fill {
        // Shared with fill; shown there for fill layers.
        switch(
            f,
            ui,
            "Both directions",
            "Burn on the way back too; off returns with the laser off.",
            &mut draft.bidirectional,
            "bidirectional",
            edits,
        );
        switch(
            f,
            ui,
            "Overscan",
            "Run on past the line ends with the laser off.",
            &mut draft.overscan,
            "overscan",
            edits,
        );
    }
}

/// Kerf offset and tabs (line mode, M4).
fn line_settings(
    f: &mut Form,
    ui: &mut egui::Ui,
    draft: &mut Layer,
    edits: &mut Vec<(&str, String)>,
) {
    let kerf = lasercraft_project::layer::MAX_KERF;
    number(
        f,
        ui,
        "Kerf offset (mm)",
        "Shift the cut on closed shapes to allow for the beam width. −5 to 5.\nPositive moves the cut outward from the part.",
        &mut draft.kerf,
        (-kerf, kerf),
        "kerf",
        edits,
    );
    let mut tabs = f64::from(draft.tabs);
    number(
        f,
        ui,
        "Tabs per shape",
        "Uncut bridges left on each closed outer shape. 0 is off.",
        &mut tabs,
        (0.0, f64::from(lasercraft_project::layer::MAX_TABS)),
        "tabs",
        edits,
    );
    if draft.tabs > 0 {
        f.skip(ui);
        number(
            f,
            ui,
            "Tab width (mm)",
            "Length of each bridge. 0.01–50.",
            &mut draft.tab_width,
            lasercraft_project::layer::TAB_WIDTH_RANGE,
            "tab_width",
            edits,
        );
    }
}

/// Interval, angle and scanning switches (fill mode).
fn fill_settings(
    f: &mut Form,
    ui: &mut egui::Ui,
    draft: &mut Layer,
    edits: &mut Vec<(&str, String)>,
) {
    let lpi = 25.4 / draft.interval;
    number(
        f,
        ui,
        "Interval (mm)",
        &format!("Distance between scan lines. 0.001–10.\n{lpi:.0} lines per inch."),
        &mut draft.interval,
        (0.001, 10.0),
        "interval",
        edits,
    );
    number(
        f,
        ui,
        "Scan angle (°)",
        "Direction of the scan lines; 0 is horizontal.",
        &mut draft.angle,
        (-360.0, 360.0),
        "angle",
        edits,
    );
    switch(
        f,
        ui,
        "Both directions",
        "Burn on the way back too; off returns with the laser off.",
        &mut draft.bidirectional,
        "bidirectional",
        edits,
    );
    switch(
        f,
        ui,
        "Overscan",
        "Run on past the line ends with the laser off.",
        &mut draft.overscan,
        "overscan",
        edits,
    );
    switch(
        f,
        ui,
        "Crosshatch",
        "Add a second pass at 90°.",
        &mut draft.crosshatch,
        "crosshatch",
        edits,
    );
}

/// Decimals shown for a setting (`docs/spec/style/naming.md`, "Numbers").
fn decimals(key: &str) -> usize {
    match key {
        "passes" | "tabs" | "image.halftone_cells" => 0,
        "speed" | "max_power" | "min_power" => 1,
        "angle" | "image.halftone_angle" => 2,
        _ => 3,
    }
}

/// A labelled number field. Dragging previews in the draft; the change is
/// committed when the drag ends or a typed value is entered.
#[allow(clippy::too_many_arguments)]
fn number<'a>(
    f: &mut Form,
    ui: &mut egui::Ui,
    label: &str,
    tip: &str,
    value: &mut f64,
    range: (f64, f64),
    key: &'a str,
    edits: &mut Vec<(&'a str, String)>,
) {
    number_enabled(f, ui, true, label, tip, value, range, key, edits);
}

#[allow(clippy::too_many_arguments)]
fn number_enabled<'a>(
    f: &mut Form,
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    tip: &str,
    value: &mut f64,
    (min, max): (f64, f64),
    key: &'a str,
    edits: &mut Vec<(&'a str, String)>,
) {
    f.pair(ui, label, tip, |ui| {
        let r = ui.add_enabled(
            enabled,
            DragValue::new(value)
                .range(min..=max)
                .speed((max - min) / 1000.0)
                .max_decimals(decimals(key)),
        );
        if r.drag_stopped() || (r.changed() && !r.dragged()) {
            edits.push((key, value.to_string()));
        }
    });
}

fn switch<'a>(
    f: &mut Form,
    ui: &mut egui::Ui,
    label: &str,
    tip: &str,
    value: &mut bool,
    key: &'a str,
    edits: &mut Vec<(&'a str, String)>,
) {
    f.pair(ui, label, tip, |ui| {
        if widgets::switch(ui, value, "").changed() {
            edits.push((key, on_off(*value).to_owned()));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swatch_numbers_meet_the_text_contrast_target() {
        for (color, _) in colors::all() {
            let fill = layer_color(color);
            let ratio = theme::contrast(theme::label_on(fill), fill);
            assert!(ratio >= 4.5, "{color:02}: {ratio:.2}");
        }
    }

    #[test]
    fn readable_names_cover_every_method_and_mode() {
        for (d, name) in METHODS {
            assert_eq!(method_name(d), name);
        }
        assert_eq!(method_name(Dither::FloydSteinberg), "Floyd–Steinberg");
        assert_eq!(mode_name(Mode::Fill), "Fill");
        assert_eq!(short(12.5), "12.5");
        assert_eq!(short(20.0), "20");
    }
}
