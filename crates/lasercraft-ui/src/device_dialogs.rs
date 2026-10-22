//! The Machine panel's Device row and the Device profiles and Device
//! profile dialogs (`docs/spec/devices.md`, "In the app";
//! `docs/spec/style/machine-panels.md`, "9. Device row"). Every change is
//! a `profile.*` or `project.set_device` command.

use lasercraft_commands::Invocation;
use lasercraft_project::{Origin, Project};
use lasercraft_workshop::{DeviceProfile, Workshop};

use crate::shell::PromptKind;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Bounded, Fill};

/// The Device profile dialog's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    WorkArea,
    Options,
    Motion,
}

/// The Device profile dialog: the stored profile (`None` for a new one)
/// and the draft.
#[derive(Debug, Clone)]
struct ProfileForm {
    stored: Option<DeviceProfile>,
    draft: DeviceProfile,
    tab: Tab,
}

impl ProfileForm {
    /// What stops OK, if anything.
    fn problem(&self, workshop: &Workshop) -> Option<String> {
        if let Err(e) = lasercraft_workshop::profile::check_name(&self.draft.name) {
            return Some(e.to_string());
        }
        let taken = workshop.profile(&self.draft.name).is_some_and(|p| {
            self.stored
                .as_ref()
                .is_none_or(|s| !s.name.eq_ignore_ascii_case(&p.name))
        });
        if taken {
            return Some(format!(
                "A profile named {} exists.",
                self.draft.name.trim()
            ));
        }
        self.draft.validate().err().map(|e| e.to_string())
    }

    /// The commands OK runs: add, or rename then set each changed field.
    fn invocations(&self) -> Vec<Invocation> {
        let name = self.draft.name.trim().to_owned();
        let keys = lasercraft_workshop::profile::KEYS;
        let Some(stored) = &self.stored else {
            let mut args = vec![name];
            args.extend(
                keys.iter()
                    .filter_map(|k| Some(format!("{k}={}", self.draft.get(k)?))),
            );
            return vec![Invocation::new("profile.add", args)];
        };
        let mut out = Vec::new();
        if stored.name != name {
            out.push(Invocation::new(
                "profile.rename",
                [stored.name.clone(), name.clone()],
            ));
        }
        for key in keys {
            let new = self.draft.get(key).unwrap_or_default();
            if stored.get(key).unwrap_or_default() != new {
                out.push(Invocation::new(
                    "profile.set",
                    [name.clone(), key.to_owned(), new],
                ));
            }
        }
        out
    }
}

/// UI state of the device windows.
#[derive(Default)]
pub(crate) struct DeviceWindows {
    /// The Device profiles dialog is open.
    list: bool,
    selected: Option<String>,
    form: Option<ProfileForm>,
    remove: Option<String>,
    /// A path the windows want asked for; the shell takes it.
    pub(crate) prompt: Option<PromptKind>,
}

fn origin_word(origin: Origin) -> &'static str {
    match origin {
        Origin::FrontLeft => "Front left",
        Origin::FrontRight => "Front right",
        Origin::RearLeft => "Rear left",
        Origin::RearRight => "Rear right",
    }
}

fn size_line(p: &DeviceProfile) -> String {
    format!("{} × {} mm", p.width, p.height)
}

impl DeviceWindows {
    /// Opens the Device profiles dialog.
    pub(crate) fn open_list(&mut self) {
        self.list = true;
    }

    /// Opens the Device profile dialog for the project's profile (or the
    /// default one); with none, for a new profile.
    pub(crate) fn open_current(&mut self, project: &Project, workshop: &Workshop) {
        let current = project
            .device()
            .or_else(|| workshop.default_profile())
            .and_then(|n| workshop.profile(n));
        self.form = Some(match current {
            Some(p) => ProfileForm {
                stored: Some(p.clone()),
                draft: p.clone(),
                tab: Tab::WorkArea,
            },
            None => Self::new_form(workshop),
        });
    }

    fn new_form(workshop: &Workshop) -> ProfileForm {
        let mut n = workshop.profiles().len() + 1;
        let mut name = format!("Laser {n}");
        while workshop.profile(&name).is_some() {
            n += 1;
            name = format!("Laser {n}");
        }
        ProfileForm {
            stored: None,
            draft: DeviceProfile::new(&name).unwrap_or_else(|_| unreachable!("valid name")),
            tab: Tab::WorkArea,
        }
    }

    /// The Device row. Returns the commands and, when a profile was
    /// chosen, that profile (so the connection row can take its port).
    pub(crate) fn device_row(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        connected: bool,
    ) -> (Vec<Invocation>, Option<DeviceProfile>) {
        let t = Tokens::of(ui.ctx());
        let mut out = Vec::new();
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            ui.label("Device");
            let current = project.device().unwrap_or("(none)");
            ui.add_enabled_ui(!connected, |ui| {
                egui::ComboBox::from_id_salt("device-profile")
                    .selected_text(current)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(project.device().is_none(), "(none)")
                            .on_hover_text("Keep the work area; use the default limits")
                            .clicked()
                            && project.device().is_some()
                        {
                            out.push(Invocation::new("project.set_device", ["none"]));
                        }
                        for p in workshop.profiles() {
                            let on = project
                                .device()
                                .is_some_and(|d| d.eq_ignore_ascii_case(&p.name));
                            let mut text = egui::text::LayoutJob::default();
                            let style = ui.style().clone();
                            egui::RichText::new(&p.name).color(t.text).append_to(
                                &mut text,
                                &style,
                                egui::FontSelection::Default,
                                egui::Align::Center,
                            );
                            let mut extra = format!("  {}", size_line(p));
                            if workshop.default_profile() == Some(p.name.as_str()) {
                                extra.push_str("  default");
                            }
                            egui::RichText::new(extra).color(t.text_muted).append_to(
                                &mut text,
                                &style,
                                egui::FontSelection::Default,
                                egui::Align::Center,
                            );
                            if ui.selectable_label(on, text).clicked() && !on {
                                out.push(Invocation::new("project.set_device", [p.name.clone()]));
                                chosen = Some(p.clone());
                            }
                        }
                    })
                    .response
                    .on_hover_text("Device profile in use: work area, origin, port and limits")
                    .on_disabled_hover_text("Disconnect to change the device.");
            });
            if ui
                .button("Device profiles…")
                .on_hover_text("Add, remove and choose saved machines")
                .clicked()
            {
                self.list = true;
            }
        });
        (out, chosen)
    }

    /// Draws the open windows; returns their commands.
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        workshop: &Workshop,
        connected: bool,
    ) -> Vec<Invocation> {
        let mut out = Vec::new();
        if self
            .selected
            .as_ref()
            .is_some_and(|n| workshop.profile(n).is_none())
        {
            self.selected = None;
        }
        self.list_window(ctx, workshop, &mut out);
        self.form_window(ctx, workshop, connected, &mut out);
        self.remove_window(ctx, &mut out);
        out
    }

    fn list_window(&mut self, ctx: &egui::Context, workshop: &Workshop, out: &mut Vec<Invocation>) {
        if !self.list {
            return;
        }
        let t = Tokens::of(ctx);
        let mut close = false;
        egui::Window::new("Device profiles")
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(240.0);
                        egui::Frame::new()
                            .fill(t.field)
                            .stroke(egui::Stroke::new(weight::THIN, t.border))
                            .inner_margin(size::GAP_TIGHT)
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(ui.available_width(), 180.0));
                                if workshop.profiles().is_empty() {
                                    ui.label(
                                        egui::RichText::new("No device profiles yet.")
                                            .color(t.text_muted),
                                    );
                                }
                                for p in workshop.profiles() {
                                    let on = self.selected.as_deref() == Some(p.name.as_str());
                                    let mut name = p.name.clone();
                                    if workshop.default_profile() == Some(p.name.as_str()) {
                                        name.push_str("  (default)");
                                    }
                                    let row = ui.selectable_label(on, name);
                                    ui.label(
                                        egui::RichText::new(size_line(p))
                                            .small()
                                            .color(t.text_muted),
                                    );
                                    if row.clicked() {
                                        self.selected = Some(p.name.clone());
                                    }
                                    if row.double_clicked() {
                                        self.form = Some(ProfileForm {
                                            stored: Some(p.clone()),
                                            draft: p.clone(),
                                            tab: Tab::WorkArea,
                                        });
                                    }
                                }
                            });
                        if let Some(p) = self.selected.as_ref().and_then(|n| workshop.profile(n)) {
                            ui.label(egui::RichText::new(p.summary()).color(t.text_muted));
                        }
                    });
                    ui.vertical(|ui| self.list_buttons(ui, workshop, out));
                });
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("Close").min_size(egui::vec2(80.0, size::ROW)))
                        .clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        close = true;
                    }
                });
            });
        if close {
            self.list = false;
        }
    }

    fn list_buttons(&mut self, ui: &mut egui::Ui, workshop: &Workshop, out: &mut Vec<Invocation>) {
        let min = egui::vec2(130.0, size::ROW);
        if ui
            .add(egui::Button::new("New profile…").min_size(min))
            .clicked()
        {
            self.form = Some(Self::new_form(workshop));
        }
        if ui.add(egui::Button::new("Import…").min_size(min)).clicked() {
            self.prompt = Some(PromptKind::Run {
                title: "Import device profile",
                what: "Path to a .lcdevice file:",
                command: Invocation::bare("profile.import"),
                at: 0,
            });
        }
        ui.add_space(size::GAP);
        let selected = self
            .selected
            .as_ref()
            .and_then(|n| workshop.profile(n))
            .cloned();
        let on = selected.is_some();
        let pick = "Select a profile first.";
        let is_default = selected.as_ref().map(|p| p.name.as_str()) == workshop.default_profile();
        let button = |ui: &mut egui::Ui, label: &str, enabled: bool, why: &str| {
            ui.add_enabled(enabled, egui::Button::new(label).min_size(min))
                .on_disabled_hover_text(why)
                .clicked()
        };
        let name = selected
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if button(
            ui,
            "Set as default",
            on && !is_default,
            if on { "It is the default." } else { pick },
        ) {
            out.push(Invocation::new("profile.default", [name.clone()]));
        }
        if button(ui, "Edit…", on, pick)
            && let Some(p) = &selected
        {
            self.form = Some(ProfileForm {
                stored: Some(p.clone()),
                draft: p.clone(),
                tab: Tab::WorkArea,
            });
        }
        if button(ui, "Duplicate", on, pick) {
            let mut n = 2;
            while workshop.profile(&format!("{name} {n}")).is_some() {
                n += 1;
            }
            out.push(Invocation::new(
                "profile.duplicate",
                [name.clone(), format!("{name} {n}")],
            ));
        }
        if button(ui, "Remove", on, pick) {
            self.remove = Some(name.clone());
        }
        if button(ui, "Export…", on, pick) {
            self.prompt = Some(PromptKind::Run {
                title: "Export device profile",
                what: "Path to the .lcdevice file to write:",
                command: Invocation::new("profile.export", [name]),
                at: 1,
            });
        }
    }

    fn remove_window(&mut self, ctx: &egui::Context, out: &mut Vec<Invocation>) {
        let Some(name) = self.remove.clone() else {
            return;
        };
        let mut done = false;
        egui::Window::new("Remove device profile?")
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.set_max_width(340.0);
                ui.label(format!(
                    "The profile {name} is removed. Projects that name it keep their work area."
                ));
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let min = egui::vec2(96.0, size::ROW);
                    if ui.add(egui::Button::new("Cancel").min_size(min)).clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        done = true;
                    }
                    if widgets::filled(ui, "Remove profile", Fill::Danger, min).clicked() {
                        out.push(Invocation::new("profile.delete", [name.clone()]));
                        done = true;
                    }
                });
            });
        if done {
            self.remove = None;
        }
    }

    fn form_window(
        &mut self,
        ctx: &egui::Context,
        workshop: &Workshop,
        connected: bool,
        out: &mut Vec<Invocation>,
    ) {
        let Some(form) = &mut self.form else {
            return;
        };
        let title = match &form.stored {
            Some(p) => format!("Device profile: {}", p.name),
            None => "New device profile".to_owned(),
        };
        let problem = form.problem(workshop);
        let mut done = None;
        egui::Window::new(title)
            .id(egui::Id::unique("device-profile-form"))
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut form.draft.name);
                });
                ui.add_space(size::GAP_TIGHT);
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (Tab::WorkArea, "Work area"),
                        (Tab::Options, "Options"),
                        (Tab::Motion, "Motion"),
                    ] {
                        ui.selectable_value(&mut form.tab, tab, label);
                    }
                });
                ui.separator();
                form_tab(ui, &mut form.draft, form.tab, connected);
                if let Some(problem) = &problem {
                    widgets::band(ui, widgets::Band::Error, problem);
                }
                let (ok, cancel) = crate::dialogs::buttons(
                    ui,
                    "OK",
                    problem.is_none(),
                    "Fix the field shown above.",
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
                out.extend(form.invocations());
            }
            self.form = None;
        }
    }
}

fn form_tab(ui: &mut egui::Ui, p: &mut DeviceProfile, tab: Tab, connected: bool) {
    let label = |ui: &mut egui::Ui, text: &str| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(text);
        });
    };
    let number = |ui: &mut egui::Ui, v: &mut f64, max: f64| {
        ui.add(egui::DragValue::new(v).range(0.0..=max).max_decimals(2));
    };
    egui::Grid::new(("device-form", tab as u8))
        .num_columns(2)
        .spacing([size::GAP, size::GAP_TIGHT])
        .min_col_width(80.0)
        .show(ui, |ui| match tab {
            Tab::WorkArea => {
                label(ui, "Width (mm)");
                number(ui, &mut p.width, 10_000.0);
                ui.end_row();
                label(ui, "Height (mm)");
                number(ui, &mut p.height, 10_000.0);
                ui.end_row();
                label(ui, "Origin corner");
                ui.vertical(|ui| {
                    for o in Origin::ALL {
                        ui.radio_value(&mut p.origin, o, origin_word(o));
                    }
                });
                ui.end_row();
            }
            Tab::Options => {
                let why = "Disconnect to change the connection.";
                label(ui, "Port");
                ui.add_enabled(!connected, egui::TextEdit::singleline(&mut p.port))
                    .on_hover_text("Serial port; leave empty to choose in the Machine panel")
                    .on_disabled_hover_text(why);
                ui.end_row();
                label(ui, "Baud rate");
                ui.add_enabled(
                    !connected,
                    egui::DragValue::new(&mut p.baud).range(1..=10_000_000),
                )
                .on_disabled_hover_text(why);
                ui.end_row();
                label(ui, "Full-power S value ($30)");
                number(ui, &mut p.s_max, 100_000.0);
                ui.end_row();
                label(ui, "Max power (%)");
                number(ui, &mut p.max_power, 100.0);
                ui.end_row();
                label(ui, "Air assist code");
                ui.horizontal(|ui| {
                    for code in ["m7", "m8"] {
                        ui.radio_value(&mut p.air, code.to_owned(), code.to_uppercase());
                    }
                });
                ui.end_row();
            }
            Tab::Motion => {
                label(ui, "Max speed (mm/s)");
                number(ui, &mut p.max_speed, 10_000.0);
                ui.end_row();
                label(ui, "Acceleration (mm/s²)");
                number(ui, &mut p.acceleration, 100_000.0);
                ui.end_row();
                ui.label("");
                ui.label(
                    egui::RichText::new("Used only to estimate job times.")
                        .color(Tokens::of(ui.ctx()).text_muted),
                );
                ui.end_row();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_form_adds_or_renames_and_sets_changed_fields() {
        let mut ws = Workshop::in_memory();
        let form = DeviceWindows::new_form(&ws);
        assert_eq!(form.draft.name, "Laser 1");
        assert!(form.problem(&ws).is_none());
        let add = form.invocations();
        assert_eq!(add.len(), 1);
        assert!(
            add[0]
                .to_string()
                .starts_with("profile.add \"Laser 1\" width="),
            "{}",
            add[0]
        );

        ws.add_profile(form.draft.clone()).unwrap();
        let mut edit = ProfileForm {
            stored: Some(form.draft.clone()),
            draft: form.draft.clone(),
            tab: Tab::Options,
        };
        assert!(edit.invocations().is_empty(), "nothing changed");
        edit.draft.name = "Shop".into();
        edit.draft.max_power = 60.0;
        let lines: Vec<String> = edit.invocations().iter().map(ToString::to_string).collect();
        assert_eq!(
            lines,
            [
                "profile.rename \"Laser 1\" Shop",
                "profile.set Shop max_power 60"
            ]
        );
        edit.draft.max_power = 300.0;
        assert!(edit.problem(&ws).is_some());
        // A new profile may not take a used name.
        let mut new = DeviceWindows::new_form(&ws);
        assert_eq!(new.draft.name, "Laser 2");
        new.draft.name = "laser 1".into();
        assert!(new.problem(&ws).is_some());
    }
}
