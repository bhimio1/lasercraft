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
use crate::menus;
use crate::preview::{Preview, PreviewWindow};

/// The drawing tools: label, `ui.tool` argument.
const TOOLS: [(&str, &str, Tool); 6] = [
    ("Select", "select", Tool::Select),
    ("Line", "line", Tool::Line),
    ("Rectangle", "rect", Tool::Rect),
    ("Ellipse", "ellipse", Tool::Ellipse),
    ("Text", "text", Tool::Text),
    ("Edit nodes", "nodes", Tool::Nodes),
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

    fn menu_item(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        bound: &str,
        enabled: bool,
        why: &str,
    ) -> bool {
        let shortcut = self
            .keymap
            .chord_for(bound)
            .map(Chord::label)
            .unwrap_or_default();
        let mut response =
            ui.add_enabled(enabled, egui::Button::new(label).shortcut_text(shortcut));
        if !why.is_empty() {
            response = response.on_disabled_hover_text(why);
        }
        response.clicked()
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>, actions: &mut Vec<Action>) {
        let cx = menus::Context::of(view.project);
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| self.menu_rows(ui, menus::FILE, &cx, actions));
            ui.menu_button("Edit", |ui| {
                let undo = view.undo.map_or_else(
                    || "Undo".to_owned(),
                    |inv| format!("Undo {}", lasercraft_commands::action_name(inv)),
                );
                if self.menu_item(ui, &undo, "edit.undo", view.undo.is_some(), "") {
                    actions.push(Action::Run(Invocation::bare("edit.undo")));
                }
                if self.menu_item(ui, "Redo", "edit.redo", view.can_redo, "") {
                    actions.push(Action::Run(Invocation::bare("edit.redo")));
                }
                self.menu_rows(ui, menus::EDIT, &cx, actions);
            });
            ui.menu_button("Tools", |ui| self.menu_rows(ui, menus::TOOLS, &cx, actions));
            ui.menu_button("Arrange", |ui| {
                self.menu_rows(ui, menus::ARRANGE, &cx, actions);
            });
            ui.menu_button("Machine", |ui| self.machine_menu(ui, view, actions));
            ui.menu_button("View", |ui| {
                self.menu_rows(ui, menus::VIEW, &cx, actions);
                ui.separator();
                let shortcut = self
                    .keymap
                    .chord_for("ui.toggle_panels")
                    .map(Chord::label)
                    .unwrap_or_default();
                let tick = egui::Button::new("Side panels")
                    .selected(self.panels)
                    .shortcut_text(shortcut);
                if ui.add(tick).clicked() {
                    actions.extend(self.resolve(Invocation::bare("ui.toggle_panels")));
                }
            });
        });
    }

    /// Draws menu rows from a table in [`menus`].
    fn menu_rows(
        &mut self,
        ui: &mut egui::Ui,
        rows: &[menus::Row],
        cx: &menus::Context,
        actions: &mut Vec<Action>,
    ) {
        for row in rows {
            match *row {
                menus::Row::Item(label, line, need) => {
                    self.menu_line(ui, label, line, need.met(cx), need.reason(), actions);
                }
                menus::Row::Tool(label, name) => {
                    let line = format!("ui.tool {name}");
                    let active = TOOLS
                        .iter()
                        .any(|t| t.1 == name && t.2 == self.canvas.tool());
                    let shortcut = self
                        .keymap
                        .chord_for(&line)
                        .map(Chord::label)
                        .unwrap_or_default();
                    let button = egui::Button::new(label)
                        .selected(active)
                        .shortcut_text(shortcut);
                    if ui.add(button).clicked() {
                        actions.extend(self.resolve(Invocation::new("ui.tool", [name])));
                    }
                }
                menus::Row::Sub(label, need, rows) => {
                    ui.add_enabled_ui(need.met(cx), |ui| {
                        ui.menu_button(label, |ui| self.menu_rows(ui, rows, cx, actions))
                            .response
                            .on_disabled_hover_text(need.reason());
                    });
                }
                menus::Row::Sep => {
                    ui.separator();
                }
            }
        }
    }

    /// Machine: connection, job control, homing and the user origin. Pause
    /// and Stop are enabled whenever a machine is connected.
    fn machine_menu(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>, actions: &mut Vec<Action>) {
        const OFFLINE: &str = "Connect to a machine first.";
        let laser = view.laser;
        let connected = laser.port.is_some();
        let alarm = laser.state.starts_with("Alarm");
        let idle = laser.ready && laser.busy.is_none() && !alarm;
        let output = has_output(view.project);
        let why = |ok: bool| if ok { "" } else { OFFLINE };
        if connected {
            self.menu_line(
                ui,
                "Disconnect",
                "device.disconnect",
                laser.busy.is_none(),
                "Stop the job first.",
                actions,
            );
        } else if ui
            .add_enabled(self.laser.can_connect(), egui::Button::new("Connect"))
            .on_disabled_hover_text("Choose a port in the Machine panel first.")
            .clicked()
        {
            actions.extend(self.laser.connect());
        }
        ui.separator();
        let no_output = if idle {
            "Nothing to cut: no output layer has shapes."
        } else {
            OFFLINE
        };
        self.menu_line(
            ui,
            "Start",
            "ui.device_start",
            idle && output,
            no_output,
            actions,
        );
        let (pause, hint) = match &laser.busy {
            Some(busy) if busy.paused => ("Resume", ""),
            Some(busy) if busy.constant_power => ("Pause (motion only)", PAUSE_M3),
            _ => ("Pause", ""),
        };
        let shortcut = self
            .keymap
            .chord_for("ui.device_pause")
            .map(Chord::label)
            .unwrap_or_default();
        let mut response = ui
            .add_enabled(connected, egui::Button::new(pause).shortcut_text(shortcut))
            .on_disabled_hover_text(OFFLINE);
        if !hint.is_empty() {
            response = response.on_hover_text(hint);
        }
        if response.clicked() {
            actions.extend(self.resolve(Invocation::bare("ui.device_pause")));
        }
        self.menu_line(ui, "Stop", "device.stop", connected, OFFLINE, actions);
        ui.separator();
        self.menu_line(
            ui,
            "Frame",
            "ui.device_frame",
            idle && output,
            no_output,
            actions,
        );
        self.menu_line(
            ui,
            "Frame outline",
            "ui.device_frame_outline",
            idle && output,
            no_output,
            actions,
        );
        ui.separator();
        self.menu_line(
            ui,
            "Home",
            "device.home",
            laser.ready && laser.busy.is_none(),
            why(connected),
            actions,
        );
        let go = idle && laser.user_origin.is_some();
        if ui
            .add_enabled(go, egui::Button::new("Go to user origin"))
            .on_disabled_hover_text(if idle {
                "Set a user origin first."
            } else {
                OFFLINE
            })
            .clicked()
            && let Some(origin) = laser.user_origin
        {
            actions.extend(self.laser.go_to(origin));
        }
        if ui
            .add_enabled(
                idle && laser.position.is_some(),
                egui::Button::new("Set user origin"),
            )
            .on_disabled_hover_text(OFFLINE)
            .clicked()
        {
            actions.push(Action::Laser(LaserAction::SetOrigin));
        }
        if ui
            .add_enabled(
                laser.user_origin.is_some(),
                egui::Button::new("Clear user origin"),
            )
            .on_disabled_hover_text("No user origin is set.")
            .clicked()
        {
            actions.push(Action::Laser(LaserAction::ClearOrigin));
        }
        let clear = connected && alarm && laser.busy.is_none();
        self.menu_line(
            ui,
            "Clear alarm",
            "device.unlock",
            clear,
            "Only needed after an alarm.",
            actions,
        );
    }

    /// A menu item that runs `line` (shown with its bound chord).
    fn menu_line(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        line: &str,
        enabled: bool,
        why: &str,
        actions: &mut Vec<Action>,
    ) {
        if self.menu_item(ui, label, line, enabled, why)
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
                .map_or_else(String::new, |c| format!(" ({})", c.label()));
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

/// What the Pause item says while a constant-power (`M3`) job runs.
const PAUSE_M3: &str = "Stops motion at once. The laser may stay on until the head \
    stops; LaserCraft then switches it off.";

/// Whether an output layer has shapes: a job would cut something.
fn has_output(project: &Project) -> bool {
    project
        .used_layers()
        .into_iter()
        .any(|c| project.layer(c).is_ok_and(|l| l.output))
}
