//! The layer list, the layer settings editor and the palette strip
//! (`docs/spec/layers.md`, `docs/spec/ui-shell.md`). Every change is emitted
//! as a command; nothing here edits the project.

use egui::{DragValue, Sense, Stroke, StrokeKind, vec2};
use lasercraft_commands::Invocation;
use lasercraft_project::{Color, Dither, Layer, Mode, Project};

use crate::colors::{self, layer_color};

/// UI state of the layer list and editor.
#[derive(Default)]
pub(crate) struct LayersPanel {
    /// The layer shown in the editor.
    editing: Option<Color>,
    /// Edits in progress (a drag or typed text), committed as commands.
    draft: Option<Layer>,
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

fn swatch(ui: &mut egui::Ui, color: Color, size: f32, marked: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());
    ui.painter().rect_filled(rect, 2.0, layer_color(color));
    let outline = if marked {
        Stroke::new(2.0, ui.visuals().strong_text_color())
    } else {
        Stroke::new(1.0, ui.visuals().weak_text_color())
    };
    ui.painter()
        .rect_stroke(rect, 2.0, outline, StrokeKind::Outside);
    response
}

/// The palette strip. A click moves the selection to that layer, or with
/// nothing selected picks the color for new shapes (returned).
pub(crate) fn palette_strip(
    ui: &mut egui::Ui,
    project: &Project,
    current: Color,
) -> (Vec<Invocation>, Option<Color>) {
    let mut out = Vec::new();
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        for (color, _) in colors::all() {
            let name = project
                .layer(color)
                .map(|l| l.name.clone())
                .unwrap_or_default();
            let response =
                swatch(ui, color, 18.0, color == current).on_hover_text(format!("{color}: {name}"));
            if response.clicked() {
                if project.selection().is_empty() {
                    picked = Some(color);
                } else {
                    out.push(Invocation::new("shape.set_layer", [color.to_string()]));
                }
            }
        }
    });
    (out, picked)
}

impl LayersPanel {
    /// Draws the list and the editor; returns the commands they produce.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, project: &Project) -> Vec<Invocation> {
        let mut out = Vec::new();
        ui.heading("Layers");
        let used = project.used_layers();
        if used.is_empty() {
            ui.weak("Draw a shape to start a layer.");
        }
        if self.editing.is_none_or(|c| !used.contains(&c)) {
            self.editing = used.first().copied();
        }
        egui::Grid::new("layer_list")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                for color in &used {
                    if let Ok(layer) = project.layer(*color) {
                        self.list_row(ui, project, layer, &mut out);
                    }
                }
            });
        if let Some(color) = self.editing {
            ui.horizontal(|ui| {
                for (label, to) in [("Move up", "up"), ("Move down", "down")] {
                    if ui.small_button(label).clicked() {
                        out.push(Invocation::new(
                            "layer.move",
                            [color.to_string(), to.to_owned()],
                        ));
                    }
                }
                if ui.small_button("Delete shapes").clicked() {
                    out.push(Invocation::new("layer.delete", [color.to_string()]));
                }
            });
            ui.separator();
            if let Ok(layer) = project.layer(color) {
                let has_images = project
                    .shapes()
                    .iter()
                    .any(|s| s.layer() == color && s.geometry().is_image());
                self.editor(ui, layer, has_images, &mut out);
            }
        }
        out
    }

    fn list_row(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        layer: &Layer,
        out: &mut Vec<Invocation>,
    ) {
        let c = layer.color;
        let on_layer = || project.shapes().iter().filter(move |s| s.layer() == c);
        let images = on_layer().any(|s| s.geometry().is_image());
        let vectors = on_layer().any(|s| !s.geometry().is_image());
        let kind = match (vectors, images) {
            (true, true) => format!("{} + image", layer.mode),
            (false, true) => "image".to_owned(),
            _ => layer.mode.to_string(),
        };
        swatch(ui, c, 14.0, false);
        let label = ui
            .selectable_label(self.editing == Some(c), &layer.name)
            .on_hover_text("Click to edit; Shift+click selects its shapes");
        if label.clicked() {
            if ui.input(|i| i.modifiers.shift) {
                out.push(Invocation::new("layer.select", [c.to_string()]));
            } else {
                self.editing = Some(c);
            }
        }
        ui.label(format!("{kind} {} mm/s {}%", layer.speed, layer.max_power));
        let mut output = layer.output;
        if ui
            .checkbox(&mut output, "")
            .on_hover_text("Output")
            .changed()
        {
            out.push(set(c, "output", on_off(output)));
        }
        let mut show = layer.show;
        if ui.checkbox(&mut show, "").on_hover_text("Show").changed() {
            out.push(set(c, "show", on_off(show)));
        }
        ui.end_row();
    }

    fn editor(
        &mut self,
        ui: &mut egui::Ui,
        layer: &Layer,
        has_images: bool,
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
        egui::Grid::new("layer_editor")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Name");
                let name = ui.text_edit_singleline(&mut draft.name);
                if name.lost_focus() && draft.name != layer.name {
                    edits.push(("name", draft.name.clone()));
                }
                ui.end_row();

                ui.label("Mode");
                let before = draft.mode;
                egui::ComboBox::from_id_salt("mode")
                    .selected_text(draft.mode.as_str())
                    .show_ui(ui, |ui| {
                        for mode in [Mode::Line, Mode::Fill] {
                            ui.selectable_value(&mut draft.mode, mode, mode.as_str());
                        }
                    });
                if draft.mode != before {
                    edits.push(("mode", draft.mode.to_string()));
                }
                ui.end_row();

                number(
                    ui,
                    "Speed",
                    &mut draft.speed,
                    (0.1, 10_000.0),
                    " mm/s",
                    "speed",
                    &mut edits,
                );
                number(
                    ui,
                    "Max power",
                    &mut draft.max_power,
                    (0.0, 100.0),
                    " %",
                    "max_power",
                    &mut edits,
                );
                // Min power cannot exceed max power.
                let max_power = draft.max_power;
                number(
                    ui,
                    "Min power",
                    &mut draft.min_power,
                    (0.0, max_power),
                    " %",
                    "min_power",
                    &mut edits,
                );
                let mut passes = f64::from(draft.passes);
                number(
                    ui,
                    "Passes",
                    &mut passes,
                    (1.0, 1000.0),
                    "",
                    "passes",
                    &mut edits,
                );
                if let Some(last) = edits.last_mut().filter(|e| e.0 == "passes") {
                    // Passes are whole numbers.
                    last.1 = format!("{}", passes.round());
                }
                switch(ui, "Air assist", &mut draft.air, "air", &mut edits);
                if draft.mode == Mode::Line {
                    line_settings(ui, draft, &mut edits);
                }
                if draft.mode == Mode::Fill {
                    number(
                        ui,
                        "Interval",
                        &mut draft.interval,
                        (0.001, 10.0),
                        " mm",
                        "interval",
                        &mut edits,
                    );
                    ui.label("");
                    ui.weak(format!("{:.0} lines per inch", 25.4 / draft.interval));
                    ui.end_row();
                    number(
                        ui,
                        "Scan angle",
                        &mut draft.angle,
                        (-360.0, 360.0),
                        "°",
                        "angle",
                        &mut edits,
                    );
                    switch(
                        ui,
                        "Both directions",
                        &mut draft.bidirectional,
                        "bidirectional",
                        &mut edits,
                    );
                    switch(ui, "Overscan", &mut draft.overscan, "overscan", &mut edits);
                    switch(
                        ui,
                        "Crosshatch",
                        &mut draft.crosshatch,
                        "crosshatch",
                        &mut edits,
                    );
                }
            });
        egui::CollapsingHeader::new("Images on this layer")
            .default_open(has_images)
            .show(ui, |ui| image_settings(ui, draft, &mut edits));
        out.extend(edits.into_iter().map(|(key, value)| set(c, key, value)));
    }
}

/// The image method and its settings (M7, `docs/spec/images.md`).
fn image_settings(ui: &mut egui::Ui, draft: &mut Layer, edits: &mut Vec<(&str, String)>) {
    egui::Grid::new("layer_image")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Method");
            let before = draft.image.dither;
            egui::ComboBox::from_id_salt("dither")
                .selected_text(draft.image.dither.as_str())
                .show_ui(ui, |ui| {
                    for d in [
                        Dither::Grayscale,
                        Dither::Threshold,
                        Dither::FloydSteinberg,
                        Dither::Jarvis,
                        Dither::Stucki,
                        Dither::Atkinson,
                        Dither::Ordered,
                        Dither::Halftone,
                    ] {
                        ui.selectable_value(&mut draft.image.dither, d, d.as_str());
                    }
                });
            if draft.image.dither != before {
                edits.push(("image.dither", draft.image.dither.to_string()));
            }
            ui.end_row();
            number(
                ui,
                "Interval",
                &mut draft.image.interval,
                (0.001, 10.0),
                " mm",
                "image.interval",
                edits,
            );
            ui.label("");
            ui.weak(format!("{:.0} DPI", 25.4 / draft.image.interval));
            ui.end_row();
            switch(
                ui,
                "Negative",
                &mut draft.image.negative,
                "image.negative",
                edits,
            );
            if draft.mode != Mode::Fill {
                // Shared with fill; shown there for fill layers.
                switch(
                    ui,
                    "Both directions",
                    &mut draft.bidirectional,
                    "bidirectional",
                    edits,
                );
                switch(ui, "Overscan", &mut draft.overscan, "overscan", edits);
            }
            let interval = draft.image.interval;
            number(
                ui,
                "Dot correction",
                &mut draft.image.dot_correction,
                (0.0, interval),
                " mm",
                "image.dot_correction",
                edits,
            );
            if draft.image.dither == Dither::Halftone {
                number(
                    ui,
                    "Halftone cells",
                    &mut draft.image.halftone_cells,
                    lasercraft_project::layer::HALFTONE_CELLS_RANGE,
                    " /in",
                    "image.halftone_cells",
                    edits,
                );
                number(
                    ui,
                    "Halftone angle",
                    &mut draft.image.halftone_angle,
                    (-360.0, 360.0),
                    "°",
                    "image.halftone_angle",
                    edits,
                );
            }
        });
}

/// Kerf offset and tabs (line mode, M4).
fn line_settings(ui: &mut egui::Ui, draft: &mut Layer, edits: &mut Vec<(&str, String)>) {
    number(
        ui,
        "Kerf offset",
        &mut draft.kerf,
        (
            -lasercraft_project::layer::MAX_KERF,
            lasercraft_project::layer::MAX_KERF,
        ),
        " mm",
        "kerf",
        edits,
    );
    let mut tabs = f64::from(draft.tabs);
    number(
        ui,
        "Tabs per shape",
        &mut tabs,
        (0.0, f64::from(lasercraft_project::layer::MAX_TABS)),
        "",
        "tabs",
        edits,
    );
    if draft.tabs > 0 {
        number(
            ui,
            "Tab width",
            &mut draft.tab_width,
            lasercraft_project::layer::TAB_WIDTH_RANGE,
            " mm",
            "tab_width",
            edits,
        );
    }
}

/// A labelled number field. Dragging previews in the draft; the change is
/// committed when the drag ends or a typed value is entered.
fn number<'a>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    (min, max): (f64, f64),
    suffix: &str,
    key: &'a str,
    edits: &mut Vec<(&'a str, String)>,
) {
    // Passes and tabs are whole numbers; everything else allows three
    // decimals.
    let whole = matches!(key, "passes" | "tabs");
    let decimals = if whole { 0 } else { 3 };
    ui.label(label);
    let r = ui.add(
        DragValue::new(value)
            .range(min..=max)
            .suffix(suffix)
            .speed((max - min) / 1000.0)
            .max_decimals(decimals),
    );
    if r.drag_stopped() || (r.changed() && !r.dragged()) {
        let text = if whole {
            value.round().to_string()
        } else {
            value.to_string()
        };
        edits.push((key, text));
    }
    ui.end_row();
}

fn switch<'a>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut bool,
    key: &'a str,
    edits: &mut Vec<(&'a str, String)>,
) {
    ui.label(label);
    if ui.checkbox(value, "").changed() {
        edits.push((key, on_off(*value).to_owned()));
    }
    ui.end_row();
}
