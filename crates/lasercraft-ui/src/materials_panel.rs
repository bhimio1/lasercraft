//! The Materials panel (`docs/spec/materials.md`; `docs/spec/style/
//! later-windows.md`, "Materials panel"): the open libraries, the entry
//! tree, the button column with its entry form, and the panel's small
//! windows. Every change is a `library.*` or `material.*` command.

use lasercraft_commands::Invocation;
use lasercraft_project::{Color, Layer, Mode, Project};
use lasercraft_workshop::{Material, OpenLibrary, Workshop};

use crate::shell::PromptKind;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Band, Bounded, Fill};

/// The entry form, in place of the button column.
#[derive(Debug, Clone, PartialEq)]
struct EntryForm {
    /// The entry being described; `None` stores the layer `color` as new.
    id: Option<u32>,
    color: Option<Color>,
    material: String,
    thickness: f64,
    no_thickness: bool,
    group: String,
    description: String,
}

impl EntryForm {
    fn ready(&self) -> bool {
        !self.material.trim().is_empty()
            && !self.description.trim().is_empty()
            && if self.no_thickness {
                !self.group.trim().is_empty()
            } else {
                self.thickness > 0.0
            }
    }

    /// The command the form's OK runs.
    fn invocation(&self) -> Invocation {
        let thickness = if self.no_thickness {
            "-".to_owned()
        } else {
            self.thickness.to_string()
        };
        let mut args = vec![
            self.material.trim().to_owned(),
            thickness,
            self.description.trim().to_owned(),
        ];
        let name = match (self.id, self.color) {
            (Some(id), _) => {
                args.insert(0, id.to_string());
                "material.describe"
            }
            (None, color) => {
                args.push(color.unwrap_or_default().to_string());
                "material.add"
            }
        };
        if self.no_thickness {
            args.push(format!("group={}", self.group.trim()));
        }
        Invocation::new(name, args)
    }
}

/// An action on the selected entry (button column and entry menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Act {
    Apply,
    Follow,
    SaveAs,
    EditSettings,
    Update,
    Describe,
    Duplicate,
    Delete,
}

impl Act {
    fn text(self) -> (&'static str, &'static str) {
        match self {
            Self::Apply => (
                "Apply to layer",
                "Copy this material's settings into the target layer\nThe layer does not change when the entry changes later.",
            ),
            Self::Follow => (
                "Follow this entry",
                "Keep the target layer tied to this material\nThe layer's settings become read-only and follow the entry.",
            ),
            Self::SaveAs => (
                "Save layer as material…",
                "Store the target layer's settings as a new material entry",
            ),
            Self::EditSettings => ("Edit settings…", "Change this entry's settings"),
            Self::Update => (
                "Update from layer",
                "Replace this entry's settings with the target layer's",
            ),
            Self::Describe => (
                "Edit description…",
                "Change this entry's material, thickness and description",
            ),
            Self::Duplicate => ("Duplicate", "Copy this entry"),
            Self::Delete => ("Delete", "Remove this entry from the library"),
        }
    }
}

/// A question waiting for an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Confirm {
    Replace { id: u32, color: Color },
    Delete(u32),
    Close,
}

/// What the small name window is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameFor {
    New,
    Rename,
}

/// UI state of the Materials panel.
#[derive(Default)]
pub(crate) struct MaterialsPanel {
    selected: Option<u32>,
    form: Option<EntryForm>,
    confirm: Option<Confirm>,
    name: Option<(NameFor, String)>,
    /// Edit settings…: the entry, its stored settings and the draft.
    settings: Option<(u32, Layer, Layer)>,
    /// A path the panel wants asked for; the shell takes it.
    pub(crate) prompt: Option<PromptKind>,
}

fn run(name: &str, args: impl IntoIterator<Item = impl Into<String>>) -> Invocation {
    Invocation::new(name, args.into_iter().map(Into::into))
}

fn mode_word(mode: Mode) -> &'static str {
    match mode {
        Mode::Line => "Line",
        Mode::Fill => "Fill",
    }
}

/// Whether a layer of `project` follows `entry` of `library`.
fn followed(project: &Project, library: &str, entry: &Material) -> bool {
    project.layers().iter().any(|l| {
        l.follows
            .as_ref()
            .is_some_and(|f| entry.is_followed_by(library, f))
    })
}

fn ask(title: &'static str, what: &'static str, name: &str) -> PromptKind {
    PromptKind::Run {
        title,
        what,
        command: Invocation::bare(name),
        at: 0,
    }
}

const READ_ONLY: &str =
    "The starter library cannot be changed. Use Save library as… to make your own copy.";

impl MaterialsPanel {
    /// Draws the panel. `target` is the Layers panel's edited layer.
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        target: Option<Color>,
    ) -> Vec<Invocation> {
        let mut out = Vec::new();
        let open = workshop.active();
        if self
            .selected
            .is_some_and(|id| open.library.entry(id).is_err())
        {
            self.selected = None;
        }
        self.library_row(ui, workshop, &mut out);
        ui.add_space(size::GAP_TIGHT);
        if ui.available_width() < 420.0 {
            // Narrow dock: the buttons wrap above the tree.
            self.right(ui, project, open, target, &mut out);
            ui.add_space(size::GAP_TIGHT);
            self.left(ui, project, open, target, &mut out);
        } else {
            let tree = (ui.available_width() * 0.55).max(200.0);
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(tree);
                    self.left(ui, project, open, target, &mut out);
                });
                ui.vertical(|ui| self.right(ui, project, open, target, &mut out));
            });
        }
        self.windows(ui.ctx(), open, &mut out);
        out
    }

    fn library_row(&mut self, ui: &mut egui::Ui, workshop: &Workshop, out: &mut Vec<Invocation>) {
        let open = workshop.active();
        let label = |l: &OpenLibrary| {
            if l.modified {
                format!("{} •", l.library.name)
            } else {
                l.library.name.clone()
            }
        };
        ui.horizontal_wrapped(|ui| {
            ui.label("Library");
            let tip = open.library.device.as_ref().map_or_else(
                || "The material library in use".to_owned(),
                |d| format!("Used with device profile {d}"),
            );
            egui::ComboBox::from_id_salt("materials-library")
                .selected_text(label(open))
                .width(160.0)
                .show_ui(ui, |ui| {
                    for l in workshop.libraries() {
                        let active = l.library.name == open.library.name;
                        if ui.selectable_label(active, label(l)).clicked() && !active {
                            out.push(run("library.use", [l.library.name.clone()]));
                        }
                    }
                })
                .response
                .on_hover_text(tip);
            if !open.library.note.is_empty() {
                widgets::mark(ui, Band::Hint).on_hover_text(&open.library.note);
            }
            self.library_menu(ui, open, out);
        });
    }

    /// The entry tree.
    fn left(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        open: &OpenLibrary,
        target: Option<Color>,
        out: &mut Vec<Invocation>,
    ) {
        let t = Tokens::of(ui.ctx());
        egui::Frame::new()
            .fill(t.field)
            .stroke(egui::Stroke::new(weight::THIN, t.border))
            .inner_margin(size::GAP_TIGHT)
            .show(ui, |ui| {
                let room = (ui.available_height() - 2.0 * size::ROW).max(120.0);
                egui::ScrollArea::vertical()
                    .id_salt("materials-tree")
                    .max_height(room.min(260.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.tree(ui, project, open, target, out);
                    });
            });
    }

    /// The Library ▸ menu (`docs/spec/style/menus.md`, "Library menu").
    fn library_menu(&mut self, ui: &mut egui::Ui, open: &OpenLibrary, out: &mut Vec<Invocation>) {
        widgets::menu_button(ui, "Library", |ui| {
            if ui.button("Open library…").clicked() {
                self.prompt = Some(ask(
                    "Open library",
                    "Path to a .lcmat file:",
                    "library.open",
                ));
            }
            let can_save = !open.built_in && open.path.is_some();
            if ui
                .add_enabled(can_save && open.modified, egui::Button::new("Save library"))
                .on_disabled_hover_text(if open.built_in || open.path.is_none() {
                    "Use Save library as… first."
                } else {
                    "No unsaved changes."
                })
                .clicked()
            {
                out.push(Invocation::bare("library.save"));
            }
            if ui.button("Save library as…").clicked() {
                self.prompt = Some(ask(
                    "Save library as",
                    "Path to the .lcmat file to write:",
                    "library.save",
                ));
            }
            if ui.button("New library").clicked() {
                self.name = Some((NameFor::New, String::new()));
            }
            if ui
                .add_enabled(!open.built_in, egui::Button::new("Merge library…"))
                .on_disabled_hover_text(READ_ONLY)
                .clicked()
            {
                self.prompt = Some(ask(
                    "Merge library",
                    "Path to the .lcmat file to add:",
                    "library.merge",
                ));
            }
            ui.separator();
            let editable = !open.built_in;
            if ui
                .add_enabled(editable, egui::Button::new("Rename library…"))
                .on_disabled_hover_text("The starter library cannot be renamed.")
                .clicked()
            {
                self.name = Some((NameFor::Rename, open.library.name.clone()));
            }
            if ui
                .add_enabled(editable, egui::Button::new("Close library"))
                .on_disabled_hover_text("The starter library stays open.")
                .clicked()
            {
                if open.modified {
                    self.confirm = Some(Confirm::Close);
                } else {
                    out.push(run("library.close", [open.library.name.clone()]));
                }
            }
        });
    }

    fn tree(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        open: &OpenLibrary,
        target: Option<Color>,
        out: &mut Vec<Invocation>,
    ) {
        let t = Tokens::of(ui.ctx());
        let entries = open.library.entries();
        if entries.is_empty() {
            ui.label(
                egui::RichText::new(
                    "No materials yet. Select a layer, then use Save layer as material….",
                )
                .color(t.text_muted),
            );
            return;
        }
        let mut i = 0;
        while i < entries.len() {
            let material = entries[i].material.clone();
            let end = entries[i..]
                .iter()
                .position(|e| e.material != material)
                .map_or(entries.len(), |n| i + n);
            egui::CollapsingHeader::new(&material)
                .id_salt(("material", &material))
                .default_open(true)
                .show(ui, |ui| {
                    let mut k = i;
                    while k < end {
                        let level = entries[k].level();
                        let stop = entries[k..end]
                            .iter()
                            .position(|e| e.level() != level)
                            .map_or(end, |n| k + n);
                        egui::CollapsingHeader::new(&level)
                            .id_salt(("level", &material, &level))
                            .default_open(true)
                            .show(ui, |ui| {
                                for e in &entries[k..stop] {
                                    self.entry_row(ui, project, open, e, target, out);
                                }
                            });
                        k = stop;
                    }
                });
            i = end;
        }
    }

    fn entry_row(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        open: &OpenLibrary,
        e: &Material,
        target: Option<Color>,
        out: &mut Vec<Invocation>,
    ) {
        let t = Tokens::of(ui.ctx());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(mode_word(e.settings.mode))
                    .small()
                    .color(t.text_muted),
            );
            let row = ui
                .selectable_label(self.selected == Some(e.id), &e.description)
                .on_hover_text(format!(
                    "{} mm/s, {}%, {} pass(es)\nDouble-click to apply to the target layer.",
                    e.settings.speed, e.settings.max_power, e.settings.passes
                ));
            if row.clicked() || row.secondary_clicked() {
                self.selected = Some(e.id);
            }
            row.context_menu(|ui| {
                let acts = self.acts(open, target);
                for (i, a) in acts.iter().enumerate() {
                    if i == 2 || i == 6 {
                        ui.separator();
                    }
                    self.act_button(ui, *a, 0.0, (project, open, target), out);
                }
            });
            if row.double_clicked()
                && let Some(c) = target
            {
                out.push(run("material.apply", [e.id.to_string(), c.to_string()]));
            }
            if followed(project, &open.library.name, e) {
                ui.label(egui::RichText::new("followed").small().color(t.text_muted))
                    .on_hover_text("A layer of this project follows this entry");
            }
        });
    }

    /// The button column (or the entry form) and the target line.
    fn right(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        open: &OpenLibrary,
        target: Option<Color>,
        out: &mut Vec<Invocation>,
    ) {
        let compact = ui.available_width() < 420.0;
        if self.form.is_some() {
            self.form(ui, out);
        } else if compact {
            ui.horizontal_wrapped(|ui| self.buttons(ui, project, open, target, true, out));
        } else {
            self.buttons(ui, project, open, target, false, out);
        }
        ui.add_space(size::GAP);
        let t = Tokens::of(ui.ctx());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Target layer").color(t.text_muted));
            match target.and_then(|c| project.layer(c).ok()) {
                Some(layer) => {
                    ui.label(format!("{:02} {}", layer.color, layer.name));
                }
                None => {
                    ui.label(egui::RichText::new("No layer selected").color(t.text_muted));
                }
            }
        });
    }

    /// The entry actions with what each is missing, in button order.
    fn acts(&self, open: &OpenLibrary, target: Option<Color>) -> [(Act, Option<&'static str>); 8] {
        let entry = self.selected.and_then(|id| open.library.entry(id).ok());
        let no_target = "Click a layer in the Layers panel first.";
        let no_entry = "Select an entry first.";
        let use_missing = match (entry, target) {
            (None, _) => Some(no_entry),
            (_, None) => Some(no_target),
            _ => None,
        };
        let edit_missing = |need_entry: bool, need_target: bool| {
            if open.built_in {
                Some(READ_ONLY)
            } else if need_entry && entry.is_none() {
                Some(no_entry)
            } else if need_target && target.is_none() {
                Some(no_target)
            } else {
                None
            }
        };
        [
            (Act::Apply, use_missing),
            (Act::Follow, use_missing),
            (Act::SaveAs, edit_missing(false, true)),
            (Act::EditSettings, edit_missing(true, false)),
            (Act::Update, edit_missing(true, true)),
            (Act::Describe, edit_missing(true, false)),
            (Act::Duplicate, edit_missing(true, false)),
            (Act::Delete, edit_missing(true, false)),
        ]
    }

    fn act_button(
        &mut self,
        ui: &mut egui::Ui,
        (act, missing): (Act, Option<&str>),
        width: f32,
        cx: (&Project, &OpenLibrary, Option<Color>),
        out: &mut Vec<Invocation>,
    ) {
        let (label, tip) = act.text();
        let clicked = ui
            .add_enabled(
                missing.is_none(),
                egui::Button::new(label).min_size(egui::vec2(width, size::ROW)),
            )
            .on_hover_text(tip)
            .on_disabled_hover_text(missing.unwrap_or_default())
            .clicked();
        if clicked {
            self.act(act, cx, out);
        }
    }

    fn act(
        &mut self,
        act: Act,
        (project, open, target): (&Project, &OpenLibrary, Option<Color>),
        out: &mut Vec<Invocation>,
    ) {
        let entry = self.selected.and_then(|id| open.library.entry(id).ok());
        let (id, c) = (
            entry.map(|e| e.id).unwrap_or_default(),
            target.unwrap_or_default(),
        );
        match act {
            Act::Apply => out.push(run("material.apply", [id.to_string(), c.to_string()])),
            Act::Follow => out.push(run("material.follow", [id.to_string(), c.to_string()])),
            Act::SaveAs => {
                let name = project.layer(c).map(|l| l.name.clone()).unwrap_or_default();
                self.form = Some(EntryForm {
                    id: None,
                    color: Some(c),
                    material: String::new(),
                    thickness: 3.0,
                    no_thickness: false,
                    group: String::new(),
                    description: name,
                });
            }
            Act::EditSettings => {
                if let Some(e) = entry {
                    self.settings = Some((id, e.settings.clone(), e.settings.clone()));
                }
            }
            Act::Update => self.confirm = Some(Confirm::Replace { id, color: c }),
            Act::Describe => {
                if let Some(e) = entry {
                    self.form = Some(EntryForm {
                        id: Some(id),
                        color: None,
                        material: e.material.clone(),
                        thickness: e.thickness.unwrap_or(3.0),
                        no_thickness: e.thickness.is_none(),
                        group: e.group.clone().unwrap_or_default(),
                        description: e.description.clone(),
                    });
                }
            }
            Act::Duplicate => out.push(run("material.duplicate", [id.to_string()])),
            Act::Delete => self.confirm = Some(Confirm::Delete(id)),
        }
    }

    /// The button column; `compact` wraps the use buttons and puts the
    /// entry changes in an "Entry ▸" menu.
    fn buttons(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        open: &OpenLibrary,
        target: Option<Color>,
        compact: bool,
        out: &mut Vec<Invocation>,
    ) {
        let acts = self.acts(open, target);
        let cx = (project, open, target);
        if compact {
            for a in &acts[..3] {
                self.act_button(ui, *a, 0.0, cx, out);
            }
            widgets::menu_button(ui, "Entry", |ui| {
                for a in &acts[3..] {
                    self.act_button(ui, *a, 0.0, cx, out);
                }
            });
            return;
        }
        let width = ui.available_width().clamp(140.0, 220.0);
        for (i, a) in acts.iter().enumerate() {
            if i == 2 || i == 6 {
                ui.add_space(size::ROW);
            }
            self.act_button(ui, *a, width, cx, out);
        }
    }

    fn form(&mut self, ui: &mut egui::Ui, out: &mut Vec<Invocation>) {
        let Some(form) = &mut self.form else {
            return;
        };
        let label = |ui: &mut egui::Ui, text: &str| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(text);
            });
        };
        egui::Grid::new("material-form")
            .num_columns(2)
            .spacing([size::GAP, size::GAP_TIGHT])
            .show(ui, |ui| {
                label(ui, "Material");
                ui.text_edit_singleline(&mut form.material);
                ui.end_row();
                label(ui, "Thickness (mm)");
                ui.add_enabled(
                    !form.no_thickness,
                    egui::DragValue::new(&mut form.thickness)
                        .range(0.01..=1000.0)
                        .max_decimals(2),
                );
                ui.end_row();
                ui.label("");
                widgets::switch(ui, &mut form.no_thickness, "Thickness does not apply");
                ui.end_row();
                label(ui, "Group name");
                ui.add_enabled(
                    form.no_thickness,
                    egui::TextEdit::singleline(&mut form.group),
                );
                ui.end_row();
                label(ui, "Description");
                ui.text_edit_singleline(&mut form.description);
                ui.end_row();
            });
        let ready = form.ready();
        let mut close = false;
        ui.horizontal(|ui| {
            let ok = ui
                .add_enabled_ui(ready, |ui| {
                    widgets::filled(ui, "OK", Fill::Primary, egui::vec2(72.0, size::ROW))
                })
                .inner
                .on_disabled_hover_text(
                    "Fill in material, description and a thickness or group name.",
                );
            if ok.clicked() {
                out.push(form.invocation());
                close = true;
            }
            if ui.button("Cancel").clicked() {
                close = true;
            }
        });
        if close {
            self.form = None;
        }
    }

    /// The name, confirmation and Edit settings windows.
    fn windows(&mut self, ctx: &egui::Context, open: &OpenLibrary, out: &mut Vec<Invocation>) {
        self.name_window(ctx, out);
        self.confirm_window(ctx, open, out);
        self.settings_window(ctx, out);
    }

    fn name_window(&mut self, ctx: &egui::Context, out: &mut Vec<Invocation>) {
        let Some((what, text)) = &mut self.name else {
            return;
        };
        let title = match what {
            NameFor::New => "New library",
            NameFor::Rename => "Rename library",
        };
        let mut done = None;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(text);
                });
                let blank = text.trim().is_empty();
                let (ok, cancel) = crate::dialogs::buttons(ui, "OK", !blank, "Type a name first.");
                if ok {
                    done = Some(true);
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = Some(false);
                }
            });
        if let Some(ok) = done {
            if ok {
                let name = text.trim().to_owned();
                out.push(match what {
                    NameFor::New => run("library.new", [name]),
                    NameFor::Rename => run("library.rename", [name]),
                });
            }
            self.name = None;
        }
    }

    fn confirm_window(
        &mut self,
        ctx: &egui::Context,
        open: &OpenLibrary,
        out: &mut Vec<Invocation>,
    ) {
        let Some(confirm) = self.confirm else {
            return;
        };
        let describe = |id: u32| {
            open.library
                .entry(id)
                .map(|e| e.description.clone())
                .unwrap_or_default()
        };
        let (title, body) = match confirm {
            Confirm::Replace { id, color } => (
                "Replace material settings?",
                format!(
                    "The settings of \"{}\" are replaced by those of layer {color:02}. Layers that follow it change too.",
                    describe(id)
                ),
            ),
            Confirm::Delete(id) => (
                "Delete material entry?",
                format!(
                    "The entry \"{}\" is removed from {}. Layers that follow it keep their current settings and stop following it.",
                    describe(id),
                    open.library.name
                ),
            ),
            Confirm::Close => (
                "Close library?",
                format!("{} has changes that are not saved.", open.library.name),
            ),
        };
        let mut answer = None;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.set_max_width(360.0);
                ui.label(body);
                ui.add_space(size::GAP);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Cancel").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        answer = Some(None);
                    }
                    let min = egui::vec2(96.0, size::ROW);
                    match confirm {
                        Confirm::Replace { id, color } => {
                            if ui
                                .add(egui::Button::new("Replace settings").min_size(min))
                                .clicked()
                            {
                                answer = Some(Some(run(
                                    "material.update",
                                    [id.to_string(), color.to_string()],
                                )));
                            }
                        }
                        Confirm::Delete(id) => {
                            if widgets::filled(ui, "Delete entry", Fill::Danger, min).clicked() {
                                answer = Some(Some(run("material.delete", [id.to_string()])));
                            }
                        }
                        Confirm::Close => {
                            if widgets::filled(ui, "Discard", Fill::Danger, min).clicked() {
                                answer = Some(Some(run(
                                    "library.close",
                                    [open.library.name.clone(), "force".to_owned()],
                                )));
                            }
                            if ui
                                .add(egui::Button::new("Save library").min_size(min))
                                .clicked()
                            {
                                answer = Some(if open.path.is_some() {
                                    Some(Invocation::bare("library.save"))
                                } else {
                                    self.prompt = Some(ask(
                                        "Save library as",
                                        "Path to the .lcmat file to write:",
                                        "library.save",
                                    ));
                                    None
                                });
                            }
                        }
                    }
                });
            });
        if let Some(answer) = answer {
            out.extend(answer);
            self.confirm = None;
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context, out: &mut Vec<Invocation>) {
        let Some((id, stored, draft)) = &mut self.settings else {
            return;
        };
        let mut done = None;
        egui::Window::new("Material settings")
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                egui::Grid::new("material-settings")
                    .num_columns(2)
                    .spacing([size::GAP, size::GAP_TIGHT])
                    .show(ui, |ui| {
                        let label = |ui: &mut egui::Ui, text: &str| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(text);
                                },
                            );
                        };
                        label(ui, "Mode");
                        egui::ComboBox::from_id_salt("material-mode")
                            .selected_text(mode_word(draft.mode))
                            .show_ui(ui, |ui| {
                                for m in [Mode::Line, Mode::Fill] {
                                    ui.selectable_value(&mut draft.mode, m, mode_word(m));
                                }
                            });
                        ui.end_row();
                        for (text, value, range) in [
                            ("Speed (mm/s)", &mut draft.speed, 0.1..=10_000.0),
                            ("Max power (%)", &mut draft.max_power, 0.0..=100.0),
                            ("Interval (mm)", &mut draft.interval, 0.001..=10.0),
                        ] {
                            label(ui, text);
                            ui.add(egui::DragValue::new(value).range(range).max_decimals(3));
                            ui.end_row();
                        }
                        label(ui, "Passes");
                        ui.add(egui::DragValue::new(&mut draft.passes).range(1..=1000));
                        ui.end_row();
                        ui.label("");
                        widgets::switch(ui, &mut draft.air, "Air assist");
                        ui.end_row();
                        ui.label("");
                        widgets::switch(ui, &mut draft.constant_power, "Constant power (M3)");
                        ui.end_row();
                    });
                let (ok, cancel) = crate::dialogs::buttons(ui, "OK", true, "");
                if ok {
                    done = Some(true);
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    done = Some(false);
                }
            });
        if let Some(ok) = done {
            if ok {
                for key in [
                    "mode",
                    "speed",
                    "max_power",
                    "interval",
                    "passes",
                    "air",
                    "constant_power",
                ] {
                    let (Ok(old), Ok(new)) = (stored.get(key), draft.get(key)) else {
                        continue;
                    };
                    if old != new {
                        out.push(run("material.set", [id.to_string(), key.to_owned(), new]));
                    }
                }
            }
            self.settings = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_form_builds_add_and_describe() {
        let mut form = EntryForm {
            id: None,
            color: Some(4),
            material: " Birch ".into(),
            thickness: 3.0,
            no_thickness: false,
            group: String::new(),
            description: "Cut".into(),
        };
        assert!(form.ready());
        assert_eq!(form.invocation().to_string(), "material.add Birch 3 Cut 4");
        form.no_thickness = true;
        assert!(!form.ready(), "a group is needed without a thickness");
        form.group = "Any".into();
        assert_eq!(
            form.invocation().to_string(),
            "material.add Birch - Cut 4 group=Any"
        );
        form.id = Some(7);
        assert_eq!(
            form.invocation().to_string(),
            "material.describe 7 Birch - Cut group=Any"
        );
    }
}
