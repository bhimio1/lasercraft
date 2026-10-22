//! The Speed and power test and Line interval test windows
//! (`docs/spec/test-patterns.md`, "Windows (app)"; `docs/spec/style/
//! later-windows.md`, "Test patterns"). The window edits the pattern's
//! settings as JSON; the app checks and plans them ([`TestRequest`],
//! [`TestInfo`]), and every job goes through `device.test`,
//! `device.test_frame` or `test.export`.

use lasercraft_commands::Invocation;
use lasercraft_project::Project;
use lasercraft_workshop::Workshop;
use serde_json::{Value, json};

use crate::keymap::Keymap;
use crate::laser_panel::{self, Enabled, LaserPanel, LaserView};
use crate::preview::Preview;
use crate::shell::{Action, PromptKind};
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Band};

/// Which test a window runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    /// The speed and power grid.
    SpeedPower,
    /// The line interval patches.
    Interval,
}

impl TestKind {
    /// The pattern's argument spelling.
    #[must_use]
    pub fn arg(self) -> &'static str {
        match self {
            Self::SpeedPower => "speed_power",
            Self::Interval => "interval",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::SpeedPower => "Speed and power test",
            Self::Interval => "Line interval test",
        }
    }
}

/// The settings a window wants checked (and planned, for the preview).
#[derive(Debug, Clone, PartialEq)]
pub struct TestRequest {
    /// Which test.
    pub kind: TestKind,
    /// The settings as the window holds them (may be partial).
    pub settings: Value,
    /// Answer with [`TestInfo::revision`] set to this.
    pub revision: u64,
    /// No field is being dragged: the settings may be remembered.
    pub settled: bool,
    /// Also build the job preview.
    pub preview: bool,
}

/// The app's answer to a [`TestRequest`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TestInfo {
    /// The request's revision.
    pub revision: u64,
    /// The complete settings, when they are valid.
    pub settings: Option<Value>,
    /// Pattern size, mm.
    pub size: Option<(f64, f64)>,
    /// Why the pattern cannot run (invalid settings, outside the bed).
    pub error: Option<String>,
    /// Planner warnings (power clamped and the like).
    pub warnings: Vec<String>,
    /// The planned pattern, when asked for.
    pub preview: Option<Preview>,
}

/// One test window.
pub(crate) struct TestWindow {
    kind: TestKind,
    settings: Value,
    revision: u64,
    sent: Option<(u64, bool)>,
    info: TestInfo,
    want_preview: bool,
    preset: Option<String>,
    save_preset: Option<String>,
    part: Option<&'static str>,
    /// A path the window wants asked for; the shell takes it.
    pub(crate) prompt: Option<PromptKind>,
}

fn get(v: &Value, path: &str) -> f64 {
    v.pointer(path).and_then(Value::as_f64).unwrap_or_default()
}

fn set(v: &mut Value, path: &str, new: Value) {
    let mut at = v;
    let keys: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    for (i, key) in keys.iter().enumerate() {
        if !at.is_object() {
            *at = json!({});
        }
        let map = at
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("made an object"));
        if i + 1 == keys.len() {
            map.insert((*key).to_owned(), new);
            return;
        }
        at = map.entry((*key).to_owned()).or_insert_with(|| json!({}));
    }
}

const SETTINGS: [(&str, &str); 4] = [
    ("speed", "Speed"),
    ("power", "Power"),
    ("interval", "Interval"),
    ("passes", "Passes"),
];

fn unit(setting: &str) -> &'static str {
    match setting {
        "speed" => " (mm/s)",
        "power" => " (%)",
        "interval" => " (mm)",
        _ => "",
    }
}

impl TestWindow {
    pub(crate) fn new(kind: TestKind, workshop: &Workshop) -> Self {
        Self {
            kind,
            settings: workshop
                .last_settings(kind.arg())
                .cloned()
                .unwrap_or_else(|| json!({})),
            revision: 1,
            sent: None,
            info: TestInfo::default(),
            want_preview: false,
            preset: None,
            save_preset: None,
            part: None,
            prompt: None,
        }
    }

    pub(crate) fn kind(&self) -> TestKind {
        self.kind
    }

    /// The request to send, if the settings changed since the last one.
    pub(crate) fn request(&mut self, settled: bool) -> Option<TestRequest> {
        if self.sent == Some((self.revision, settled)) && !self.want_preview {
            return None;
        }
        self.sent = Some((self.revision, settled));
        Some(TestRequest {
            kind: self.kind,
            settings: self.settings.clone(),
            revision: self.revision,
            settled,
            preview: std::mem::take(&mut self.want_preview),
        })
    }

    /// Takes the app's answer; returns the preview it carries.
    pub(crate) fn set_info(&mut self, mut info: TestInfo) -> Option<Preview> {
        if info.revision != self.revision {
            return None;
        }
        if let Some(full) = &info.settings {
            self.settings = full.clone();
        }
        let preview = info.preview.take();
        self.info = info;
        preview
    }

    fn changed(&mut self) {
        self.revision += 1;
        self.preset = None;
    }

    fn json(&self) -> String {
        self.settings.to_string()
    }

    fn number(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        path: &str,
        range: std::ops::RangeInclusive<f64>,
        whole: bool,
    ) {
        right_label(ui, label);
        let mut value = get(&self.settings, path);
        let drag = egui::DragValue::new(&mut value)
            .range(range)
            .max_decimals(if whole { 0 } else { 3 })
            .clamp_existing_to_range(false);
        if ui.add(drag).changed() {
            let new = if whole {
                json!(value.round() as u64)
            } else {
                json!(value)
            };
            set(&mut self.settings, path, new);
            self.changed();
        }
        ui.end_row();
    }

    fn switch(&mut self, ui: &mut egui::Ui, label: &str, key: &str) {
        let mut on = self
            .settings
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if widgets::switch(ui, &mut on, label).changed() {
            set(&mut self.settings, &format!("/{key}"), json!(on));
            self.changed();
        }
    }

    /// Draws the window. Returns false once closed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        workshop: &Workshop,
        laser: &LaserPanel,
        view: &LaserView,
        keymap: &Keymap,
        actions: &mut Vec<Action>,
    ) -> bool {
        let mut open = true;
        let mut close = false;
        egui::Window::new(self.kind.title())
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_max_width(520.0);
                if self.kind == TestKind::SpeedPower {
                    self.preset_row(ui, workshop, actions);
                    ui.add_space(size::GAP_TIGHT);
                }
                if self.info.settings.is_none() && self.info.error.is_none() {
                    ui.label("Checking the settings…");
                    return;
                }
                let absolute = laser.absolute();
                match self.kind {
                    TestKind::SpeedPower => self.grid_fields(ui, project, absolute),
                    TestKind::Interval => self.interval_fields(ui, project, absolute),
                }
                if self.kind == TestKind::SpeedPower {
                    self.parts_row(ui);
                } else {
                    self.switch(ui, "Draw labels", "draw_labels");
                }
                ui.add_space(size::GAP);
                self.run_group(ui, laser, view, keymap, actions);
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui
                        .add(egui::Button::new("Close").min_size(egui::vec2(80.0, size::ROW)))
                        .on_hover_text("Close the window; a running test keeps running")
                        .clicked();
                });
            });
        self.part_window(ctx);
        self.preset_window(ctx, actions);
        open && !close
    }

    fn preset_row(&mut self, ui: &mut egui::Ui, workshop: &Workshop, actions: &mut Vec<Action>) {
        let presets: Vec<_> = workshop
            .presets()
            .into_iter()
            .filter(|(p, _)| p.kind == self.kind.arg())
            .collect();
        ui.horizontal_wrapped(|ui| {
            ui.label("Preset");
            let shown = self.preset.clone().unwrap_or_else(|| "(none)".to_owned());
            egui::ComboBox::from_id_salt(("test-preset", self.kind.arg()))
                .selected_text(shown)
                .width(150.0)
                .show_ui(ui, |ui| {
                    let t = Tokens::of(ui.ctx());
                    for (heading, built_in) in [("Built-in", true), ("Yours", false)] {
                        ui.label(egui::RichText::new(heading).small().color(t.text_muted));
                        for (p, _) in presets.iter().filter(|(_, b)| *b == built_in) {
                            let on = self.preset.as_deref() == Some(p.name.as_str());
                            if ui.selectable_label(on, &p.name).clicked() {
                                self.settings = p.settings.clone();
                                self.changed();
                                self.preset = Some(p.name.clone());
                            }
                        }
                    }
                });
            if ui.button("Save preset…").clicked() {
                self.save_preset = Some(String::new());
            }
            let yours = presets
                .iter()
                .any(|(p, b)| !b && self.preset.as_deref() == Some(p.name.as_str()));
            if ui
                .add_enabled(yours, egui::Button::new("Delete preset"))
                .on_disabled_hover_text("Choose one of your presets first.")
                .clicked()
                && let Some(name) = self.preset.take()
            {
                actions.push(Action::Run(Invocation::new(
                    "test.preset_delete",
                    [name, self.kind.arg().to_owned()],
                )));
            }
            if ui.button("Import presets…").clicked() {
                self.prompt = Some(PromptKind::Run {
                    title: "Import presets",
                    what: "Path to a .lctests file:",
                    command: Invocation::bare("test.preset_import"),
                    at: 0,
                });
            }
            if ui.button("Export presets…").clicked() {
                self.prompt = Some(PromptKind::Run {
                    title: "Export presets",
                    what: "Path to the .lctests file to write:",
                    command: Invocation::bare("test.preset_export"),
                    at: 0,
                });
            }
        });
    }

    fn center(&mut self, ui: &mut egui::Ui, project: &Project, axis: usize, absolute: bool) {
        let label = if axis == 0 {
            "Center X (mm)"
        } else {
            "Center Y (mm)"
        };
        right_label(ui, label);
        let area = project.work_area();
        let bed = [area.width_mm / 2.0, area.height_mm / 2.0];
        let mut c = self
            .settings
            .get("center")
            .and_then(Value::as_array)
            .map_or(bed, |a| {
                [
                    a.first().and_then(Value::as_f64).unwrap_or(bed[0]),
                    a.get(1).and_then(Value::as_f64).unwrap_or(bed[1]),
                ]
            });
        let response = ui
            .add_enabled(
                absolute,
                egui::DragValue::new(&mut c[axis])
                    .range(0.0..=10_000.0)
                    .max_decimals(2)
                    .clamp_existing_to_range(false),
            )
            .on_disabled_hover_text(
                "Place job at and the job reference point place the pattern (Machine panel).",
            );
        if response.changed() {
            set(&mut self.settings, "/center", json!(c));
            self.changed();
        }
        ui.end_row();
    }

    fn grid_fields(&mut self, ui: &mut egui::Ui, project: &Project, absolute: bool) {
        ui.horizontal_top(|ui| {
            for (axis, title, size_label) in [
                ("rows", "Rows", "Box height (mm)"),
                ("columns", "Columns", "Box width (mm)"),
            ] {
                group(ui, title, |ui| {
                    egui::Grid::new(("grid-axis", axis))
                        .num_columns(2)
                        .spacing([size::GAP, size::GAP_TIGHT])
                        .show(ui, |ui| {
                            self.number(ui, "Count", &format!("/{axis}/count"), 2.0..=50.0, true);
                            right_label(ui, "Setting");
                            let current = self
                                .settings
                                .pointer(&format!("/{axis}/setting"))
                                .and_then(Value::as_str)
                                .unwrap_or("speed")
                                .to_owned();
                            let shown = SETTINGS
                                .iter()
                                .find(|(k, _)| *k == current)
                                .map_or("Speed", |(_, l)| l);
                            egui::ComboBox::from_id_salt(("axis-setting", axis))
                                .selected_text(shown)
                                .show_ui(ui, |ui| {
                                    for (key, label) in SETTINGS {
                                        if ui.selectable_label(current == key, label).clicked()
                                            && current != key
                                        {
                                            set(
                                                &mut self.settings,
                                                &format!("/{axis}/setting"),
                                                json!(key),
                                            );
                                            self.changed();
                                        }
                                    }
                                });
                            ui.end_row();
                            let u = unit(&current);
                            let whole = current == "passes";
                            self.number(
                                ui,
                                &format!("From{u}"),
                                &format!("/{axis}/from"),
                                0.0..=10_000.0,
                                whole,
                            );
                            self.number(
                                ui,
                                &format!("To{u}"),
                                &format!("/{axis}/to"),
                                0.0..=10_000.0,
                                whole,
                            );
                            self.number(
                                ui,
                                size_label,
                                &format!("/{axis}/size"),
                                1.0..=100.0,
                                false,
                            );
                            self.center(ui, project, usize::from(axis == "rows"), absolute);
                        });
                });
            }
        });
    }

    fn interval_fields(&mut self, ui: &mut egui::Ui, project: &Project, absolute: bool) {
        ui.horizontal_top(|ui| {
            group(ui, "Patches", |ui| {
                egui::Grid::new("interval-a")
                    .num_columns(2)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| {
                        self.number(ui, "Speed (mm/s)", "/speed", 0.1..=10_000.0, false);
                        self.number(ui, "Max power (%)", "/power", 0.0..=100.0, false);
                        self.number(ui, "Patches", "/patches", 2.0..=30.0, true);
                        self.center(ui, project, 0, absolute);
                        self.center(ui, project, 1, absolute);
                    });
            });
            group(ui, "Intervals", |ui| {
                egui::Grid::new("interval-b")
                    .num_columns(3)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| {
                        let t = Tokens::of(ui.ctx());
                        for (label, path) in [
                            ("Smallest interval (mm)", "/smallest"),
                            ("Largest interval (mm)", "/largest"),
                        ] {
                            right_label(ui, label);
                            let mut v = get(&self.settings, path);
                            if ui
                                .add(
                                    egui::DragValue::new(&mut v)
                                        .range(0.01..=2.0)
                                        .max_decimals(3)
                                        .clamp_existing_to_range(false),
                                )
                                .changed()
                            {
                                set(&mut self.settings, path, json!(v));
                                self.changed();
                            }
                            let lpi = if v > 0.0 { 25.4 / v } else { 0.0 };
                            ui.label(
                                egui::RichText::new(format!("{lpi:.0} lines per inch"))
                                    .small()
                                    .color(t.text_muted),
                            );
                            ui.end_row();
                        }
                        self.number(ui, "Patch size (mm)", "/size", 2.0..=100.0, false);
                    });
                ui.horizontal(|ui| {
                    let fill = self
                        .settings
                        .get("fill")
                        .and_then(Value::as_str)
                        .unwrap_or("solid")
                        .to_owned();
                    for (key, label) in [("solid", "Solid"), ("gradient", "Gradient")] {
                        if ui.radio(fill == key, label).clicked() && fill != key {
                            set(&mut self.settings, "/fill", json!(key));
                            self.changed();
                        }
                    }
                });
            });
        });
    }

    fn parts_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (part, label, color) in [
                ("box", "Box settings…", 0),
                ("labels", "Label settings…", 1),
                ("border", "Border settings…", 2),
            ] {
                let chip = crate::colors::layer_color(color);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(size::ROW * 0.6, size::ROW * 0.6),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, crate::theme::ROUNDED, chip);
                if ui
                    .button(label)
                    .on_hover_text(format!("Layer {color:02}: mode, speed, power and passes"))
                    .clicked()
                {
                    self.part = Some(part);
                }
            }
        });
        ui.horizontal(|ui| {
            self.switch(ui, "Draw labels", "draw_labels");
            self.switch(ui, "Draw border", "draw_border");
        });
    }

    fn run_group(
        &mut self,
        ui: &mut egui::Ui,
        laser: &LaserPanel,
        view: &LaserView,
        keymap: &Keymap,
        actions: &mut Vec<Action>,
    ) {
        group(ui, "Run", |ui| {
            if let Some(e) = &self.info.error {
                widgets::band(ui, Band::Error, &format!("Cannot run: {e}"));
            }
            for w in &self.info.warnings {
                widgets::band(ui, Band::Warning, &format!("Warning: {w}"));
            }
            let on = Enabled::of(view, true);
            let fine = self.info.error.is_none() && self.info.settings.is_some();
            let why = if fine {
                laser_panel::why_not(view, false)
            } else {
                "Fix the settings first."
            };
            let width = ((ui.available_width() - 2.0 * size::GAP) / 3.0).max(80.0);
            let big = egui::vec2(width, size::ROW_BIG);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(fine, egui::Button::new("Job preview…").min_size(big))
                    .on_hover_text("Show the pattern's moves in the job preview")
                    .on_disabled_hover_text("Fix the settings first.")
                    .clicked()
                {
                    self.want_preview = true;
                }
                if ui
                    .add_enabled(fine && on.start, egui::Button::new("Frame").min_size(big))
                    .on_hover_text("Trace the pattern's box with the laser off")
                    .on_disabled_hover_text(why)
                    .clicked()
                {
                    actions.push(laser.test_job(self.kind.arg(), &self.json(), true));
                }
                let start = ui
                    .add_enabled(fine && on.start, egui::Button::new("Start").min_size(big))
                    .on_hover_text("Check the pattern against the work area, then run it")
                    .on_disabled_hover_text(why);
                if start.clicked() {
                    start.surrender_focus();
                    actions.push(laser.test_job(self.kind.arg(), &self.json(), false));
                }
            });
            ui.horizontal(|ui| {
                let mut run = Vec::new();
                laser_panel::pause_button(ui, view, keymap, big, &mut run);
                laser_panel::stop_button(ui, view, keymap, big, &mut run);
                actions.extend(run);
                if ui
                    .add_enabled(fine, egui::Button::new("Save G-code…").min_size(big))
                    .on_hover_text("Write the pattern to a G-code file")
                    .on_disabled_hover_text("Fix the settings first.")
                    .clicked()
                {
                    self.prompt = Some(PromptKind::Run {
                        title: "Save test G-code",
                        what: "Path to the .gcode file to write:",
                        command: Invocation::new("test.export", [self.kind.arg(), &self.json()]),
                        at: 1,
                    });
                }
            });
            if let Some((w, h)) = self.info.size {
                let t = Tokens::of(ui.ctx());
                ui.label(
                    egui::RichText::new(format!("Pattern size {w:.1} × {h:.1} mm"))
                        .color(t.text_muted),
                );
            }
        });
    }

    fn part_window(&mut self, ctx: &egui::Context) {
        let Some(part) = self.part else {
            return;
        };
        let mut open = true;
        let title = match part {
            "box" => "Box settings",
            "labels" => "Label settings",
            _ => "Border settings",
        };
        egui::Window::new(title)
            .id(egui::Id::unique(("test-part", self.kind.arg())))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new(("part-grid", part))
                    .num_columns(2)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| {
                        right_label(ui, "Mode");
                        let path = format!("/{part}/mode");
                        let mode = self
                            .settings
                            .pointer(&path)
                            .and_then(Value::as_str)
                            .unwrap_or("line")
                            .to_owned();
                        ui.horizontal(|ui| {
                            for (key, label) in [("line", "Line"), ("fill", "Fill")] {
                                if ui.radio(mode == key, label).clicked() && mode != key {
                                    set(&mut self.settings, &path, json!(key));
                                    self.changed();
                                }
                            }
                        });
                        ui.end_row();
                        self.number(
                            ui,
                            "Speed (mm/s)",
                            &format!("/{part}/speed"),
                            0.1..=10_000.0,
                            false,
                        );
                        self.number(
                            ui,
                            "Max power (%)",
                            &format!("/{part}/max_power"),
                            0.0..=100.0,
                            false,
                        );
                        self.number(ui, "Passes", &format!("/{part}/passes"), 1.0..=100.0, true);
                        self.number(
                            ui,
                            "Interval (mm)",
                            &format!("/{part}/interval"),
                            0.01..=2.0,
                            false,
                        );
                    });
                if part == "box" {
                    ui.label(
                        egui::RichText::new("The varied settings override these in each box.")
                            .small()
                            .color(Tokens::of(ui.ctx()).text_muted),
                    );
                }
            });
        if !open {
            self.part = None;
        }
    }

    fn preset_window(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let Some(name) = &mut self.save_preset else {
            return;
        };
        let mut done = None;
        egui::Window::new("Save preset")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(name);
                });
                let (ok, cancel) = crate::dialogs::buttons(
                    ui,
                    "OK",
                    !name.trim().is_empty(),
                    "Type a name first.",
                );
                if ok {
                    done = Some(true);
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = Some(false);
                }
            });
        if let Some(ok) = done {
            if ok {
                let name = name.trim().to_owned();
                actions.push(Action::Run(Invocation::new(
                    "test.preset_save",
                    [name.clone(), self.kind.arg().to_owned(), self.json()],
                )));
                self.preset = Some(name);
            }
            self.save_preset = None;
        }
    }
}

fn right_label(ui: &mut egui::Ui, text: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(text);
    });
}

/// A group box: heading over a thin-bordered frame.
fn group(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    let t = Tokens::of(ui.ctx());
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(title).color(t.text));
        egui::Frame::new()
            .fill(t.panel)
            .stroke(egui::Stroke::new(weight::THIN, t.border))
            .inner_margin(size::GAP)
            .show(ui, body);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_paths_are_set_and_requests_follow_changes() {
        let mut v = json!({});
        set(&mut v, "/rows/count", json!(3));
        set(&mut v, "/draw_border", json!(false));
        assert_eq!(v, json!({"rows": {"count": 3}, "draw_border": false}));
        assert!((get(&v, "/rows/count") - 3.0).abs() < 1e-12);

        let mut w = TestWindow::new(TestKind::Interval, &Workshop::in_memory());
        let first = w.request(true).unwrap();
        assert_eq!(first.settings, json!({}));
        assert!(w.request(true).is_none(), "nothing changed");
        // An answer for an old revision is ignored; the current one fills in.
        w.changed();
        assert!(
            w.set_info(TestInfo {
                revision: first.revision,
                ..TestInfo::default()
            })
            .is_none()
        );
        let full = json!({"patches": 8});
        let rev = w.request(true).unwrap().revision;
        w.set_info(TestInfo {
            revision: rev,
            settings: Some(full.clone()),
            ..TestInfo::default()
        });
        assert_eq!(w.settings, full);
        w.want_preview = true;
        assert!(w.request(true).unwrap().preview);
    }
}
