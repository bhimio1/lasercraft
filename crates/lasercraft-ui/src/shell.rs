//! The main window: menu bar, tools, canvas, console and status bar.

use std::path::{Path, PathBuf};

use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_project::Project;

use crate::canvas::Canvas;
use crate::dialogs::Dialogs;
use crate::interaction::Tool;
use crate::keymap::{Chord, Keymap};
use crate::laser_panel::{self, LaserAction, LaserPanel, LaserView};
use crate::layers_panel::{LayersPanel, palette_strip};
use crate::preview::{Preview, PreviewWindow};

/// Tools in the tool palette: label, `ui.tool` argument.
const TOOLS: [(&str, &str, Tool); 6] = [
    ("Select", "select", Tool::Select),
    ("Line", "line", Tool::Line),
    ("Rectangle", "rect", Tool::Rect),
    ("Ellipse", "ellipse", Tool::Ellipse),
    ("Text", "text", Tool::Text),
    ("Edit Nodes", "nodes", Tool::Nodes),
];

/// M3 Tools menu entries that act on the selection: label, invocation.
const PATH_TOOLS: [(&str, &str); 9] = [
    ("Weld", "path.weld"),
    ("Union", "path.union"),
    ("Subtract", "path.subtract"),
    ("Intersect", "path.intersect"),
    ("Offset…", "ui.offset"),
    ("Convert to Path", "shape.to_path"),
    ("Close Path", "path.close"),
    ("Join Paths", "path.join"),
    ("Break Apart", "path.break_apart"),
];

/// M3 Arrange menu entries: submenu (empty for top level), label, invocation.
const ARRANGE_M3: [(&str, &str, &str); 22] = [
    ("", "Group", "group.create"),
    ("", "Ungroup", "group.ungroup"),
    ("Align", "Left Edges", "arrange.align left"),
    ("Align", "Right Edges", "arrange.align right"),
    ("Align", "Top Edges", "arrange.align top"),
    ("Align", "Bottom Edges", "arrange.align bottom"),
    ("Align", "Vertical Centers", "arrange.align center-x"),
    ("Align", "Horizontal Centers", "arrange.align center-y"),
    ("Align", "Centers", "arrange.align center"),
    (
        "Distribute",
        "Horizontal Spacing",
        "arrange.distribute h-spaced",
    ),
    (
        "Distribute",
        "Horizontal Centers",
        "arrange.distribute h-centered",
    ),
    (
        "Distribute",
        "Vertical Spacing",
        "arrange.distribute v-spaced",
    ),
    (
        "Distribute",
        "Vertical Centers",
        "arrange.distribute v-centered",
    ),
    (
        "Distribute",
        "Pack Horizontally",
        "arrange.move_together horizontal",
    ),
    (
        "Distribute",
        "Pack Vertically",
        "arrange.move_together vertical",
    ),
    ("Move on Bed", "Center", "arrange.move_to_bed center"),
    ("Move on Bed", "Top Left", "arrange.move_to_bed top-left"),
    ("Move on Bed", "Top Right", "arrange.move_to_bed top-right"),
    (
        "Move on Bed",
        "Bottom Left",
        "arrange.move_to_bed bottom-left",
    ),
    (
        "Move on Bed",
        "Bottom Right",
        "arrange.move_to_bed bottom-right",
    ),
    ("", "Grid Array…", "ui.grid_array"),
    ("", "Circular Array…", "ui.circular_array"),
];

/// Menu entries that act on the selection: label, invocation.
const SELECTION_EDITS: [(&str, &str); 5] = [
    ("Cut", "edit.cut"),
    ("Copy", "edit.copy"),
    ("Duplicate", "shape.duplicate"),
    ("Delete", "shape.delete"),
    ("Invert Selection", "select.invert"),
];

const ARRANGE: [(&str, &str); 10] = [
    ("Rotate 90° Clockwise", "shape.rotate 90"),
    ("Rotate 90° Counter-clockwise", "shape.rotate -90"),
    ("Mirror Horizontally", "shape.flip horizontal"),
    ("Mirror Vertically", "shape.flip vertical"),
    ("Forward", "shape.raise"),
    ("Backward", "shape.lower"),
    ("To Front", "shape.to_front"),
    ("To Back", "shape.to_back"),
    ("Select Next", "select.next"),
    ("Select Previous", "select.previous"),
];

/// How many console lines are kept.
const LOG_LINES: usize = 200;

/// What the shell needs to draw a frame. Read-only.
pub struct ShellView<'a> {
    /// The current project.
    pub project: &'a Project,
    /// Where the project was last loaded from or saved to.
    pub file: Option<&'a Path>,
    /// Whether there are changes since the last save or load.
    pub modified: bool,
    /// The command `edit.undo` would reverse, for the menu label.
    pub undo: Option<&'a Invocation>,
    /// Whether `edit.redo` has anything to do.
    pub can_redo: bool,
    /// The laser connection (M6).
    pub laser: &'a LaserView,
}

/// Something the user asked for. The app carries it out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Run a command through the session.
    Run(Invocation),
    /// Start a new project (the app asks first if there are unsaved changes).
    New,
    /// Open this file.
    Open(PathBuf),
    /// Save to the current file, or ask for a path if there is none.
    Save,
    /// Save to this file.
    SaveAs(PathBuf),
    /// Close the window.
    Quit,
    /// Plan the job and hand it to [`Shell::set_preview`]; the preview
    /// window is open and wants it.
    Preview,
    /// Something the app remembers about the laser (`docs/spec/laser.md`).
    Laser(LaserAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptKind {
    Open,
    SaveAs,
    Import,
    ExportGcode,
}

/// UI-only state of the main window.
pub struct Shell {
    keymap: Keymap,
    console: String,
    log: Vec<String>,
    prompt: Option<(PromptKind, String)>,
    canvas: Canvas,
    layers: LayersPanel,
    panels: bool,
    /// The notes window: open, with its text once loaded from the project.
    notes: Option<Option<String>>,
    /// The job preview window, while open.
    preview: Option<PreviewWindow>,
    /// Offset, array and text windows (M3).
    dialogs: Dialogs,
    /// The Laser panel's settings (M6).
    laser: LaserPanel,
    /// The right dock shows the Laser panel instead of Cuts / Layers.
    laser_tab: bool,
    /// Whether the running stream is paused, for the pause/resume key.
    paused: bool,
    /// Alt+I was pressed: open the selected image's adjustments.
    adjust_requested: bool,
    /// The theme has been installed on the context.
    themed: bool,
}

impl Shell {
    /// A shell using `keymap` for shortcuts.
    #[must_use]
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            console: String::new(),
            log: Vec::new(),
            prompt: None,
            canvas: Canvas::default(),
            layers: LayersPanel::default(),
            panels: true,
            notes: None,
            preview: None,
            dialogs: Dialogs::default(),
            laser: LaserPanel::default(),
            laser_tab: false,
            paused: false,
            adjust_requested: false,
            themed: false,
        }
    }

    /// Whether the job preview window is open (the app then keeps its
    /// contents up to date with [`Shell::set_preview`]).
    #[must_use]
    pub fn preview_open(&self) -> bool {
        self.preview.is_some()
    }

    /// Shows a planned job in the preview window, if it is open.
    pub fn set_preview(&mut self, preview: Preview) {
        if let Some(window) = &mut self.preview {
            window.set(preview);
        }
    }

    /// Adds a line to the console log.
    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > LOG_LINES {
            let excess = self.log.len() - LOG_LINES;
            self.log.drain(..excess);
        }
    }

    /// The canvas's zoom and pan, once it has been laid out.
    #[must_use]
    pub fn canvas_view(&self) -> Option<crate::view::View> {
        self.canvas.view()
    }

    /// How many image textures the canvas holds (for tests).
    #[must_use]
    pub fn image_textures(&self) -> usize {
        self.canvas.image_textures()
    }

    /// The active canvas tool.
    #[must_use]
    pub fn tool(&self) -> Tool {
        self.canvas.tool()
    }

    /// Shows the Laser panel in the right dock.
    pub fn show_laser_tab(&mut self) {
        self.laser_tab = true;
        self.panels = true;
    }

    /// Asks the user for a path to save to.
    pub fn prompt_save_as(&mut self) {
        self.prompt = Some((PromptKind::SaveAs, String::new()));
    }

    /// Draws one frame and returns what the user asked for.
    pub fn show(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>) -> Vec<Action> {
        // Ctrl+= / Ctrl+- / Ctrl+0 zoom the canvas, not the whole UI.
        ui.ctx().options_mut(|o| o.zoom_with_keyboard = false);
        if !std::mem::replace(&mut self.themed, true) {
            crate::theme::install(ui.ctx());
        }
        self.paused = view.laser.busy.as_ref().is_some_and(|b| b.paused);
        let mut actions = self.shortcuts(ui);
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui, view, &mut actions));
        let pointer = self.canvas.pointer();
        egui::Panel::bottom("status").show(ui, |ui| status_bar(ui, view, pointer, &mut actions));
        egui::Panel::bottom("console")
            .resizable(true)
            .default_size(120.0)
            .show(ui, |ui| self.console(ui, &mut actions));
        egui::Panel::bottom("palette").show(ui, |ui| {
            let (invocations, picked) = palette_strip(ui, view.project, self.canvas.color());
            actions.extend(invocations.into_iter().map(Action::Run));
            if let Some(color) = picked {
                self.canvas.set_color(color);
            }
        });
        if self.panels {
            egui::Panel::left("tools")
                .resizable(false)
                .show(ui, |ui| self.tool_palette(ui, &mut actions));
            egui::Panel::right("layers")
                .resizable(true)
                .default_size(290.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(!self.laser_tab, "Cuts / Layers")
                            .clicked()
                        {
                            self.laser_tab = false;
                        }
                        if ui.selectable_label(self.laser_tab, "Laser").clicked() {
                            self.laser_tab = true;
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.laser_tab {
                            self.laser.show(ui, view.laser, &mut actions);
                        } else {
                            let invocations = self.layers.show(ui, view.project);
                            actions.extend(invocations.into_iter().map(Action::Run));
                        }
                    });
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let drawn = self.canvas.show(ui, view.project);
                actions.extend(drawn.into_iter().map(Action::Run));
            });
        self.path_prompt(ui, &mut actions);
        self.notes_window(ui, view.project, &mut actions);
        if let Some(window) = &mut self.preview
            && !window.show(ui.ctx(), view.project)
        {
            self.preview = None;
        }
        laser_panel::windows(ui.ctx(), view.laser, &mut actions);
        if std::mem::take(&mut self.adjust_requested) && !self.dialogs.open_adjust(view.project) {
            self.log("select an image to adjust");
        }
        if let Some(at) = self.canvas.take_text_click() {
            self.dialogs.open_text(at);
        }
        let color = self.canvas.color();
        actions.extend(
            self.dialogs
                .show(ui.ctx(), view.project, color)
                .map(Action::Run),
        );
        actions
    }

    fn notes_window(&mut self, ui: &mut egui::Ui, project: &Project, actions: &mut Vec<Action>) {
        let Some(slot) = &mut self.notes else {
            return;
        };
        let text = slot.get_or_insert_with(|| project.notes().to_owned());
        let mut open = true;
        let mut save = false;
        egui::Window::new("Project notes")
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.add(egui::TextEdit::multiline(text).desired_rows(10));
                save = ui.button("Save notes").clicked();
            });
        if save {
            actions.push(Action::Run(Invocation::new(
                "project.set_notes",
                [text.clone()],
            )));
        }
        if save || !open {
            self.notes = None;
        }
    }

    fn shortcuts(&mut self, ui: &egui::Ui) -> Vec<Action> {
        if ui.ctx().text_edit_focused() {
            return Vec::new();
        }
        let chords: Vec<Chord> = ui.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        ..
                    } => Some(Chord::from_event(*key, *modifiers)),
                    // The windowing layer turns the platform's clipboard
                    // chords into these events instead of key presses.
                    egui::Event::Copy => {
                        Some(Chord::from_event(egui::Key::C, egui::Modifiers::COMMAND))
                    }
                    egui::Event::Cut => {
                        Some(Chord::from_event(egui::Key::X, egui::Modifiers::COMMAND))
                    }
                    egui::Event::Paste(_) | egui::Event::PasteImage(_) => {
                        Some(Chord::from_event(egui::Key::V, egui::Modifiers::COMMAND))
                    }
                    _ => None,
                })
                .collect()
        });
        let bound: Vec<Invocation> = chords
            .iter()
            .filter_map(|chord| self.keymap.lookup(chord).cloned())
            .collect();
        bound
            .into_iter()
            .filter_map(|invocation| self.resolve(invocation))
            .collect()
    }

    /// Maps a bound invocation to an action; `ui.*` names need UI state.
    fn resolve(&mut self, invocation: Invocation) -> Option<Action> {
        Some(match invocation.name.as_str() {
            "ui.new" => Action::New,
            "ui.open" => {
                self.prompt = Some((PromptKind::Open, String::new()));
                return None;
            }
            "ui.save" => Action::Save,
            "ui.save_as" => {
                self.prompt_save_as();
                return None;
            }
            "ui.quit" => Action::Quit,
            "ui.tool" => {
                match invocation.args.first().and_then(|t| Tool::parse(t)) {
                    Some(tool) => self.canvas.set_tool(tool),
                    None => self.log(format!("unknown tool in `{invocation}`")),
                }
                return None;
            }
            "ui.paste" => Action::Run(match self.canvas.pointer() {
                Some(p) => Invocation::new("edit.paste", [p.x.to_string(), p.y.to_string()]),
                None => Invocation::bare("edit.paste"),
            }),
            "ui.notes" => {
                self.notes = Some(None);
                return None;
            }
            "ui.import" => {
                self.prompt = Some((PromptKind::Import, String::new()));
                return None;
            }
            "ui.export_gcode" => {
                self.prompt = Some((PromptKind::ExportGcode, String::new()));
                return None;
            }
            "ui.adjust_image" => {
                // Opened in `show`, which has the project.
                self.adjust_requested = true;
                return None;
            }
            "ui.offset" | "ui.grid_array" | "ui.circular_array" => {
                self.dialogs.open(&invocation.name);
                return None;
            }
            "ui.toggle_panels" => {
                self.panels = !self.panels;
                return None;
            }
            "ui.preview" => {
                self.preview.get_or_insert_with(PreviewWindow::new);
                Action::Preview
            }
            "view.zoom_in" | "view.zoom_out" | "view.zoom_fit" => {
                self.canvas.zoom(&invocation.name);
                return None;
            }
            "ui.laser" => {
                self.show_laser_tab();
                return None;
            }
            name => match self.laser.resolve(name, self.paused) {
                Some(action) => return action,
                None => Action::Run(invocation),
            },
        })
    }

    fn menu_item(&mut self, ui: &mut egui::Ui, label: &str, bound: &str, enabled: bool) -> bool {
        let shortcut = self
            .keymap
            .chord_for(bound)
            .map(ToString::to_string)
            .unwrap_or_default();
        ui.add_enabled(enabled, egui::Button::new(label).shortcut_text(shortcut))
            .clicked()
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>, actions: &mut Vec<Action>) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                for (label, bound) in [
                    ("New", "ui.new"),
                    ("Open…", "ui.open"),
                    ("Save", "ui.save"),
                    ("Save As…", "ui.save_as"),
                    ("Import…", "ui.import"),
                    ("Save G-code…", "ui.export_gcode"),
                    ("Project Notes…", "ui.notes"),
                    ("Quit", "ui.quit"),
                ] {
                    if self.menu_item(ui, label, bound, true) {
                        actions.extend(self.resolve(Invocation::bare(bound)));
                    }
                }
            });
            ui.menu_button("Edit", |ui| {
                let undo = view
                    .undo
                    .map_or_else(|| "Undo".to_owned(), |inv| format!("Undo {}", inv.name));
                if self.menu_item(ui, &undo, "edit.undo", view.undo.is_some()) {
                    actions.push(Action::Run(Invocation::bare("edit.undo")));
                }
                if self.menu_item(ui, "Redo", "edit.redo", view.can_redo) {
                    actions.push(Action::Run(Invocation::bare("edit.redo")));
                }
                ui.separator();
                let selected = !view.project.selection().is_empty();
                for (label, line) in SELECTION_EDITS {
                    self.menu_line(ui, label, line, selected, actions);
                }
                let pastable = !view.project.clipboard().is_empty();
                self.menu_line(ui, "Paste", "ui.paste", pastable, actions);
                self.menu_line(
                    ui,
                    "Paste in Place",
                    "edit.paste_in_place",
                    pastable,
                    actions,
                );
                self.menu_line(ui, "Select All", "select.all", true, actions);
            });
            ui.menu_button("Tools", |ui| {
                for (label, name, _) in TOOLS {
                    self.menu_line(ui, label, &format!("ui.tool {name}"), true, actions);
                }
                ui.separator();
                let selected = !view.project.selection().is_empty();
                for (label, line) in PATH_TOOLS {
                    self.menu_line(ui, label, line, selected, actions);
                }
                ui.separator();
                let image = view.project.selection().into_iter().any(|id| {
                    view.project
                        .shape(id)
                        .is_some_and(|s| s.geometry().is_image())
                });
                self.menu_line(ui, "Adjust Image…", "ui.adjust_image", image, actions);
            });
            ui.menu_button("Arrange", |ui| {
                let selected = !view.project.selection().is_empty();
                for (label, line) in ARRANGE {
                    let enabled = selected || line.starts_with("select.");
                    self.menu_line(ui, label, line, enabled, actions);
                }
                ui.separator();
                self.arrange_m3(ui, selected, actions);
            });
            ui.menu_button("View", |ui| {
                self.menu_line(ui, "Zoom In", "view.zoom_in", true, actions);
                self.menu_line(ui, "Zoom Out", "view.zoom_out", true, actions);
                self.menu_line(ui, "Fit Bed", "view.zoom_fit", true, actions);
                ui.separator();
                self.menu_line(ui, "Side Panels", "ui.toggle_panels", true, actions);
                self.menu_line(ui, "Job Preview…", "ui.preview", true, actions);
            });
            ui.menu_button("Laser", |ui| {
                let ready = view.laser.ready;
                let idle = ready && view.laser.busy.is_none();
                self.menu_line(ui, "Laser Panel", "ui.laser", true, actions);
                ui.separator();
                self.menu_line(ui, "Start", "ui.device_start", idle, actions);
                self.menu_line(ui, "Frame", "ui.device_frame", idle, actions);
                self.menu_line(
                    ui,
                    "Frame Outline",
                    "ui.device_frame_outline",
                    idle,
                    actions,
                );
                self.menu_line(ui, "Pause / Resume", "ui.device_pause", ready, actions);
                self.menu_line(
                    ui,
                    "Stop",
                    "device.stop",
                    view.laser.port.is_some(),
                    actions,
                );
                ui.separator();
                self.menu_line(ui, "Home", "device.home", idle, actions);
                self.menu_line(ui, "Unlock", "device.unlock", idle, actions);
                self.menu_line(
                    ui,
                    "Disconnect",
                    "device.disconnect",
                    view.laser.port.is_some(),
                    actions,
                );
            });
        });
    }

    /// Grouping, alignment, distribution, bed placement and arrays, with
    /// submenus in the order of [`ARRANGE_M3`].
    fn arrange_m3(&mut self, ui: &mut egui::Ui, selected: bool, actions: &mut Vec<Action>) {
        let mut done: Vec<&str> = Vec::new();
        for (menu, label, line) in ARRANGE_M3 {
            if menu.is_empty() {
                self.menu_line(ui, label, line, selected, actions);
            } else if !done.contains(&menu) {
                done.push(menu);
                ui.menu_button(menu, |ui| {
                    for (_, label, line) in ARRANGE_M3.iter().filter(|e| e.0 == menu) {
                        self.menu_line(ui, label, line, selected, actions);
                    }
                });
            }
        }
    }

    /// A menu item that runs `line` (shown with its bound chord).
    fn menu_line(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        line: &str,
        enabled: bool,
        actions: &mut Vec<Action>,
    ) {
        if self.menu_item(ui, label, line, enabled)
            && let Ok(invocation) = Invocation::parse(line)
        {
            actions.extend(self.resolve(invocation));
        }
    }

    fn tool_palette(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.add_space(4.0);
        for (label, name, tool) in TOOLS {
            let line = format!("ui.tool {name}");
            let hint = self
                .keymap
                .chord_for(&line)
                .map_or_else(String::new, |c| format!(" ({c})"));
            let active = self.canvas.tool() == tool;
            if ui
                .selectable_label(active, label)
                .on_hover_text(format!("{label} tool{hint}"))
                .clicked()
            {
                actions.extend(self.resolve(Invocation::new("ui.tool", [name])));
            }
        }
    }

    fn console(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let input = ui.add(
            egui::TextEdit::singleline(&mut self.console)
                .hint_text("Type a command, e.g. project.set_work_area 400 300")
                .desired_width(f32::INFINITY),
        );
        if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let line = std::mem::take(&mut self.console);
            match Invocation::parse(&line) {
                Ok(invocation) => actions.extend(self.resolve(invocation)),
                Err(lasercraft_commands::CommandError::EmptyLine) => {}
                Err(err) => self.log(format!("error: {err}")),
            }
            input.request_focus();
        }
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for line in &self.log {
                    ui.monospace(line);
                }
            });
    }

    fn path_prompt(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Some((kind, text)) = &mut self.prompt else {
            return;
        };
        let (title, what) = match kind {
            PromptKind::Open => ("Open project", "Path to a .lcproj file:"),
            PromptKind::SaveAs => ("Save project as", "Path to a .lcproj file:"),
            PromptKind::Import => ("Import", "Path to an .svg, .dxf, .png or .jpg file:"),
            PromptKind::ExportGcode => ("Save G-code", "Path to the .gcode file to write:"),
        };
        let mut open = true;
        let mut done = None;
        egui::Window::new(title)
            .collapsible(false)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.label(what);
                let field = ui.text_edit_singleline(text);
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.button("OK").clicked() || enter) && !text.trim().is_empty() {
                    done = Some(PathBuf::from(text.trim()));
                }
            });
        if let Some(path) = done {
            actions.push(match kind {
                PromptKind::Open => Action::Open(path),
                PromptKind::SaveAs => Action::SaveAs(path),
                PromptKind::Import => Action::Run(Invocation::new(
                    "project.import",
                    [path.display().to_string()],
                )),
                PromptKind::ExportGcode => Action::Run(Invocation::new(
                    "project.export_gcode",
                    [path.display().to_string()],
                )),
            });
            self.prompt = None;
        } else if !open {
            self.prompt = None;
        }
    }
}

fn status_bar(
    ui: &mut egui::Ui,
    view: &ShellView<'_>,
    pointer: Option<Point>,
    actions: &mut Vec<Action>,
) {
    ui.horizontal(|ui| {
        // Stop and Pause stay reachable whatever else is open (safety rule 7).
        if let Some(port) = &view.laser.port {
            laser_panel::stop_and_pause(ui, view.laser, actions);
            ui.label(format!("Laser {} on {port}", view.laser.state));
            ui.separator();
        }
        let file = view
            .file
            .map_or_else(|| "not saved".to_owned(), |p| p.display().to_string());
        let marker = if view.modified { " •" } else { "" };
        ui.label(format!("{}{marker} — {file}", view.project.name()));
        ui.separator();
        let area = view.project.work_area();
        ui.label(format!(
            "Work area {} × {} mm, origin {}",
            area.width_mm, area.height_mm, area.origin
        ));
        if let Some(p) = pointer {
            ui.separator();
            ui.monospace(format!("x {:.2}  y {:.2} mm", p.x, p.y));
        }
        let selection = view.project.selection();
        if let Ok(Some(b)) = view.project.bounds_of(&selection) {
            ui.separator();
            ui.label(format!(
                "{} selected, {:.2} × {:.2} mm",
                selection.len(),
                b.width(),
                b.height()
            ));
        }
    });
}
