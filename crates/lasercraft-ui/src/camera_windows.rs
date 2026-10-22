//! Camera list, Add camera, Camera setup, and the lens calibration and
//! camera alignment pages that run inside the last two
//! (`docs/spec/camera.md`, "In the app"; `docs/spec/style/
//! later-windows.md`, "Camera"). Every change is a `camera.*` command;
//! pictures and fitting are the app's ([`CameraAction`]).

use lasercraft_camera::lens::Model;
use lasercraft_camera::marks::Pattern;
use lasercraft_commands::Invocation;
use lasercraft_geom::Point;
use lasercraft_project::Project;
use lasercraft_workshop::{Camera, Source, Workshop};

use crate::camera_panel::{CameraAction, CameraView, PictureTextures, picture, picture_settings};
use crate::keymap::Keymap;
use crate::laser_panel::{self, Enabled, LaserView};
use crate::shell::{Action, PromptKind};
use crate::theme::{Tokens, size, weight};
use crate::widgets::Bounded;
use crate::widgets::{self, Band, Fill};

/// Pictures the lens correction wants.
const LENS_PICTURES: usize = 5;
/// Where the sheet goes in each lens picture, as fractions of the picture.
const PLACES: [(f32, f32, &str); LENS_PICTURES] = [
    (0.0, 0.0, "top-left"),
    (0.5, 0.0, "top-right"),
    (0.5, 0.5, "bottom-right"),
    (0.0, 0.5, "bottom-left"),
    (0.25, 0.25, "center"),
];

/// The lens calibration or camera alignment pages.
#[derive(Debug, Clone, PartialEq)]
enum Flow {
    Lens {
        page: usize,
        model: Model,
    },
    Align {
        page: usize,
        manual: bool,
        pattern: Pattern,
        speed: f64,
        power: f64,
        clicked: Vec<Point>,
        zoom: f32,
    },
}

/// The window the flows run in.
#[derive(Debug, Clone, PartialEq)]
enum Host {
    /// Add camera, with its page (0 kind, 1 source, 2 mount, 3 lens choice,
    /// 4 done) and the camera made on page 0.
    Add {
        page: usize,
        camera: Option<String>,
        name: String,
    },
    /// Camera setup of one camera, with the name being edited.
    Setup { camera: String, name: String },
}

#[derive(Default)]
pub(crate) struct CameraWindows {
    list: bool,
    selected: Option<String>,
    remove: Option<String>,
    host: Option<Host>,
    flow: Option<Flow>,
    textures: PictureTextures,
    /// A path the windows want asked for; the shell takes it.
    pub(crate) prompt: Option<PromptKind>,
}

fn run(name: &str, args: impl IntoIterator<Item = impl Into<String>>) -> Action {
    Action::Run(Invocation::new(name, args.into_iter().map(Into::into)))
}

fn card(ui: &mut egui::Ui, chosen: bool, enabled: bool, title: &str, about: &str) -> bool {
    let t = Tokens::of(ui.ctx());
    let (fill, stroke) = if chosen {
        (t.accent_soft, egui::Stroke::new(weight::NORMAL, t.accent))
    } else {
        (t.field_alt, egui::Stroke::new(weight::THIN, t.border))
    };
    let mut dot = false;
    let response = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(crate::theme::ROUNDED)
        .inner_margin(size::GAP)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add_enabled_ui(enabled, |ui| {
                ui.horizontal(|ui| {
                    dot = ui.radio(chosen, "").clicked();
                    ui.vertical(|ui| {
                        ui.label(title);
                        ui.label(egui::RichText::new(about).small().color(t.text_muted));
                    });
                });
            });
        })
        .response
        .interact(egui::Sense::click());
    enabled && (response.clicked() || dot)
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.heading(text);
    });
    ui.add_space(size::GAP);
}

/// The cameras of the project's device profile.
fn cameras<'a>(project: &Project, workshop: &'a Workshop) -> &'a [Camera] {
    project
        .device()
        .and_then(|d| workshop.cameras(d).ok())
        .unwrap_or_default()
}

impl CameraWindows {
    /// Whether a window wants live pictures.
    pub(crate) fn live(&self) -> bool {
        self.host.is_some()
    }

    pub(crate) fn open_list(&mut self) {
        self.list = true;
    }

    pub(crate) fn open_add(&mut self) {
        if self.host.is_none() {
            self.host = Some(Host::Add {
                page: 0,
                camera: None,
                name: String::new(),
            });
            self.flow = None;
        }
    }

    pub(crate) fn open_setup(&mut self, camera: &str) {
        self.host = Some(Host::Setup {
            camera: camera.to_owned(),
            name: camera.to_owned(),
        });
        self.flow = None;
    }

    pub(crate) fn open_lens(&mut self, camera: &str) {
        self.open_setup(camera);
        self.flow = Some(Flow::Lens {
            page: 0,
            model: Model::Standard,
        });
    }

    pub(crate) fn open_align(&mut self, project: &Project, camera: &str) {
        self.open_setup(camera);
        let area = project.work_area();
        self.flow = Some(Flow::Align {
            page: 0,
            manual: false,
            pattern: Pattern::default_for(area.width_mm, area.height_mm),
            speed: 100.0,
            power: 20.0,
            clicked: Vec::new(),
            zoom: 1.0,
        });
    }

    pub(crate) fn import(&mut self) {
        self.prompt = Some(PromptKind::Run {
            title: "Import camera",
            what: "Path to a .lccam file:",
            command: Invocation::bare("camera.import"),
            at: 0,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        laser: &LaserView,
        keymap: &Keymap,
        actions: &mut Vec<Action>,
    ) {
        self.list_window(ctx, project, workshop, actions);
        self.remove_window(ctx, actions);
        self.host_window(ctx, project, workshop, view, laser, keymap, actions);
    }

    fn list_window(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        workshop: &Workshop,
        actions: &mut Vec<Action>,
    ) {
        if !self.list {
            return;
        }
        let t = Tokens::of(ctx);
        let cams = cameras(project, workshop);
        let mut close = false;
        egui::Window::new("Camera list")
            .collapsible(false)
            .resizable(false)
            .bounded(ctx)
            .show(ctx, |ui| {
                ui.set_width(380.0);
                let device = project.device().unwrap_or("no device profile");
                ui.label(egui::RichText::new(format!("Cameras for {device}")).strong());
                egui::Frame::new()
                    .fill(t.field)
                    .stroke(egui::Stroke::new(weight::THIN, t.border))
                    .inner_margin(size::GAP_TIGHT)
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(ui.available_width(), 120.0));
                        if cams.is_empty() {
                            ui.label(
                                egui::RichText::new(
                                    "No camera set up. Use Add camera… to add one.",
                                )
                                .color(t.text_muted),
                            );
                        }
                        for c in cams {
                            let on = self.selected.as_deref() == Some(c.name.as_str());
                            let row = ui.selectable_label(on, &c.name);
                            ui.label(
                                egui::RichText::new(c.source.summary())
                                    .small()
                                    .color(t.text_muted),
                            );
                            if row.clicked() {
                                self.selected = Some(c.name.clone());
                            }
                            if row.double_clicked() {
                                self.host = Some(Host::Setup {
                                    camera: c.name.clone(),
                                    name: c.name.clone(),
                                });
                            }
                        }
                    });
                if let Some(c) = self
                    .selected
                    .as_ref()
                    .and_then(|n| cams.iter().find(|c| &c.name == n))
                {
                    ui.label(egui::RichText::new(c.summary()).small().color(t.text_muted));
                }
                let min = egui::vec2(110.0, size::ROW);
                ui.horizontal(|ui| {
                    let no_profile = project.device().is_none();
                    if ui
                        .add_enabled(!no_profile, egui::Button::new("Add camera…").min_size(min))
                        .on_disabled_hover_text("Choose a device profile first.")
                        .clicked()
                    {
                        self.open_add();
                    }
                    let some = self
                        .selected
                        .as_ref()
                        .is_some_and(|n| cams.iter().any(|c| &c.name == n));
                    if ui
                        .add_enabled(some, egui::Button::new("Edit…").min_size(min))
                        .clicked()
                        && let Some(n) = self.selected.clone()
                    {
                        self.open_setup(&n);
                    }
                    if ui
                        .add_enabled(some, egui::Button::new("Remove").min_size(min))
                        .clicked()
                    {
                        self.remove = self.selected.clone();
                    }
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
        let _ = actions;
        if close {
            self.list = false;
        }
    }

    fn remove_window(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        let Some(name) = self.remove.clone() else {
            return;
        };
        let mut done = false;
        egui::Window::new("Remove camera?").collapsible(false).resizable(false).bounded(ctx).show(ctx, |ui| {
            ui.set_max_width(340.0);
            ui.label(format!(
                "The camera \"{name}\", its lens correction and its alignment are removed from this device profile."
            ));
            ui.add_space(size::GAP);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let min = egui::vec2(96.0, size::ROW);
                if ui.add(egui::Button::new("Cancel").min_size(min)).clicked() {
                    done = true;
                }
                if widgets::filled(ui, "Remove camera", Fill::Danger, min).clicked() {
                    actions.push(run("camera.remove", [name.clone()]));
                    done = true;
                }
            });
        });
        if done {
            self.remove = None;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn host_window(
        &mut self,
        ctx: &egui::Context,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        laser: &LaserView,
        keymap: &Keymap,
        actions: &mut Vec<Action>,
    ) {
        let Some(host) = self.host.clone() else {
            return;
        };
        let title = match &host {
            Host::Add { .. } => "Add camera".to_owned(),
            Host::Setup { camera, .. } => format!("Camera setup: {camera}"),
        };
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::unique("camera-host"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([760.0, 560.0])
            .bounded(ctx)
            .show(ctx, |ui| {
                if self.flow.is_some() {
                    let camera = match &host {
                        Host::Add { camera, .. } => camera.clone().unwrap_or_default(),
                        Host::Setup { camera, .. } => camera.clone(),
                    };
                    self.flow_ui(ui, project, workshop, view, laser, keymap, &camera, actions);
                } else {
                    match host {
                        Host::Add { .. } => self.add_ui(ui, project, workshop, view, actions),
                        Host::Setup { .. } => self.setup_ui(ui, project, workshop, view, actions),
                    }
                }
            });
        if !open {
            // Closing Add camera before its last page drops the camera.
            if let Some(Host::Add {
                camera: Some(c),
                page,
                ..
            }) = &self.host
                && *page < 4
            {
                actions.push(run("camera.remove", [c.clone()]));
            }
            self.leave_flow(actions);
            self.host = None;
        }
    }

    fn leave_flow(&mut self, actions: &mut Vec<Action>) {
        if matches!(self.flow, Some(Flow::Lens { .. })) {
            actions.push(Action::Camera(CameraAction::SimulatedSheet(None)));
            actions.push(Action::Camera(CameraAction::ClearPictures));
        }
        self.flow = None;
    }

    fn footer(
        ui: &mut egui::Ui,
        back: bool,
        next: Option<(&str, bool, &str)>,
    ) -> (bool, bool, bool) {
        ui.add_space(size::GAP);
        ui.separator();
        let (mut b, mut n, mut c) = (false, false, false);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let min = egui::vec2(80.0, size::ROW);
            c = ui.add(egui::Button::new("Cancel").min_size(min)).clicked();
            if let Some((label, enabled, why)) = next {
                n = ui
                    .add_enabled_ui(enabled, |ui| widgets::filled(ui, label, Fill::Primary, min))
                    .inner
                    .on_disabled_hover_text(why)
                    .clicked();
            }
            b = ui
                .add_enabled(back, egui::Button::new("Back").min_size(min))
                .clicked();
        });
        (b, n, c)
    }

    fn live_picture(
        &mut self,
        ui: &mut egui::Ui,
        view: &CameraView,
        camera: &str,
        width: f32,
    ) -> Option<(egui::Response, egui::Rect, f32)> {
        let t = Tokens::of(ui.ctx());
        let live = view.live(camera)?.clone();
        match self.textures.get(ui.ctx(), &live) {
            Some(texture) => Some(picture(ui, &texture, width, None)),
            None => {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(width, width * 0.75), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, t.field);
                let text = match &live.status {
                    crate::camera_panel::CameraStatus::Failed(e) => {
                        format!("No picture source: {e}")
                    }
                    _ => "Connecting…".to_owned(),
                };
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(14.0),
                    t.text_muted,
                );
                None
            }
        }
    }

    fn add_ui(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        actions: &mut Vec<Action>,
    ) {
        let Some(Host::Add { page, camera, name }) = self.host.clone() else {
            return;
        };
        let cams = cameras(project, workshop);
        let cam = camera
            .as_ref()
            .and_then(|n| cams.iter().find(|c| &c.name == n));
        let mut set_page = None;
        match page {
            0 => {
                heading(ui, "What kind of camera?");
                let kinds = [
                    ("usb", "USB camera", "A camera on a USB cable"),
                    (
                        "network",
                        "Network camera",
                        "A camera that serves pictures over the network",
                    ),
                    (
                        "simulated",
                        "Simulated camera",
                        "LaserCraft's own camera, for trying things out",
                    ),
                ];
                for (kind, title, about) in kinds {
                    if card(ui, false, project.device().is_some(), title, about) {
                        let mut n = cams.len() + 1;
                        let base = title.replace(" camera", "");
                        let mut new = format!("{base} camera {n}");
                        while cams.iter().any(|c| c.name.eq_ignore_ascii_case(&new)) {
                            n += 1;
                            new = format!("{base} camera {n}");
                        }
                        let mut args = vec![new.clone(), kind.to_owned()];
                        if kind == "network" {
                            args.push("url=http://camera.local/picture.jpg".to_owned());
                        }
                        actions.push(Action::Run(Invocation::new("camera.add", args)));
                        actions.push(Action::Camera(CameraAction::RefreshUsb));
                        self.host = Some(Host::Add {
                            page: 1,
                            camera: Some(new.clone()),
                            name: new,
                        });
                        return;
                    }
                    ui.add_space(size::GAP_TIGHT);
                }
                if project.device().is_none() {
                    widgets::band(
                        ui,
                        Band::Warning,
                        "Cameras belong to a device profile. Choose one in the Machine panel first.",
                    );
                }
                let (_, _, cancel) = Self::footer(ui, false, None);
                if cancel {
                    self.host = None;
                }
                return;
            }
            1 => {
                heading(ui, "Choose the picture source");
                let ready = view.live(&name).is_some_and(|l| l.picture.is_some());
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(300.0);
                        if let Some(c) = cam {
                            source_box(ui, c, view, actions);
                        }
                    });
                    self.live_picture(ui, view, &name, ui.available_width().min(420.0));
                });
                let (back, next, cancel) = Self::footer(
                    ui,
                    false,
                    Some(("Next", ready, "Waiting for pictures from the source.")),
                );
                if next {
                    set_page = Some(2);
                }
                let _ = back;
                if cancel {
                    actions.push(run("camera.remove", [name.clone()]));
                    self.host = None;
                    return;
                }
            }
            2 => {
                heading(ui, "Where is the camera mounted?");
                card(
                    ui,
                    true,
                    true,
                    "Above the bed",
                    "Fixed over the work area; sees the whole bed",
                );
                ui.add_space(size::GAP_TIGHT);
                card(
                    ui,
                    false,
                    false,
                    "On the laser head (later)",
                    "Moves with the head; sees a small area",
                );
                let (back, next, cancel) = Self::footer(ui, true, Some(("Next", true, "")));
                if back {
                    set_page = Some(1);
                }
                if next {
                    set_page = Some(3);
                }
                if cancel {
                    actions.push(run("camera.remove", [name.clone()]));
                    self.host = None;
                    return;
                }
            }
            3 => {
                heading(ui, "Lens correction");
                if card(
                    ui,
                    false,
                    true,
                    "Calibrate now",
                    "Take five pictures of the calibration sheet",
                ) {
                    self.flow = Some(Flow::Lens {
                        page: 0,
                        model: Model::Standard,
                    });
                }
                ui.add_space(size::GAP_TIGHT);
                if card(
                    ui,
                    false,
                    true,
                    "Load from file…",
                    "Use the lens correction of an exported camera",
                ) {
                    self.prompt = Some(PromptKind::Run {
                        title: "Load lens correction",
                        what: "Path to a .lccam file:",
                        command: Invocation::new("camera.set_lens", [name.clone()]),
                        at: 1,
                    });
                }
                ui.add_space(size::GAP_TIGHT);
                if card(
                    ui,
                    false,
                    true,
                    "No lens correction",
                    "The picture is used as it is (not recommended)",
                ) {
                    actions.push(run("camera.set_lens", [name.clone(), "none".to_owned()]));
                    self.open_align_inside(project);
                }
                if cam.is_some_and(|c| c.lens.is_some()) {
                    widgets::band(
                        ui,
                        Band::Success,
                        "The lens is corrected. Next aligns the camera with the bed.",
                    );
                }
                let lens = cam.is_some_and(|c| c.lens.is_some());
                let (back, next, cancel) = Self::footer(
                    ui,
                    true,
                    Some((
                        "Next",
                        lens,
                        "Correct the lens, load a correction or choose No lens correction.",
                    )),
                );
                if back {
                    set_page = Some(2);
                }
                if next {
                    self.open_align_inside(project);
                }
                if cancel {
                    actions.push(run("camera.remove", [name.clone()]));
                    self.host = None;
                    return;
                }
            }
            _ => {
                heading(ui, "Camera added");
                ui.horizontal(|ui| {
                    widgets::mark(ui, Band::Success);
                    ui.label("Name");
                    let mut text = name.clone();
                    if ui.text_edit_singleline(&mut text).changed() {
                        self.host = Some(Host::Add {
                            page,
                            camera: camera.clone(),
                            name: text,
                        });
                    }
                });
                if let Some(c) = cam {
                    ui.label(
                        egui::RichText::new(c.summary())
                            .small()
                            .color(Tokens::of(ui.ctx()).text_muted),
                    );
                }
                let (_, finish, cancel) = Self::footer(
                    ui,
                    false,
                    Some(("Finish", !name.trim().is_empty(), "Type a name.")),
                );
                if finish {
                    if let Some(old) = &camera
                        && old.trim() != name.trim()
                    {
                        actions.push(run("camera.rename", [old.clone(), name.trim().to_owned()]));
                    }
                    self.host = None;
                }
                if cancel {
                    self.host = None;
                }
                return;
            }
        }
        if let Some(p) = set_page {
            self.host = Some(Host::Add {
                page: p,
                camera,
                name,
            });
        }
    }

    fn open_align_inside(&mut self, project: &Project) {
        let area = project.work_area();
        self.flow = Some(Flow::Align {
            page: 0,
            manual: false,
            pattern: Pattern::default_for(area.width_mm, area.height_mm),
            speed: 100.0,
            power: 20.0,
            clicked: Vec::new(),
            zoom: 1.0,
        });
    }

    fn setup_ui(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        actions: &mut Vec<Action>,
    ) {
        let Some(Host::Setup { camera, name }) = self.host.clone() else {
            return;
        };
        let t = Tokens::of(ui.ctx());
        let Some(cam) = cameras(project, workshop)
            .iter()
            .find(|c| c.name == camera)
            .cloned()
        else {
            ui.label("This camera was removed.");
            if ui.button("Close").clicked() {
                self.host = None;
            }
            return;
        };
        let mut new_name = name.clone();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                group(ui, "Camera", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut new_name);
                    });
                    ui.label(egui::RichText::new(format!("Kind: {} camera", cam.source.kind())).color(t.text_muted));
                    ui.label(egui::RichText::new("Mounted: above the bed").color(t.text_muted));
                    if ui.button("Export…").clicked() {
                        self.prompt = Some(PromptKind::Run {
                            title: "Export camera",
                            what: "Path to the .lccam file to write:",
                            command: Invocation::new("camera.export", [camera.clone()]),
                            at: 1,
                        });
                    }
                });
                group(ui, "Source", |ui| source_box(ui, &cam, view, actions));
                group(ui, "Lens correction", |ui| {
                    match &cam.lens {
                        Some(l) => {
                            ui.horizontal(|ui| {
                                widgets::mark(ui, Band::Success);
                                ui.label(format!("{} model, fit {:.0}%", l.model.title(), l.score()));
                            });
                        }
                        None => {
                            ui.horizontal(|ui| {
                                widgets::mark(ui, Band::Warning);
                                ui.label("Not calibrated");
                            });
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Calibrate lens…").clicked() {
                            self.flow = Some(Flow::Lens { page: 0, model: Model::Standard });
                        }
                        if ui.button("Import…").clicked() {
                            self.prompt = Some(PromptKind::Run {
                                title: "Import lens correction",
                                what: "Path to a .lccam file:",
                                command: Invocation::new("camera.set_lens", [camera.clone()]),
                                at: 1,
                            });
                        }
                        if ui.add_enabled(cam.lens.is_some(), egui::Button::new("Remove")).clicked() {
                            actions.push(run("camera.set_lens", [camera.clone(), "none".to_owned()]));
                        }
                    });
                });
                group(ui, "Bed alignment", |ui| {
                    if cam.alignment_outdated() {
                        widgets::band(ui, Band::Warning, "This alignment was made with another lens correction. Align to bed again.");
                    }
                    match cam.alignment() {
                        Some(a) => {
                            egui::ComboBox::from_id_salt("alignments")
                                .selected_text(&a.name)
                                .show_ui(ui, |ui| {
                                    for other in &cam.alignments {
                                        if ui.selectable_label(other.name == a.name, &other.name).clicked() && other.name != a.name {
                                            actions.push(run("camera.use_alignment", [camera.clone(), other.name.clone()]));
                                        }
                                    }
                                });
                        }
                        None => {
                            ui.horizontal(|ui| {
                                widgets::mark(ui, Band::Warning);
                                ui.label("Not aligned");
                            });
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .button("Align to bed…")
                            .clicked()
                        {
                            self.open_align_inside(project);
                        }
                        if ui.add_enabled(cam.alignment().is_some(), egui::Button::new("Remove")).clicked()
                            && let Some(a) = cam.alignment()
                        {
                            actions.push(run("camera.remove_alignment", [camera.clone(), a.name.clone()]));
                        }
                    });
                });
            });
            ui.vertical(|ui| {
                ui.label("Live view");
                let live = view.live(&camera).cloned().unwrap_or_default();
                let width = ui.available_width().min(480.0);
                if let Some(texture) = self.textures.get(ui.ctx(), &live) {
                    picture(ui, &texture, width, Some((&cam, project)));
                    if cam.alignment().is_some() {
                        ui.label(egui::RichText::new("grid 10 mm").small().color(t.text_muted));
                    }
                } else {
                    self.live_picture(ui, view, &camera, width);
                }
                group(ui, "Picture settings", |ui| {
                    picture_settings(ui, &cam, &live, actions);
                });
            });
        });
        self.host = Some(Host::Setup {
            camera: camera.clone(),
            name: new_name.clone(),
        });
        let (ok, cancel) =
            crate::dialogs::buttons(ui, "OK", !new_name.trim().is_empty(), "Type a name.");
        if ok {
            if new_name.trim() != camera {
                actions.push(run(
                    "camera.rename",
                    [camera.clone(), new_name.trim().to_owned()],
                ));
            }
            self.host = None;
        }
        if cancel {
            self.host = None;
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn flow_ui(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        laser: &LaserView,
        keymap: &Keymap,
        camera: &str,
        actions: &mut Vec<Action>,
    ) {
        let t = Tokens::of(ui.ctx());
        let cam = cameras(project, workshop)
            .iter()
            .find(|c| c.name == camera)
            .cloned();
        let simulated = cam.as_ref().is_some_and(|c| c.source == Source::Simulated);
        let Some(flow) = self.flow.clone() else {
            return;
        };
        match flow {
            Flow::Lens { page, model } => {
                let mut next_flow = Some(Flow::Lens { page, model });
                match page {
                    0 => {
                        heading(ui, "Correct the lens");
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("About ten minutes").color(t.text_muted));
                        });
                        ui.horizontal_top(|ui| {
                            for (n, title, about) in [
                                (
                                    1,
                                    "Print the sheet",
                                    "Print LaserCraft's calibration sheet and keep it flat",
                                ),
                                (
                                    2,
                                    "Take pictures",
                                    "Five pictures: the four corners and the center",
                                ),
                                (
                                    3,
                                    "Pick a model",
                                    "Choose the model whose pictures look straight",
                                ),
                            ] {
                                egui::Frame::new()
                                    .fill(t.field_alt)
                                    .corner_radius(crate::theme::ROUNDED)
                                    .inner_margin(size::GAP)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.set_width(200.0);
                                            ui.label(
                                                egui::RichText::new(n.to_string())
                                                    .color(t.text_muted),
                                            );
                                            ui.label(title);
                                            ui.label(
                                                egui::RichText::new(about)
                                                    .small()
                                                    .color(t.text_muted),
                                            );
                                        });
                                    });
                            }
                        });
                        if ui.button("Save the calibration sheet…").clicked() {
                            self.prompt = Some(PromptKind::Run {
                                title: "Save the calibration sheet",
                                what: "Path to the .svg file to write:",
                                command: Invocation::bare("camera.pattern"),
                                at: 0,
                            });
                        }
                        let (back, next, cancel) =
                            Self::footer(ui, false, Some(("Next", true, "")));
                        let _ = back;
                        if next {
                            actions.push(Action::Camera(CameraAction::ClearPictures));
                            next_flow = Some(Flow::Lens { page: 1, model });
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    1 => {
                        heading(ui, "Take pictures of the sheet");
                        let good = view.captures.iter().filter(|c| c.error.is_none()).count();
                        if simulated {
                            actions.push(Action::Camera(CameraAction::SimulatedSheet(Some(
                                good.min(LENS_PICTURES - 1),
                            ))));
                        }
                        ui.horizontal_top(|ui| {
                            let width = (ui.available_width() - 240.0).max(240.0);
                            if let Some((_, rect, _)) = self.live_picture(ui, view, camera, width)
                                && good < LENS_PICTURES
                            {
                                let (fx, fy, _) = PLACES[good];
                                let area = egui::Rect::from_min_size(
                                    rect.min + egui::vec2(rect.width() * fx, rect.height() * fy),
                                    egui::vec2(rect.width() * 0.5, rect.height() * 0.5),
                                );
                                ui.painter().rect_stroke(
                                    area,
                                    0.0,
                                    egui::Stroke::new(weight::HEAVY, t.marquee),
                                    egui::StrokeKind::Middle,
                                );
                            }
                            ui.vertical(|ui| {
                                ui.label(format!("Pictures taken: {good} of {LENS_PICTURES}"));
                                let line = PLACES.get(good).map_or_else(
                                    || {
                                        "All five pictures are taken. More may be added anywhere."
                                            .to_owned()
                                    },
                                    |p| {
                                        format!(
                                            "Hold the sheet in the {} part of the picture.",
                                            p.2
                                        )
                                    },
                                );
                                ui.label(egui::RichText::new(line).small());
                                if ui
                                    .add(
                                        egui::Button::new("Capture")
                                            .min_size(egui::vec2(120.0, size::ROW)),
                                    )
                                    .clicked()
                                {
                                    actions.push(Action::Camera(CameraAction::KeepPicture(
                                        camera.to_owned(),
                                    )));
                                }
                                ui.add(
                                    egui::ProgressBar::new(
                                        good.min(LENS_PICTURES) as f32 / LENS_PICTURES as f32,
                                    )
                                    .show_percentage(),
                                );
                                for (i, c) in view.captures.iter().enumerate() {
                                    let file = c.path.rsplit(['/', '\\']).next().unwrap_or(&c.path);
                                    let text = match &c.error {
                                        None => format!("{} · {} dots", file, c.dots.len()),
                                        Some(e) => format!("{file} · {e}"),
                                    };
                                    ui.horizontal(|ui| {
                                        widgets::mark(
                                            ui,
                                            if c.error.is_none() {
                                                Band::Success
                                            } else {
                                                Band::Error
                                            },
                                        );
                                        ui.label(egui::RichText::new(text).small());
                                        if ui.small_button("Discard").clicked() {
                                            actions
                                                .push(Action::Camera(CameraAction::DropPicture(i)));
                                        }
                                    });
                                }
                            });
                        });
                        let (back, next, cancel) = Self::footer(
                            ui,
                            true,
                            Some((
                                "Next",
                                good >= LENS_PICTURES,
                                "Take five pictures in which the sheet is found.",
                            )),
                        );
                        if back {
                            next_flow = Some(Flow::Lens { page: 0, model });
                        }
                        if next {
                            actions.push(Action::Camera(CameraAction::FitLens(camera.to_owned())));
                            next_flow = Some(Flow::Lens { page: 2, model });
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    2 => {
                        heading(ui, "Pick a lens model");
                        let mut chosen = model;
                        for (m, fit) in Model::ALL.iter().zip(&view.fits) {
                            let about = match fit {
                                Ok(l) => format!("Fit {:.0}% ({:.2} pixels off)", l.score(), l.rms),
                                Err(e) => e.clone(),
                            };
                            if card(ui, chosen == *m, fit.is_ok(), m.title(), &about) {
                                chosen = *m;
                            }
                            ui.add_space(size::GAP_TIGHT);
                        }
                        if view.fits.is_empty() {
                            ui.label("Fitting…");
                        }
                        egui::Frame::new().fill(t.info_soft).inner_margin(size::GAP).show(ui, |ui| {
                            ui.label(egui::RichText::new("Higher is better, but compare the pictures too. Fish-eye is only for fish-eye lenses.").small());
                        });
                        let fit = Model::ALL
                            .iter()
                            .position(|m| *m == chosen)
                            .and_then(|i| view.fits.get(i))
                            .and_then(|f| f.as_ref().ok());
                        let (back, next, cancel) = Self::footer(
                            ui,
                            true,
                            Some(("Next", fit.is_some(), "Choose a model that fitted.")),
                        );
                        next_flow = Some(Flow::Lens {
                            page: 2,
                            model: chosen,
                        });
                        if back {
                            next_flow = Some(Flow::Lens {
                                page: 1,
                                model: chosen,
                            });
                        }
                        if next && let Some(lens) = fit {
                            let json = serde_json::to_string(lens).unwrap_or_default();
                            actions.push(run("camera.set_lens", [camera.to_owned(), json]));
                            next_flow = Some(Flow::Lens {
                                page: 3,
                                model: chosen,
                            });
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    _ => {
                        heading(ui, "Lens corrected");
                        ui.vertical_centered(|ui| {
                            widgets::mark(ui, Band::Success);
                            ui.label(format!(
                                "{} lens correction stored for {camera}.",
                                model.title()
                            ));
                        });
                        let (_, finish, _) = Self::footer(ui, false, Some(("Finish", true, "")));
                        if finish {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                }
                self.flow = next_flow;
            }
            Flow::Align {
                page,
                manual,
                mut pattern,
                mut speed,
                mut power,
                mut clicked,
                mut zoom,
            } => {
                let busy = laser.busy.is_some();
                let area = *project.work_area();
                let mut go = page;
                let mut manual2 = manual;
                match page {
                    0 => {
                        heading(ui, "How should the marks be found?");
                        if card(
                            ui,
                            !manual,
                            true,
                            "Automatically",
                            "LaserCraft finds the burned marks in the picture",
                        ) {
                            manual2 = false;
                        }
                        ui.add_space(size::GAP_TIGHT);
                        if card(
                            ui,
                            manual,
                            true,
                            "By clicking",
                            "You click the center of each mark",
                        ) {
                            manual2 = true;
                        }
                        let (_, next, cancel) = Self::footer(ui, false, Some(("Next", true, "")));
                        if next {
                            go = 1;
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    1 => {
                        heading(ui, "Line up the camera with the bed");
                        let steps: &[&str] = if manual {
                            &[
                                "Place the pattern",
                                "Burn the pattern",
                                "Click the marks",
                                "Check the result",
                            ]
                        } else {
                            &["Place the pattern", "Burn the pattern", "Check the result"]
                        };
                        ui.horizontal(|ui| {
                            for (i, s) in steps.iter().enumerate() {
                                egui::Frame::new()
                                    .fill(t.field_alt)
                                    .inner_margin(size::GAP)
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new((i + 1).to_string())
                                                .color(t.text_muted),
                                        );
                                        ui.label(*s);
                                    });
                            }
                        });
                        let (back, next, cancel) = Self::footer(ui, true, Some(("Next", true, "")));
                        if back {
                            go = 0;
                        }
                        if next {
                            go = 2;
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    2 => {
                        heading(ui, "Place the pattern");
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(220.0);
                                ui.label(egui::RichText::new("Use a flat scrap sheet that covers most of the bed. The marks must be dark and crisp.").small());
                                egui::Grid::new("pattern-size").num_columns(2).show(ui, |ui| {
                                    ui.label("Mark size (mm)");
                                    ui.add(egui::DragValue::new(&mut pattern.mark).range(4.0..=50.0));
                                    ui.end_row();
                                    ui.label("Pattern width (mm)");
                                    ui.add(egui::DragValue::new(&mut pattern.width).range(10.0..=area.width_mm));
                                    ui.end_row();
                                    ui.label("Pattern height (mm)");
                                    ui.add(egui::DragValue::new(&mut pattern.height).range(10.0..=area.height_mm));
                                    ui.end_row();
                                });
                                ui.horizontal(|ui| {
                                    if ui.button("Default size").clicked() {
                                        pattern = Pattern {
                                            dx: pattern.dx,
                                            dy: pattern.dy,
                                            ..Pattern::default_for(area.width_mm, area.height_mm)
                                        };
                                    }
                                    if ui
                                        .add_enabled(pattern.dx != 0.0 || pattern.dy != 0.0, egui::Button::new("Center"))
                                        .clicked()
                                    {
                                        pattern.dx = 0.0;
                                        pattern.dy = 0.0;
                                    }
                                });
                                ui.label(egui::RichText::new("Drag the pattern on the bed to move it.").small().color(t.text_muted));
                            });
                            let (x, y) = bed_drawing(ui, &pattern, area.width_mm, area.height_mm, &t)
                                .unwrap_or((pattern.dx, pattern.dy));
                            // Keeps it on the bed after any size change too.
                            pattern = pattern.moved(x - pattern.dx, y - pattern.dy, area.width_mm, area.height_mm);
                        });
                        let problem = pattern.validate(area.width_mm, area.height_mm).err();
                        if let Some(p) = &problem {
                            widgets::band(ui, Band::Error, &p.0);
                        }
                        let (back, next, cancel) = Self::footer(
                            ui,
                            true,
                            Some(("Next", problem.is_none(), "Fix the pattern size.")),
                        );
                        if back {
                            go = 1;
                        }
                        if next {
                            go = 3;
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    3 => {
                        heading(ui, "Burn the pattern");
                        let json = serde_json::to_string(&pattern).unwrap_or_default();
                        ui.horizontal_top(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(240.0);
                                for (i, s) in ["Put a flat scrap sheet on the bed", "Check the settings on the right", "Click Start"].iter().enumerate() {
                                    ui.label(format!("{}. {s}", i + 1));
                                }
                                widgets::band(ui, Band::Hint, "The pattern is placed by bed coordinates.");
                                if laser.alarm.is_some() {
                                    widgets::band(ui, Band::Error, "The burn stopped. Clear the alarm, then click Start again.");
                                }
                            });
                            ui.vertical(|ui| {
                                let width = ui.available_width().min(320.0);
                                self.live_picture(ui, view, camera, width);
                                group(ui, "Settings", |ui| {
                                    egui::Grid::new("burn-settings").num_columns(2).show(ui, |ui| {
                                        ui.label("Speed (mm/s)");
                                        ui.add(egui::DragValue::new(&mut speed).range(1.0..=10_000.0));
                                        ui.end_row();
                                        ui.label("Max power (%)");
                                        ui.add(egui::DragValue::new(&mut power).range(0.0..=100.0));
                                        ui.end_row();
                                    });
                                    ui.menu_button("From materials…", |ui| {
                                        for e in workshop.active().library.entries() {
                                            if ui.button(format!("{} {} – {}", e.material, e.level(), e.description)).clicked() {
                                                speed = e.settings.speed;
                                                power = e.settings.max_power;
                                                ui.close();
                                            }
                                        }
                                    });
                                });
                                group(ui, "Run", |ui| {
                                    let on = Enabled::of(laser, true);
                                    let why = laser_panel::why_not(laser, false);
                                    let big = egui::vec2(100.0, size::ROW_BIG);
                                    ui.horizontal(|ui| {
                                        if ui.add_enabled(on.start, egui::Button::new("Frame").min_size(big)).on_disabled_hover_text(why).clicked() {
                                            actions.push(run("device.camera_pattern_frame", [json.clone()]));
                                        }
                                        if ui.add_enabled(on.start, egui::Button::new("Start").min_size(big)).on_disabled_hover_text(why).clicked() {
                                            actions.push(run(
                                                "device.camera_pattern",
                                                [json.clone(), format!("speed={speed}"), format!("power={power}")],
                                            ));
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        let mut more = Vec::new();
                                        laser_panel::pause_button(ui, laser, keymap, big, &mut more);
                                        laser_panel::stop_button(ui, laser, keymap, big, &mut more);
                                        actions.extend(more);
                                    });
                                });
                            });
                        });
                        let why = "Stop the burn or wait for it to finish.";
                        let (back, next, cancel) =
                            Self::footer(ui, !busy, Some(("Next", !busy, why)));
                        if back {
                            go = 2;
                        }
                        if next {
                            go = 4;
                            clicked.clear();
                            if !manual {
                                actions.push(Action::Camera(CameraAction::FindMarks(
                                    camera.to_owned(),
                                )));
                            }
                        }
                        if cancel && !busy {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    4 => {
                        heading(ui, "Find the marks");
                        let points: Vec<Point> = if manual {
                            clicked.clone()
                        } else {
                            match &view.marks {
                                Some(Ok(m)) => m.to_vec(),
                                _ => Vec::new(),
                            }
                        };
                        ui.horizontal_top(|ui| {
                            let width = (ui.available_width() - 240.0).max(240.0) * zoom;
                            egui::ScrollArea::both().max_width(ui.available_width() - 240.0).max_height(420.0).show(ui, |ui| {
                                if let Some((response, rect, scale)) = self.live_picture(ui, view, camera, width) {
                                    let to_screen = |p: Point| rect.min + egui::vec2(p.x as f32 * scale, p.y as f32 * scale);
                                    let stroke = egui::Stroke::new(weight::HEAVY, t.drawing);
                                    for (i, p) in points.iter().enumerate() {
                                        let c = to_screen(*p);
                                        ui.painter().line_segment([c - egui::vec2(8.0, 0.0), c + egui::vec2(8.0, 0.0)], stroke);
                                        ui.painter().line_segment([c - egui::vec2(0.0, 8.0), c + egui::vec2(0.0, 8.0)], stroke);
                                        ui.painter().text(c + egui::vec2(9.0, -9.0), egui::Align2::LEFT_BOTTOM, (i + 1).to_string(), egui::FontId::proportional(13.0), t.drawing);
                                    }
                                    if points.len() == 4 {
                                        let ring: Vec<egui::Pos2> = points.iter().chain(points.first()).map(|p| to_screen(*p)).collect();
                                        ui.painter().add(egui::Shape::line(ring, egui::Stroke::new(weight::THIN, t.drawing)));
                                    }
                                    if manual
                                        && clicked.len() < 4
                                        && response.clicked()
                                        && let Some(pos) = response.interact_pointer_pos()
                                    {
                                        let p = (pos - rect.min) / scale;
                                        clicked.push(Point::new(f64::from(p.x), f64::from(p.y)));
                                    }
                                }
                            });
                            ui.vertical(|ui| {
                                egui::Frame::new().fill(t.field_alt).corner_radius(crate::theme::ROUNDED).inner_margin(size::GAP).show(ui, |ui| {
                                    ui.set_width(210.0);
                                    if manual {
                                        ui.label(format!("Step {}: click mark {}", clicked.len().min(3) + 1, clicked.len().min(3) + 1));
                                        ui.label(egui::RichText::new("Click the center of each mark: top left, top right, bottom right, bottom left.").small());
                                        ui.label(if clicked.len() == 4 { "All marks placed".to_owned() } else { format!("Click mark {}", clicked.len() + 1) });
                                        if ui.add_enabled(!clicked.is_empty(), egui::Button::new("Discard")).clicked() {
                                            clicked.pop();
                                        }
                                        ui.add(egui::Slider::new(&mut zoom, 1.0..=4.0).text("Zoom"));
                                    } else {
                                        match &view.marks {
                                            Some(Ok(_)) => {
                                                ui.label("4 of 4 marks found");
                                            }
                                            Some(Err(e)) => {
                                                widgets::band(ui, Band::Error, e);
                                            }
                                            None => {
                                                ui.label("Looking for the marks…");
                                            }
                                        }
                                        if ui.button("Look again").clicked() {
                                            actions.push(Action::Camera(CameraAction::FindMarks(camera.to_owned())));
                                        }
                                    }
                                });
                            });
                        });
                        let ready = points.len() == 4;
                        let (back, next, cancel) = Self::footer(
                            ui,
                            true,
                            Some(("Next", ready, "All four marks are needed.")),
                        );
                        if back {
                            go = 3;
                        }
                        if next && ready {
                            let bed = pattern.centers(area.width_mm, area.height_mm);
                            let fmt = |pts: &[Point]| {
                                pts.iter()
                                    .map(|p| format!("[{},{}]", p.x, p.y))
                                    .collect::<Vec<_>>()
                                    .join(",")
                            };
                            let json = format!(
                                r#"{{"picture": [{}], "bed": [{}]}}"#,
                                fmt(&points),
                                fmt(&bed)
                            );
                            actions.push(run("camera.align", [camera.to_owned(), json]));
                            actions.push(Action::Camera(CameraAction::Capture(None)));
                            go = 5;
                        }
                        if cancel {
                            self.leave_flow(actions);
                            return;
                        }
                    }
                    _ => {
                        heading(ui, "Check the alignment");
                        if let Some(c) = &cam {
                            let live = view.live(camera).cloned().unwrap_or_default();
                            if let Some(texture) = self.textures.get(ui.ctx(), &live) {
                                let width = ui.available_width().min(560.0);
                                picture(ui, &texture, width, Some((c, project)));
                            }
                        }
                        ui.label("If the grid lines up with the burned marks, click Finish.");
                        ui.add_space(size::GAP);
                        ui.separator();
                        let mut finish = false;
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let min = egui::vec2(80.0, size::ROW);
                            finish = widgets::filled(ui, "Finish", Fill::Primary, min).clicked();
                            if ui
                                .add(egui::Button::new("Try again").min_size(min))
                                .clicked()
                            {
                                go = 4;
                            }
                            if ui.add(egui::Button::new("Back").min_size(min)).clicked() {
                                go = 4;
                            }
                        });
                        if finish {
                            self.leave_flow(actions);
                            if let Some(Host::Add { camera, name, .. }) = self.host.clone() {
                                self.host = Some(Host::Add {
                                    page: 4,
                                    camera,
                                    name,
                                });
                            }
                            return;
                        }
                    }
                }
                self.flow = Some(Flow::Align {
                    page: go,
                    manual: manual2,
                    pattern,
                    speed,
                    power,
                    clicked,
                    zoom,
                });
            }
        }
    }
}

/// A group box: heading over a thin frame.
fn group(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    let t = Tokens::of(ui.ctx());
    ui.label(egui::RichText::new(title).strong());
    egui::Frame::new()
        .fill(t.panel)
        .stroke(egui::Stroke::new(weight::THIN, t.border))
        .inner_margin(size::GAP)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
    ui.add_space(size::GAP_TIGHT);
}

/// The source fields of a camera; changes run at once.
fn source_box(ui: &mut egui::Ui, camera: &Camera, view: &CameraView, actions: &mut Vec<Action>) {
    let t = Tokens::of(ui.ctx());
    let live = view.live(&camera.name).cloned().unwrap_or_default();
    let (kind, status) = match &live.status {
        crate::camera_panel::CameraStatus::Connected { width, height } => {
            (Band::Success, format!("Connected at {width} × {height}"))
        }
        crate::camera_panel::CameraStatus::Failed(e) => {
            (Band::Error, format!("No picture source: {e}"))
        }
        _ => (Band::Warning, "Connecting…".to_owned()),
    };
    let set =
        |key: &str, value: String| run("camera.set", [camera.name.clone(), key.to_owned(), value]);
    match &camera.source {
        Source::Usb { index, format, .. } => {
            ui.horizontal(|ui| {
                ui.label("Device");
                let shown = view
                    .usb
                    .iter()
                    .find(|d| d.starts_with(&format!("{index}:")))
                    .cloned()
                    .unwrap_or_else(|| format!("{index}: camera"));
                egui::ComboBox::from_id_salt(("usb-device", &camera.name))
                    .selected_text(shown)
                    .show_ui(ui, |ui| {
                        if view.usb.is_empty() {
                            ui.label("No cameras found");
                        }
                        for d in &view.usb {
                            if ui.button(d).clicked()
                                && let Some((i, n)) = d.split_once(": ")
                            {
                                actions.push(set("index", i.to_owned()));
                                actions.push(set("device", n.to_owned()));
                            }
                        }
                    });
                if ui.small_button("Refresh").clicked() {
                    actions.push(Action::Camera(CameraAction::RefreshUsb));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Format");
                let mut f = format.clone();
                let r = ui.add(
                    egui::TextEdit::singleline(&mut f)
                        .hint_text("WxH, empty for the largest")
                        .desired_width(140.0),
                );
                if r.lost_focus() && &f != format {
                    actions.push(set("format", f));
                }
            });
        }
        Source::Network { url, rate } => {
            ui.horizontal(|ui| {
                ui.label("Address");
                let mut u = url.clone();
                let r = ui.add(egui::TextEdit::singleline(&mut u).desired_width(200.0));
                if r.lost_focus() && &u != url {
                    actions.push(set("url", u));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Pictures per second");
                let mut r = *rate;
                if ui
                    .add(egui::DragValue::new(&mut r).range(0.1..=30.0))
                    .drag_stopped()
                {
                    actions.push(set("rate", r.to_string()));
                }
            });
        }
        Source::Simulated => {
            ui.label(egui::RichText::new("LaserCraft's simulated camera sees the bed and what the simulated laser burns.").small().color(t.text_muted));
        }
    }
    ui.horizontal(|ui| {
        widgets::mark(ui, kind);
        ui.label(format!("Status: {status}"));
    });
}

/// The bed with the pattern on it (`bed` fill, grid, marks).
/// Draws the bed with the pattern; while dragged, returns the offset the
/// pattern should have (`dx`, `dy`, mm).
fn bed_drawing(
    ui: &mut egui::Ui,
    pattern: &Pattern,
    w: f64,
    h: f64,
    t: &Tokens,
) -> Option<(f64, f64)> {
    let width = ui.available_width().clamp(200.0, 420.0);
    let scale = width / w as f32;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, h as f32 * scale), egui::Sense::drag());
    let response = response.on_hover_cursor(egui::CursorIcon::Grab);
    // Measured from where the press began, not summed per frame: a pointer
    // that jumps to the press (touch, pen) must not move the pattern.
    if response.drag_started() {
        ui.data_mut(|d| d.insert_temp(response.id, (pattern.dx, pattern.dy)));
    }
    let start = ui.data(|d| d.get_temp::<(f64, f64)>(response.id));
    let origin = ui.input(|i| i.pointer.press_origin());
    let moved = match (
        response.dragged(),
        start,
        origin,
        response.interact_pointer_pos(),
    ) {
        (true, Some((x, y)), Some(o), Some(now)) => {
            let d = (now - o) / scale;
            Some((x + f64::from(d.x), y + f64::from(d.y)))
        }
        _ => None,
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, t.bed);
    p.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(weight::THIN, t.bed_edge),
        egui::StrokeKind::Middle,
    );
    let at = |q: Point| rect.min + egui::vec2(q.x as f32 * scale, q.y as f32 * scale);
    let centers = pattern.centers(w, h);
    let sheet = egui::Rect::from_two_pos(at(centers[0]), at(centers[2]))
        .expand(pattern.mark as f32 * scale)
        .intersect(rect);
    p.rect_filled(sheet, 2.0, egui::Color32::WHITE);
    let stroke = egui::Stroke::new(weight::NORMAL, t.text);
    for (i, c) in centers.iter().enumerate() {
        let c = at(*c);
        let r = (pattern.mark as f32 / 2.0 * scale).max(3.0);
        p.circle_stroke(c, r, stroke);
        p.circle_filled(c, (r * 0.25).max(1.0), t.text);
        p.text(
            c + egui::vec2(r + 2.0, -r - 2.0),
            egui::Align2::LEFT_BOTTOM,
            (i + 1).to_string(),
            egui::FontId::proportional(11.0),
            t.text,
        );
    }
    p.circle_filled(at(pattern.center(w, h)), 2.0, t.text);
    moved
}
