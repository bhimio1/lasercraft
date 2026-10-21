//! The main window: menu bar, work area outline, console and status bar.

use std::path::{Path, PathBuf};

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use lasercraft_commands::Invocation;
use lasercraft_project::{Origin, Project};

use crate::keymap::{Chord, Keymap};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptKind {
    Open,
    SaveAs,
}

/// UI-only state of the main window.
pub struct Shell {
    keymap: Keymap,
    console: String,
    log: Vec<String>,
    prompt: Option<(PromptKind, String)>,
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

    /// Asks the user for a path to save to.
    pub fn prompt_save_as(&mut self) {
        self.prompt = Some((PromptKind::SaveAs, String::new()));
    }

    /// Draws one frame and returns what the user asked for.
    pub fn show(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>) -> Vec<Action> {
        let mut actions = self.shortcuts(ui);
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui, view, &mut actions));
        egui::Panel::bottom("status").show(ui, |ui| status_bar(ui, view));
        egui::Panel::bottom("console")
            .resizable(true)
            .default_size(140.0)
            .show(ui, |ui| self.console(ui, &mut actions));
        egui::CentralPanel::default().show(ui, |ui| work_area(ui, view.project));
        self.path_prompt(ui, &mut actions);
        actions
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
            _ => Action::Run(invocation),
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
            });
        });
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
        let title = match kind {
            PromptKind::Open => "Open project",
            PromptKind::SaveAs => "Save project as",
        };
        let mut open = true;
        let mut done = None;
        egui::Window::new(title)
            .collapsible(false)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.label("Path to a .lcproj file:");
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
            });
            self.prompt = None;
        } else if !open {
            self.prompt = None;
        }
    }
}

fn status_bar(ui: &mut egui::Ui, view: &ShellView<'_>) {
    ui.horizontal(|ui| {
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
    });
}

/// Draws the work area to scale with its origin corner marked. The full
/// canvas arrives with M1.
fn work_area(ui: &mut egui::Ui, project: &Project) {
    let area = project.work_area();
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let space = response.rect.shrink(24.0);
    let scale = (space.width() / area.width_mm as f32).min(space.height() / area.height_mm as f32);
    let size = Vec2::new(area.width_mm as f32, area.height_mm as f32) * scale;
    let bed = Rect::from_center_size(space.center(), size);
    let stroke = Stroke::new(1.5, ui.visuals().strong_text_color());
    painter.rect_stroke(bed, 0.0, stroke, StrokeKind::Middle);
    // Screen y grows downwards, so the front of the bed is at the bottom.
    let corner = match area.origin {
        Origin::FrontLeft => bed.left_bottom(),
        Origin::FrontRight => bed.right_bottom(),
        Origin::RearLeft => bed.left_top(),
        Origin::RearRight => bed.right_top(),
    };
    painter.circle_filled(corner, 5.0, Color32::from_rgb(220, 60, 40));
    painter.text(
        Pos2::new(bed.center().x, bed.top() - 4.0),
        egui::Align2::CENTER_BOTTOM,
        format!("{} × {} mm", area.width_mm, area.height_mm),
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );
}
