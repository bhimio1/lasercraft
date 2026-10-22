//! The main window: menu bar, tools, canvas, console and status bar.

use std::path::{Path, PathBuf};

use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_project::Project;
use lasercraft_workshop::Workshop;

use crate::camera_panel::{CameraAction, CameraPanel, CameraView, PanelWish};
use crate::camera_windows::CameraWindows;
use crate::canvas::Canvas;
use crate::device_dialogs::DeviceWindows;
use crate::dialogs::Dialogs;
use crate::dock::{self, Dock, DockPanel};
use crate::interaction::Tool;
use crate::keymap::{Chord, Keymap};
use crate::laser_panel::{self, LaserAction, LaserPanel, LaserView};
use crate::layers_panel::{LayersPanel, palette_strip};
use crate::materials_panel::MaterialsPanel;
use crate::menus;
use crate::preview::{Preview, PreviewWindow};
use crate::setup_windows::SetupWindows;
use crate::test_windows::{TestInfo, TestKind, TestRequest, TestWindow};
use crate::theme::{Tokens, size};
use crate::tool_column::ToolColumn;
use crate::trace_window::{TraceInfo, TraceRequest, TraceSource, TraceWindow};
use crate::widgets::Bounded;
use crate::widgets::{self, Band};

/// The drawing tools: label, `ui.tool` argument.
const TOOLS: [(&str, &str, Tool); 6] = [
    ("Select", "select", Tool::Select),
    ("Line", "line", Tool::Line),
    ("Rectangle", "rect", Tool::Rect),
    ("Ellipse", "ellipse", Tool::Ellipse),
    ("Text", "text", Tool::Text),
    ("Edit nodes", "nodes", Tool::Nodes),
];

/// How many command line log lines are kept.
const LOG_LINES: usize = 200;

/// Bound actions that work even while a text field has focus.
const SAFETY_ACTIONS: [&str; 2] = ["ui.device_pause", "device.stop"];

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
    /// Device profiles, material libraries and test presets (M8).
    pub workshop: &'a Workshop,
    /// Cameras' pictures and the overlay (M9).
    pub camera: &'a CameraView,
}

/// Something the user asked for. The app carries it out.
#[derive(Debug, Clone, PartialEq)]
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
    /// Something only the app can do with cameras (`docs/spec/camera.md`).
    Camera(CameraAction),
}

/// What a path prompt is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PromptKind {
    Open,
    SaveAs,
    Import,
    ExportGcode,
    /// Runs `command` with the path inserted among its arguments at `at`.
    Run {
        title: &'static str,
        what: &'static str,
        command: Invocation,
        at: usize,
    },
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
    /// The dock's shown panels and front tabs.
    dock: Dock,
    /// The drawing tools column (Shape tools face, flyout).
    tools: ToolColumn,
    /// The Materials panel (M8).
    materials: MaterialsPanel,
    /// The device profile windows (M8).
    devices: DeviceWindows,
    /// Rotary, taper and print registration windows (M9).
    setups: SetupWindows,
    /// The Camera panel (M9).
    camera_panel: CameraPanel,
    /// Camera list, Add camera and Camera setup (M9).
    camera_windows: CameraWindows,
    /// The Camera panel was shown for the profile that has cameras.
    camera_shown_for: Option<String>,
    /// Open test pattern windows (M8).
    tests: Vec<TestWindow>,
    /// A test window to open (needs the workshop, which `show` has).
    open_test: Option<TestKind>,
    /// The preview window shows a test pattern, not the project.
    pattern_preview: bool,
    /// Machine → Edit device profile… was chosen.
    edit_profile: bool,
    /// A pointer button is down (test settings are not remembered yet).
    dragging: bool,
    /// A setup window to open once the project is at hand.
    setup_request: Option<String>,
    /// The last pointer position over the canvas.
    pointer: Option<Point>,
    /// Whether the running stream is paused, for the pause/resume key.
    paused: bool,
    /// Alt+I was pressed: open the selected image's adjustments.
    adjust_requested: bool,
    /// Trace to vectors was chosen: open it on the selected image.
    trace_requested: bool,
    /// The Trace to vectors window.
    trace: Option<TraceWindow>,
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
            dock: Dock::default(),
            tools: ToolColumn::default(),
            materials: MaterialsPanel::default(),
            devices: DeviceWindows::default(),
            setups: SetupWindows::default(),
            camera_panel: CameraPanel::default(),
            camera_windows: CameraWindows::default(),
            camera_shown_for: None,
            tests: Vec::new(),
            open_test: None,
            pattern_preview: false,
            edit_profile: false,
            dragging: false,
            setup_request: None,
            pointer: None,
            paused: false,
            adjust_requested: false,
            trace_requested: false,
            trace: None,
            themed: false,
        }
    }

    /// Whether the job preview window is open on the project (the app then
    /// keeps its contents up to date with [`Shell::set_preview`]).
    #[must_use]
    pub fn preview_open(&self) -> bool {
        self.preview.is_some() && !self.pattern_preview
    }

    /// Tells the shell a command failed; registration steps show the
    /// "Registration failed" message (`docs/spec/registration.md`).
    pub fn command_failed(&mut self, invocation: &Invocation, message: &str) {
        if invocation.name.starts_with("registration.") {
            let text = message
                .split_once(": ")
                .filter(|(head, _)| head.starts_with("invalid argument") || head.contains('.'))
                .map_or(message, |(_, rest)| rest);
            self.setups.registration_failed(text.to_owned());
        }
    }

    /// The test windows' settings to check, one per changed window.
    pub fn test_requests(&mut self) -> Vec<TestRequest> {
        let settled = !self.dragging;
        self.tests
            .iter_mut()
            .filter_map(|w| w.request(settled))
            .collect()
    }

    /// What the Trace to vectors window wants traced, when its settings
    /// changed.
    pub fn trace_request(&mut self) -> Option<TraceRequest> {
        self.trace.as_mut().and_then(TraceWindow::request)
    }

    /// The app's answer to a [`TraceRequest`].
    pub fn set_trace_info(&mut self, info: TraceInfo) {
        if let Some(window) = &mut self.trace {
            window.set_info(info);
        }
    }

    /// The app's answer to a [`TestRequest`] of `kind`.
    pub fn set_test_info(&mut self, kind: TestKind, info: TestInfo) {
        let Some(window) = self.tests.iter_mut().find(|w| w.kind() == kind) else {
            return;
        };
        if let Some(preview) = window.set_info(info) {
            self.preview
                .get_or_insert_with(PreviewWindow::new)
                .set(preview);
            self.pattern_preview = true;
        }
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

    /// Shows the Machine panel (and the side panels, if F12 hid them).
    pub fn show_machine_panel(&mut self) {
        self.dock.show(DockPanel::Machine);
        self.panels = true;
    }

    /// Brings a dock panel to the front of its group.
    pub fn show_panel(&mut self, panel: DockPanel) {
        self.dock.show(panel);
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
        let connected = view.laser.port.is_some();
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui, view, &mut actions));
        // Reserved here, drawn last above every window (safety rule 7).
        let job_bar = connected.then(|| {
            egui::Panel::top("job_bar")
                .exact_size(size::ROW + 2.0 * size::GAP_TIGHT)
                .show(ui, |_| {})
                .response
                .rect
        });
        let mut bounds = ui.ctx().content_rect();
        if let Some(bar) = job_bar {
            bounds.min.y = bar.max.y;
        }
        widgets::set_window_bounds(ui.ctx(), bounds);
        if let Some(p) = self.canvas.pointer() {
            self.pointer = Some(p);
        }
        let live = self.canvas.pointer().is_some();
        egui::Panel::bottom("status")
            .exact_size(size::ROW + 2.0 * size::GAP_TIGHT)
            .show(ui, |ui| self.status_bar(ui, view, live));
        let command_line = if self.dock.log_open {
            egui::Panel::bottom("command_log")
                .resizable(true)
                .default_size(140.0)
        } else {
            egui::Panel::bottom("command_line").exact_size(size::ROW + 2.0 * size::GAP_TIGHT)
        };
        command_line.show(ui, |ui| self.command_line(ui, &mut actions));
        if self.dock.palette {
            egui::Panel::bottom("palette").show(ui, |ui| {
                let (invocations, picked) = palette_strip(ui, view.project, self.canvas.color());
                actions.extend(invocations.into_iter().map(Action::Run));
                if let Some(color) = picked {
                    self.canvas.set_color(color);
                }
            });
        }
        if self.panels {
            if self.dock.tools {
                let frame = egui::Frame::NONE
                    .fill(Tokens::of(ui.ctx()).panel)
                    .inner_margin(egui::Margin::symmetric(size::GAP as i8, 0));
                egui::Panel::left("tools")
                    .resizable(false)
                    .exact_size(size::ICON + 2.0 * size::GAP_TIGHT + 2.0 * size::GAP)
                    .frame(frame)
                    .show(ui, |ui| self.drawing_tools(ui, &mut actions));
            }
            if self.dock.any() {
                let width = ui.ctx().content_rect().width() * 0.25;
                egui::Panel::right("dock")
                    .resizable(true)
                    .default_size(width.max(290.0))
                    .min_size(260.0)
                    .frame(egui::Frame::NONE.fill(Tokens::of(ui.ctx()).panel))
                    .show(ui, |ui| self.dock(ui, view, &mut actions));
            }
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.canvas.set_ghost(self.setups.ghost(view.project));
                self.canvas
                    .set_overlay(view.camera.overlay.clone(), view.camera.overlay_revision);
                let drawn = self.canvas.show(ui, view.project);
                actions.extend(drawn.into_iter().map(Action::Run));
            });
        self.path_prompt(ui, &mut actions);
        self.notes_window(ui, view.project, &mut actions);
        if let Some(window) = &mut self.preview
            && !window.show(ui.ctx(), view.project)
        {
            self.preview = None;
            self.pattern_preview = false;
        }
        if std::mem::take(&mut self.adjust_requested) && !self.dialogs.open_adjust(view.project) {
            self.log("Cannot adjust image: no image is selected. Select an image first.");
        }
        if std::mem::take(&mut self.trace_requested) {
            let image = view.project.selection().into_iter().find(|id| {
                view.project
                    .shape(*id)
                    .is_some_and(|s| s.geometry().image().is_some())
            });
            match image {
                Some(id) => self.trace = Some(TraceWindow::new(TraceSource::Image(id))),
                None => self.log("Cannot trace: no image is selected. Select an image first."),
            }
        }
        if let Some(at) = self.canvas.take_text_click() {
            self.dialogs.open_text(at);
        }
        let color = self.canvas.color();
        if let Some(window) = &mut self.trace
            && !window.show(ui.ctx(), color, &mut actions)
        {
            self.trace = None;
        }
        actions.extend(
            self.dialogs
                .show(ui.ctx(), view.project, color)
                .map(Action::Run),
        );
        self.test_windows(ui.ctx(), view, &mut actions);
        match self.setup_request.take().as_deref() {
            Some("ui.rotary") => self.setups.open_rotary(view.project),
            Some("ui.taper") => self.setups.open_taper(view.project),
            _ => {}
        }
        self.setups
            .show(ui.ctx(), view.project, view.laser, &mut actions);
        self.cameras(ui.ctx(), view, &mut actions);
        if std::mem::take(&mut self.edit_profile) {
            self.devices.open_current(view.project, view.workshop);
        }
        let connected = view.laser.port.is_some();
        actions.extend(
            self.devices
                .show(ui.ctx(), view.workshop, connected)
                .into_iter()
                .map(Action::Run),
        );
        if let Some(kind) = self.devices.prompt.take() {
            self.prompt = Some((kind, String::new()));
        }
        if let Some(rect) = job_bar {
            self.job_bar(ui.ctx(), rect, view.laser, &mut actions);
        }
        if self.laser.take_export() {
            actions.extend(self.resolve(Invocation::bare("ui.export_gcode")));
        }
        if std::mem::take(&mut self.laser.show_console) {
            self.show_panel(DockPanel::Console);
        }
        actions
    }

    /// Whether camera pictures are wanted now: the Camera panel is in
    /// front or a camera window is open.
    #[must_use]
    pub fn cameras_live(&self) -> bool {
        self.camera_windows.live()
            || (self.panels && self.dock.front_of(true) == Some(DockPanel::Camera))
    }

    /// The camera windows, the panel's wishes, and showing the Camera panel
    /// once a camera is set up.
    fn cameras(&mut self, ctx: &egui::Context, view: &ShellView<'_>, actions: &mut Vec<Action>) {
        let device = view.project.device().map(str::to_owned);
        let has = device
            .as_deref()
            .and_then(|d| view.workshop.cameras(d).ok())
            .is_some_and(|c| !c.is_empty());
        if has && self.camera_shown_for != device {
            self.camera_shown_for.clone_from(&device);
            if !self.dock.is_shown(DockPanel::Camera) {
                self.dock.show_tab(DockPanel::Camera);
            }
        }
        match self.camera_panel.wants.take() {
            Some(PanelWish::Add) => self.camera_windows.open_add(),
            Some(PanelWish::Import) => self.camera_windows.import(),
            Some(PanelWish::Setup(c)) => self.camera_windows.open_setup(&c),
            Some(PanelWish::Lens(c)) => self.camera_windows.open_lens(&c),
            Some(PanelWish::Align(c)) => self.camera_windows.open_align(view.project, &c),
            Some(PanelWish::Trace) => self.trace = Some(TraceWindow::new(TraceSource::Camera)),
            None => {}
        }
        self.camera_windows.show(
            ctx,
            view.project,
            view.workshop,
            view.camera,
            view.laser,
            &self.keymap,
            actions,
        );
        if let Some(kind) = self.camera_windows.prompt.take() {
            self.prompt = Some((kind, String::new()));
        }
    }

    /// Opens a requested test window and draws the open ones.
    fn test_windows(
        &mut self,
        ctx: &egui::Context,
        view: &ShellView<'_>,
        actions: &mut Vec<Action>,
    ) {
        if let Some(kind) = self.open_test.take()
            && !self.tests.iter().any(|w| w.kind() == kind)
        {
            self.tests.push(TestWindow::new(kind, view.workshop));
        }
        let mut tests = std::mem::take(&mut self.tests);
        tests.retain_mut(|w| {
            let open = w.show(
                ctx,
                view.project,
                view.workshop,
                &self.laser,
                view.laser,
                &self.keymap,
                actions,
            );
            if let Some(kind) = w.prompt.take() {
                self.prompt = Some((kind, String::new()));
            }
            open
        });
        self.tests = tests;
        self.dragging = ctx.input(|i| i.pointer.any_down());
    }

    /// The job bar, drawn in a layer above every window and menu so it is
    /// never covered.
    fn job_bar(
        &mut self,
        ctx: &egui::Context,
        rect: egui::Rect,
        laser: &LaserView,
        actions: &mut Vec<Action>,
    ) {
        let keymap = &self.keymap;
        let clicked = egui::Area::new(egui::Id::unique("job_bar"))
            .order(egui::Order::Tooltip)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                ui.set_clip_rect(rect);
                let inner = rect.shrink2(egui::vec2(size::GAP, size::GAP_TIGHT));
                ui.painter().rect_filled(rect, 0.0, Tokens::of(ctx).window);
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    laser_panel::job_bar(ui, laser, keymap, actions)
                })
                .inner
            })
            .inner;
        if clicked {
            self.show_machine_panel();
        }
    }

    /// The right dock: the upper group (Layers, Jog, Console) over the
    /// lower group (Machine, Materials).
    fn dock(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>, actions: &mut Vec<Action>) {
        let upper = self.dock.front_of(true);
        if let Some(front) = self.dock.front_of(false) {
            let lower = egui::Panel::bottom("dock_lower")
                .frame(egui::Frame::NONE)
                .resizable(upper.is_some());
            let lower = if upper.is_some() {
                lower.default_size(ui.available_height() * 0.45)
            } else {
                lower.exact_size(ui.available_height())
            };
            lower.show(ui, |ui| self.dock_group(ui, false, front, view, actions));
        }
        if let Some(front) = upper {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| self.dock_group(ui, true, front, view, actions));
        }
    }

    /// One dock group: title strip, tab strip along the bottom, body.
    fn dock_group(
        &mut self,
        ui: &mut egui::Ui,
        upper: bool,
        front: DockPanel,
        view: &ShellView<'_>,
        actions: &mut Vec<Action>,
    ) {
        let t = Tokens::of(ui.ctx());
        if dock::title_strip(ui, front.name()) {
            self.dock.hide(front);
        }
        let tabs = self.dock.tabs(upper);
        if tabs.len() > 1 {
            egui::Panel::bottom(if upper {
                "dock_tabs"
            } else {
                "dock_lower_tabs"
            })
            .frame(egui::Frame::NONE.fill(t.window))
            .show(ui, |ui| {
                if let Some(tab) = dock::tab_strip(ui, &tabs, front) {
                    self.dock.show(tab);
                }
            });
        }
        let body = egui::Frame::new()
            .fill(t.panel)
            .inner_margin(egui::Margin::same(size::GAP as i8));
        egui::CentralPanel::default().frame(body).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            self.dock_body(ui, front, view, actions);
        });
    }

    fn dock_body(
        &mut self,
        ui: &mut egui::Ui,
        panel: DockPanel,
        view: &ShellView<'_>,
        actions: &mut Vec<Action>,
    ) {
        match panel {
            DockPanel::Layers => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let invocations = self.layers.show(ui, view.project, view.workshop);
                    actions.extend(invocations.into_iter().map(Action::Run));
                });
            }
            DockPanel::Jog => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.laser.jog(ui, view.laser, &self.keymap, actions);
                });
            }
            DockPanel::Console => self.laser.console(ui, view.laser, actions),
            DockPanel::Machine => {
                let output = has_output(view.project);
                let devices = &mut self.devices;
                let mut run = Vec::new();
                let connected = view.laser.port.is_some();
                self.laser
                    .machine(ui, view.laser, &self.keymap, output, actions, |ui| {
                        let (invocations, chosen) =
                            devices.device_row(ui, view.project, view.workshop, connected);
                        run = invocations;
                        chosen.map(|p| (p.port, p.baud))
                    });
                actions.extend(run.into_iter().map(Action::Run));
            }
            DockPanel::Camera => {
                egui::ScrollArea::vertical()
                    .id_salt("camera-scroll")
                    .show(ui, |ui| {
                        self.camera_panel.show(
                            ui,
                            view.project,
                            view.workshop,
                            view.camera,
                            actions,
                        );
                    });
            }
            DockPanel::Materials => {
                egui::ScrollArea::vertical()
                    .id_salt("materials-scroll")
                    .show(ui, |ui| {
                        let target = self.layers.editing();
                        let invocations =
                            self.materials.show(ui, view.project, view.workshop, target);
                        actions.extend(invocations.into_iter().map(Action::Run));
                    });
                if let Some(kind) = self.materials.prompt.take() {
                    self.prompt = Some((kind, String::new()));
                }
            }
        }
    }

    fn notes_window(&mut self, ui: &mut egui::Ui, project: &Project, actions: &mut Vec<Action>) {
        let Some(slot) = &mut self.notes else {
            return;
        };
        let text = slot.get_or_insert_with(|| project.notes().to_owned());
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        let changed = text != project.notes();
        egui::Window::new("Project notes")
            .collapsible(false)
            .open(&mut open)
            .bounded(ui.ctx())
            .show(ui.ctx(), |ui| {
                ui.add(
                    egui::TextEdit::multiline(text)
                        .desired_rows(10)
                        .desired_width(f32::INFINITY),
                );
                let (ok, cancelled) = crate::dialogs::buttons(ui, "Save notes", true, "");
                // Enter is a new line; Ctrl+Enter saves.
                let commit = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);
                save = ok || commit;
                cancel = cancelled || ui.input(|i| i.key_pressed(egui::Key::Escape));
            });
        if save && changed {
            actions.push(Action::Run(Invocation::new(
                "project.set_notes",
                [text.clone()],
            )));
        }
        if save || cancel || !open {
            self.notes = None;
        }
    }

    fn shortcuts(&mut self, ui: &egui::Ui) -> Vec<Action> {
        // Pause and Stop act even while a text field has focus (safety rule 7).
        let typing = ui.ctx().text_edit_focused();
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
            .filter(|inv| !typing || SAFETY_ACTIONS.contains(&inv.name.as_str()))
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
                    None => self.log(format!("error: unknown tool in {invocation}")),
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
            "ui.trace_image" => {
                self.trace_requested = true;
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
                self.pattern_preview = false;
                Action::Preview
            }
            "ui.rotary" => {
                self.setup_request = Some(invocation.name.clone());
                return None;
            }
            "ui.taper" => {
                self.setup_request = Some(invocation.name.clone());
                return None;
            }
            "ui.registration" => {
                self.setups.open_registration();
                return None;
            }
            "ui.camera_list" => {
                self.camera_windows.open_list();
                return None;
            }
            "ui.camera_capture" => Action::Camera(CameraAction::Capture(None)),
            "ui.device_profiles" => {
                self.devices.open_list();
                return None;
            }
            "ui.device_profile" => {
                self.edit_profile = true;
                return None;
            }
            "ui.test_speed_power" | "ui.test_interval" => {
                self.open_test = Some(if invocation.name == "ui.test_interval" {
                    TestKind::Interval
                } else {
                    TestKind::SpeedPower
                });
                return None;
            }
            "view.zoom_in" | "view.zoom_out" | "view.zoom_fit" => {
                self.canvas.zoom(&invocation.name);
                return None;
            }
            "ui.laser" => {
                self.show_machine_panel();
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
            ui.menu_button("Window", |ui| self.window_menu(ui));
        });
    }

    /// Window: Reset layout, then the toolbars and panels that can be
    /// shown or hidden (`docs/spec/style/menus.md`, "Window").
    fn window_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Reset layout").clicked() {
            self.dock.reset();
            self.panels = true;
        }
        ui.separator();
        let t = Tokens::of(ui.ctx());
        ui.label(egui::RichText::new("Toolbars").small().color(t.text_muted));
        if ui
            .add(egui::Button::new("Drawing tools").selected(self.dock.tools))
            .clicked()
        {
            self.dock.tools = !self.dock.tools;
        }
        if ui
            .add(egui::Button::new("Palette").selected(self.dock.palette))
            .clicked()
        {
            self.dock.palette = !self.dock.palette;
        }
        ui.separator();
        ui.label(egui::RichText::new("Panels").small().color(t.text_muted));
        for panel in DockPanel::ALL {
            let shown = self.dock.is_shown(panel);
            let row = ui
                .add(egui::Button::new(panel.name()).selected(shown))
                .on_hover_text(panel.about());
            if row.clicked() {
                if shown {
                    self.dock.hide(panel);
                } else {
                    self.show_panel(panel);
                }
            }
        }
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
        let output = has_output(view.project);
        let on = laser_panel::Enabled::of(laser, output);
        let not_ready = if laser.port.is_none() {
            OFFLINE
        } else if laser.busy.is_none() && !output {
            "Nothing to cut: no output layer has shapes."
        } else {
            "Wait until the machine is ready."
        };
        if laser.port.is_some() {
            let why = "Stop the job first.";
            self.menu_line(
                ui,
                "Disconnect",
                "device.disconnect",
                on.disconnect,
                why,
                actions,
            );
        } else if ui
            .add_enabled(self.laser.can_connect(), egui::Button::new("Connect"))
            .on_disabled_hover_text("No serial port is chosen; see the Machine panel.")
            .clicked()
        {
            actions.extend(self.laser.connect());
        }
        ui.separator();
        self.menu_line(ui, "Start", "ui.device_start", on.start, not_ready, actions);
        let (pause, _, hint) = laser_panel::pause_label(laser);
        let shortcut = self
            .keymap
            .chord_for("ui.device_pause")
            .map(Chord::label)
            .unwrap_or_default();
        let response = ui
            .add_enabled(
                on.pause_stop,
                egui::Button::new(pause).shortcut_text(shortcut),
            )
            .on_hover_text(hint)
            .on_disabled_hover_text(OFFLINE);
        if response.clicked() {
            actions.extend(self.resolve(Invocation::bare("ui.device_pause")));
        }
        self.menu_line(ui, "Stop", "device.stop", on.pause_stop, OFFLINE, actions);
        ui.separator();
        self.menu_line(ui, "Frame", "ui.device_frame", on.start, not_ready, actions);
        let outline = "ui.device_frame_outline";
        self.menu_line(ui, "Frame outline", outline, on.start, not_ready, actions);
        ui.separator();
        self.menu_line(ui, "Home", "device.home", on.home, OFFLINE, actions);
        if ui
            .add_enabled(on.go_origin, egui::Button::new("Go to user origin"))
            .on_disabled_hover_text("Needs a ready machine and a user origin.")
            .clicked()
            && let Some(origin) = laser.user_origin
        {
            actions.extend(self.laser.go_to(origin));
        }
        if ui
            .add_enabled(on.set_origin, egui::Button::new("Set user origin"))
            .on_disabled_hover_text(OFFLINE)
            .clicked()
        {
            actions.push(Action::Laser(LaserAction::SetOrigin));
        }
        if ui
            .add_enabled(on.clear_origin, egui::Button::new("Clear user origin"))
            .on_disabled_hover_text("No user origin is set.")
            .clicked()
        {
            actions.push(Action::Laser(LaserAction::ClearOrigin));
        }
        let why = "Only needed after an alarm.";
        self.menu_line(
            ui,
            "Clear alarm",
            "device.unlock",
            on.clear_alarm,
            why,
            actions,
        );
        ui.separator();
        ui.menu_button("Print registration", |ui| {
            self.menu_line(ui, "Guided setup…", "ui.registration", true, "", actions);
            let reg = view.project.registration();
            let ready = laser.port.is_some() && laser.position.is_some();
            let why = "Connect a ready machine first.";
            self.menu_line(
                ui,
                "Set target 1",
                "registration.set_target 1",
                ready,
                why,
                actions,
            );
            let one = !reg.targets.is_empty() && ready;
            self.menu_line(
                ui,
                "Set target 2",
                "registration.set_target 2",
                one,
                "Set target 1 first.",
                actions,
            );
            let two = reg.targets.len() == 2;
            let need = "Set both targets first.";
            self.menu_line(
                ui,
                "Fit job to targets",
                "registration.fit scaled",
                two,
                need,
                actions,
            );
            self.menu_line(
                ui,
                "Place job on targets (no scaling)",
                "registration.fit rigid",
                two,
                need,
                actions,
            );
            ui.separator();
            let any = !reg.targets.is_empty();
            self.menu_line(
                ui,
                "Clear registration",
                "registration.clear",
                any,
                "No targets are set.",
                actions,
            );
        });
        ui.separator();
        self.menu_line(ui, "Camera list…", "ui.camera_list", true, "", actions);
        self.menu_line(
            ui,
            "Capture camera image",
            "ui.camera_capture",
            true,
            "",
            actions,
        );
        ui.separator();
        self.menu_line(ui, "Rotary attachment…", "ui.rotary", true, "", actions);
        let rotary = view.project.rotary_on();
        self.menu_line(
            ui,
            "Taper correction…",
            "ui.taper",
            rotary,
            "Turn the rotary attachment on first.",
            actions,
        );
        ui.separator();
        for (label, line) in [
            ("Line interval test…", "ui.test_interval"),
            ("Speed and power test…", "ui.test_speed_power"),
            ("", ""),
            ("Device profiles…", "ui.device_profiles"),
            ("Edit device profile…", "ui.device_profile"),
        ] {
            if line.is_empty() {
                ui.separator();
            } else {
                self.menu_line(ui, label, line, true, "", actions);
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
        why: &str,
        actions: &mut Vec<Action>,
    ) {
        if self.menu_item(ui, label, line, enabled, why)
            && let Ok(invocation) = Invocation::parse(line)
        {
            actions.extend(self.resolve(invocation));
        }
    }

    fn drawing_tools(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let current = self.canvas.tool();
        if let Some(name) = self.tools.show(ui, current, &self.keymap) {
            actions.extend(self.resolve(Invocation::new("ui.tool", [name])));
        }
    }

    /// The command line: an input line, and above it the log while open.
    fn command_line(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let t = Tokens::of(ui.ctx());
        if self.dock.log_open {
            egui::Panel::bottom("command_input")
                .frame(egui::Frame::NONE)
                .show(ui, |ui| self.command_input(ui, actions));
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in &self.log {
                        ui.label(egui::RichText::new(line).monospace().color(t.text));
                    }
                });
        } else {
            self.command_input(ui, actions);
        }
    }

    fn command_input(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            let toggle = if self.dock.log_open {
                "Hide log"
            } else {
                "Show log"
            };
            let width = (ui.available_width() - 60.0).max(80.0);
            let input = ui.add(
                egui::TextEdit::singleline(&mut self.console)
                    .hint_text("Type a command, for example project.set_work_area 400 300")
                    .desired_width(width),
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
            if ui
                .button(toggle)
                .on_hover_text("Show or hide the command log")
                .clicked()
            {
                self.dock.log_open = !self.dock.log_open;
            }
        });
    }

    /// Pointer, selection, messages, work area and project, left to right.
    fn status_bar(&mut self, ui: &mut egui::Ui, view: &ShellView<'_>, live: bool) {
        let t = Tokens::of(ui.ctx());
        ui.horizontal(|ui| {
            if let Some(p) = self.pointer {
                let color = if live { t.text } else { t.text_muted };
                ui.label(
                    egui::RichText::new(format!("x {:.2}  y {:.2} mm", p.x, p.y))
                        .monospace()
                        .color(color),
                )
                .on_hover_text("Pointer position on the bed");
                ui.separator();
            }
            let selection = view.project.selection();
            if let Ok(Some(b)) = view.project.bounds_of(&selection) {
                ui.label(format!(
                    "{} selected, {:.2} × {:.2} mm",
                    selection.len(),
                    b.width(),
                    b.height()
                ))
                .on_hover_text("Size of the selection");
                ui.separator();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let file = view
                    .file
                    .map_or_else(|| "not saved".to_owned(), |p| p.display().to_string());
                let marker = if view.modified { " •" } else { "" };
                ui.label(format!("{}{marker} — {file}", view.project.name()));
                ui.separator();
                let mut setups = Vec::new();
                if view.project.rotary_on() {
                    setups.push("Rotary");
                }
                if view.project.registration_transform().is_some() {
                    setups.push("Print registration");
                }
                if !setups.is_empty() {
                    ui.label(
                        egui::RichText::new(setups.join(", "))
                            .color(t.text)
                            .background_color(t.success_soft),
                    )
                    .on_hover_text("Active setups: they change how jobs run");
                    ui.separator();
                }
                let area = view.project.work_area();
                let origin = area.origin.as_str().replace('-', " ");
                ui.label(
                    egui::RichText::new(format!(
                        "Work area {} × {} mm, origin {origin}",
                        area.width_mm, area.height_mm
                    ))
                    .color(t.text_muted),
                );
                ui.separator();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    self.message(ui, view.laser);
                });
            });
        });
    }

    /// The status bar's Messages field: an alarm, else the last log entry.
    fn message(&mut self, ui: &mut egui::Ui, laser: &LaserView) {
        let (kind, text) = if let Some(alarm) = &laser.alarm {
            (Band::Error, alarm.clone())
        } else if let Some(last) = self.log.last() {
            let text = last.lines().last().unwrap_or_default().to_owned();
            let error = text.starts_with("error:") || text.starts_with("Cannot ");
            (if error { Band::Error } else { Band::Hint }, text)
        } else {
            return;
        };
        let t = Tokens::of(ui.ctx());
        let fill = match kind {
            Band::Error => t.danger_soft,
            _ => t.info_soft,
        };
        let shown = egui::Label::new(
            egui::RichText::new(&text)
                .color(t.text)
                .background_color(fill),
        )
        .truncate()
        .sense(egui::Sense::click());
        if ui.add(shown).on_hover_text(&text).clicked() {
            self.dock.log_open = true;
        }
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
            PromptKind::Run { title, what, .. } => (*title, *what),
        };
        let mut open = true;
        let mut done = None;
        let mut cancelled = false;
        egui::Window::new(title)
            .collapsible(false)
            .open(&mut open)
            .bounded(ui.ctx())
            .show(ui.ctx(), |ui| {
                ui.label(what);
                let field = ui.add(
                    egui::TextEdit::singleline(text).desired_width(ui.available_width().max(360.0)),
                );
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let blank = text.trim().is_empty();
                let (ok, cancel) = crate::dialogs::buttons(ui, "OK", !blank, "Type a path first.");
                if (ok || enter) && !blank {
                    done = Some(PathBuf::from(text.trim()));
                }
                if cancel || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancelled = true;
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
                PromptKind::Run { command, at, .. } => {
                    let mut command = command.clone();
                    let at = (*at).min(command.args.len());
                    command.args.insert(at, path.display().to_string());
                    Action::Run(command)
                }
            });
            self.prompt = None;
        } else if !open || cancelled {
            self.prompt = None;
        }
    }
}

/// Whether an output layer has shapes: a job would cut something.
fn has_output(project: &Project) -> bool {
    project
        .used_layers()
        .into_iter()
        .any(|c| project.layer(c).is_ok_and(|l| l.output))
}
