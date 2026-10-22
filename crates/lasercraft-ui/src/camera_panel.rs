//! The Camera panel (`docs/spec/camera.md`, "In the app";
//! `docs/spec/style/later-windows.md`, "Camera panel") and the data the
//! app hands the camera views. Pictures come from the app; the panel only
//! shows them and asks for actions.

use std::collections::HashMap;
use std::sync::Arc;

use lasercraft_camera::Picture;
use lasercraft_camera::lens::Lens;
use lasercraft_camera::overlay::Overlay;
use lasercraft_commands::Invocation;
use lasercraft_geom::{Point, Rect};
use lasercraft_project::Project;
use lasercraft_workshop::{Camera, Workshop};

use crate::shell::Action;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Band};

/// A camera's connection as the app sees it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CameraStatus {
    /// Not running (no camera view is shown).
    #[default]
    Off,
    /// Starting.
    Connecting,
    /// Delivering pictures of this size.
    Connected {
        /// Pixels across.
        width: u32,
        /// Pixels down.
        height: u32,
    },
    /// The source failed; the app retries.
    Failed(String),
}

/// One camera's live state.
#[derive(Debug, Clone, Default)]
pub struct CameraLive {
    /// The camera's name.
    pub name: String,
    /// Connection.
    pub status: CameraStatus,
    /// The latest picture.
    pub picture: Option<Arc<Picture>>,
    /// Changes with every new picture.
    pub revision: u64,
}

/// One picture taken for the lens correction.
#[derive(Debug, Clone, PartialEq)]
pub struct LensCapture {
    /// Where the app saved it.
    pub path: String,
    /// The sheet's dots found in it, raw pixels.
    pub dots: Vec<Point>,
    /// Why no dots were found.
    pub error: Option<String>,
}

/// What the app tells the camera views.
#[derive(Debug, Clone, Default)]
pub struct CameraView {
    /// The project's device profile's cameras, in order.
    pub cameras: Vec<CameraLive>,
    /// The overlay, if one was captured.
    pub overlay: Option<Arc<Overlay>>,
    /// Changes with every new overlay.
    pub overlay_revision: u64,
    /// Seconds since the overlay's pictures were taken.
    pub picture_age: Option<u64>,
    /// The head moved since.
    pub stale: bool,
    /// USB cameras the system lists, as "index: name".
    pub usb: Vec<String>,
    /// Pictures per second while shown.
    pub rate: f64,
    /// Pictures taken for the lens correction (the wizard's).
    pub captures: Vec<LensCapture>,
    /// Every model fitted to the captures.
    pub fits: Vec<Result<Lens, String>>,
    /// Marks found automatically: raw pixels, or why not.
    pub marks: Option<Result<[Point; 4], String>>,
}

impl CameraView {
    /// A camera's live state by name.
    #[must_use]
    pub fn live(&self, name: &str) -> Option<&CameraLive> {
        self.cameras.iter().find(|c| c.name == name)
    }
}

/// Things only the app can do with cameras.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraAction {
    /// Rebuild the overlay from new pictures (within `region`, bed mm).
    Capture(Option<Rect>),
    /// Pictures per second while a camera is shown.
    Rate(f64),
    /// List USB cameras again.
    RefreshUsb,
    /// Keep the camera's latest picture for the lens correction.
    KeepPicture(String),
    /// Drop a kept picture.
    DropPicture(usize),
    /// Forget every kept picture.
    ClearPictures,
    /// Fit every lens model to the kept pictures, for this camera.
    FitLens(String),
    /// Find the alignment marks in the camera's latest picture.
    FindMarks(String),
    /// Show the simulated camera the calibration sheet at a place (0–4).
    SimulatedSheet(Option<usize>),
    /// Put the camera's picture on the canvas as an image.
    InsertPicture {
        /// The camera.
        camera: String,
        /// Lens-corrected and aligned on the bed, or as captured.
        corrected: bool,
    },
}

/// Picture textures by camera, rebuilt when a picture changes.
#[derive(Default)]
pub(crate) struct PictureTextures {
    by_name: HashMap<String, (u64, egui::TextureHandle)>,
}

impl PictureTextures {
    pub(crate) fn get(
        &mut self,
        ctx: &egui::Context,
        live: &CameraLive,
    ) -> Option<egui::TextureHandle> {
        let picture = live.picture.as_ref()?;
        if let Some((rev, t)) = self.by_name.get(&live.name)
            && *rev == live.revision
        {
            return Some(t.clone());
        }
        let image = egui::ColorImage::from_rgb(
            [picture.width() as usize, picture.height() as usize],
            picture.rgb(),
        );
        let t = ctx.load_texture(
            format!("camera-{}", live.name),
            image,
            egui::TextureOptions::LINEAR,
        );
        self.by_name
            .insert(live.name.clone(), (live.revision, t.clone()));
        Some(t)
    }
}

/// Draws a picture fitted into `width`, with a grid of bed lines when the
/// camera is aligned. Returns the response and where the picture landed
/// with its scale (screen points per picture pixel).
pub(crate) fn picture(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    width: f32,
    grid: Option<(&Camera, &Project)>,
) -> (egui::Response, egui::Rect, f32) {
    let [w, h] = texture.size();
    let scale = width / w as f32;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, h as f32 * scale), egui::Sense::click());
    ui.painter().image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    if let Some((camera, project)) = grid
        && let Some(alignment) = camera.alignment()
    {
        let lens = &camera.lens_or_plain(w as u32, h as u32);
        let t = Tokens::of(ui.ctx());
        let area = project.work_area();
        let to_screen = |p: Point| {
            alignment.to_picture(lens, p).map(|px| {
                let (sx, sy) = (
                    f64::from(w as u32) / f64::from(lens.width),
                    f64::from(h as u32) / f64::from(lens.height),
                );
                rect.min + egui::vec2((px.x * sx) as f32 * scale, (px.y * sy) as f32 * scale)
            })
        };
        let stroke = egui::Stroke::new(weight::HAIRLINE, t.grid_major);
        let line = |a: Point, b: Point| {
            let pts: Vec<egui::Pos2> = (0..=20)
                .filter_map(|k| to_screen(a + (b - a) * (f64::from(k) / 20.0)))
                .collect();
            ui.painter().add(egui::Shape::line(pts, stroke));
        };
        let mut x = 0.0;
        while x <= area.width_mm {
            line(Point::new(x, 0.0), Point::new(x, area.height_mm));
            x += 10.0;
        }
        let mut y = 0.0;
        while y <= area.height_mm {
            line(Point::new(0.0, y), Point::new(area.width_mm, y));
            y += 10.0;
        }
    }
    (response, rect, scale)
}

/// UI state of the Camera panel.
#[derive(Default)]
pub(crate) struct CameraPanel {
    pub(crate) textures: PictureTextures,
    /// The camera menu's target.
    pub(crate) wants: Option<PanelWish>,
}

/// What the panel asks the shell to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelWish {
    Add,
    Import,
    Setup(String),
    Lens(String),
    Align(String),
}

fn badge(ui: &mut egui::Ui, kind: Band, tip: &str) -> egui::Response {
    let mark = widgets::mark(ui, kind);
    ui.interact(mark.rect, mark.id.with("badge"), egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip)
}

impl CameraPanel {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        workshop: &Workshop,
        view: &CameraView,
        actions: &mut Vec<Action>,
    ) {
        let t = Tokens::of(ui.ctx());
        let cameras: Vec<Camera> = project
            .device()
            .and_then(|d| workshop.cameras(d).ok())
            .map(<[Camera]>::to_vec)
            .unwrap_or_default();
        let Some(_) = project.device() else {
            ui.label(
                egui::RichText::new(
                    "Cameras belong to a device profile. Choose one in the Machine panel first.",
                )
                .color(t.text_muted),
            );
            return;
        };
        if cameras.is_empty() {
            ui.label(
                egui::RichText::new("No camera set up. Use Add camera… to add one.")
                    .color(t.text_muted),
            );
            ui.add_space(size::GAP);
            self.bottom_row(ui);
            return;
        }
        let half = ((ui.available_width() - size::GAP) / 2.0).max(60.0);
        let tall = egui::vec2(half, size::ROW * 1.5);
        ui.horizontal(|ui| {
            let selection = project.selection();
            let region = project.bounds_of(&selection).ok().flatten().map(|b| {
                Rect::from_corners(b.min - Point::new(5.0, 5.0), b.max + Point::new(5.0, 5.0))
            });
            let tip = if region.is_some() {
                "Take a new camera picture for the overlay\nOnly the part under the selection is replaced."
            } else {
                "Take a new camera picture for the overlay"
            };
            if ui
                .add(egui::Button::new("Capture camera image").min_size(tall))
                .on_hover_text(tip)
                .clicked()
            {
                actions.push(Action::Camera(CameraAction::Capture(region)));
            }
            ui.add_enabled(false, egui::Button::new("Trace camera image…").min_size(tall))
                .on_disabled_hover_text("Tracing comes later.");
        });
        let first = &cameras[0];
        let mut opacity = first.opacity;
        ui.horizontal(|ui| {
            ui.label("Overlay opacity (%)");
            let r = ui.add(egui::Slider::new(&mut opacity, 0.0..=100.0).max_decimals(0));
            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                for c in &cameras {
                    actions.push(Action::Run(Invocation::new(
                        "camera.set",
                        [
                            c.name.clone(),
                            "opacity".to_owned(),
                            format!("{opacity:.0}"),
                        ],
                    )));
                }
                actions.push(Action::Camera(CameraAction::Capture(None)));
            }
        });
        egui::CollapsingHeader::new("Overlay fit")
            .default_open(false)
            .show(ui, |ui| {
                let mut fit = first.fit;
                let mut changed = Vec::new();
                egui::Grid::new("overlay-fit")
                    .num_columns(4)
                    .show(ui, |ui| {
                        for (label, key, value) in [
                            ("Overlay width (%)", "fit_width", &mut fit.width),
                            ("Overlay height (%)", "fit_height", &mut fit.height),
                        ] {
                            ui.label(label);
                            if ui
                                .add(
                                    egui::DragValue::new(value)
                                        .range(10.0..=1000.0)
                                        .max_decimals(2),
                                )
                                .changed()
                            {
                                changed.push((key, *value));
                            }
                        }
                        ui.end_row();
                        for (label, key, value) in [
                            ("Overlay offset X (mm)", "fit_dx", &mut fit.dx),
                            ("Overlay offset Y (mm)", "fit_dy", &mut fit.dy),
                        ] {
                            ui.label(label);
                            if ui
                                .add(
                                    egui::DragValue::new(value)
                                        .range(-1000.0..=1000.0)
                                        .max_decimals(2),
                                )
                                .changed()
                            {
                                changed.push((key, *value));
                            }
                        }
                        ui.end_row();
                    });
                for (key, value) in changed {
                    actions.push(Action::Run(Invocation::new(
                        "camera.set",
                        [first.name.clone(), key.to_owned(), value.to_string()],
                    )));
                }
            });
        ui.horizontal(|ui| match view.picture_age {
            Some(_) if view.stale => {
                widgets::mark(ui, Band::Warning);
                ui.label("Picture taken before the head last moved");
            }
            Some(s) => {
                ui.label(
                    egui::RichText::new(format!("Picture taken {} ago", ago(s)))
                        .small()
                        .color(t.text_muted),
                );
            }
            None => {
                ui.label(
                    egui::RichText::new("No picture yet")
                        .small()
                        .color(t.text_muted),
                );
            }
        });
        egui::Frame::new()
            .fill(t.field)
            .stroke(egui::Stroke::new(weight::THIN, t.border))
            .inner_margin(size::GAP_TIGHT)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for c in &cameras {
                    self.block(ui, project, c, view, actions);
                }
            });
        ui.add_space(size::GAP);
        self.bottom_row(ui);
    }

    fn bottom_row(&mut self, ui: &mut egui::Ui) {
        let half = ((ui.available_width() - size::GAP) / 2.0).max(60.0);
        let tall = egui::vec2(half, size::ROW * 1.5);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Add camera…").min_size(tall))
                .clicked()
            {
                self.wants = Some(PanelWish::Add);
            }
            if ui
                .add(egui::Button::new("Import camera…").min_size(tall))
                .clicked()
            {
                self.wants = Some(PanelWish::Import);
            }
        });
    }

    fn block(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        camera: &Camera,
        view: &CameraView,
        actions: &mut Vec<Action>,
    ) {
        let t = Tokens::of(ui.ctx());
        let live = view.live(&camera.name).cloned().unwrap_or_default();
        let id = ui.make_persistent_id(("camera-block", &camera.name));
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                if matches!(live.status, CameraStatus::Connected { .. }) {
                    let (r, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.0, t.success);
                }
                if ui
                    .small_button("Capture")
                    .on_hover_text("Capture this camera's part of the overlay")
                    .clicked()
                {
                    actions.push(Action::Camera(CameraAction::Capture(None)));
                }
                let mut used = camera.used;
                if widgets::switch(ui, &mut used, "")
                    .on_hover_text("Use this camera in the overlay")
                    .changed()
                {
                    actions.push(Action::Run(Invocation::new(
                        "camera.set",
                        [
                            camera.name.clone(),
                            "use".to_owned(),
                            if used { "on" } else { "off" }.to_owned(),
                        ],
                    )));
                }
                ui.label(&camera.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (kind, tip) = match (camera.alignment(), camera.alignment_outdated()) {
                        (Some(_), false) => (Band::Success, "Alignment: set".to_owned()),
                        (Some(_), true) => (
                            Band::Warning,
                            "Alignment: made with another lens correction. Click to align again."
                                .to_owned(),
                        ),
                        (None, _) => (
                            Band::Warning,
                            "Alignment: missing. Click to align.".to_owned(),
                        ),
                    };
                    if badge(ui, kind, &tip).clicked() {
                        self.wants = Some(PanelWish::Align(camera.name.clone()));
                    }
                    let (kind, tip) = match &camera.lens {
                        Some(l) => (
                            Band::Success,
                            format!(
                                "Lens: corrected ({}, fit {:.0}%)",
                                l.model.title(),
                                l.score()
                            ),
                        ),
                        None => (
                            Band::Warning,
                            "Lens: not calibrated. Click to calibrate.".to_owned(),
                        ),
                    };
                    if badge(ui, kind, &tip).clicked() {
                        self.wants = Some(PanelWish::Lens(camera.name.clone()));
                    }
                    let (kind, tip) = match &live.status {
                        CameraStatus::Connected { width, height } => (
                            Band::Success,
                            format!("Source: connected at {width} × {height}"),
                        ),
                        CameraStatus::Connecting | CameraStatus::Off => {
                            (Band::Warning, "Source: connecting".to_owned())
                        }
                        CameraStatus::Failed(e) => (Band::Error, format!("Source: {e}")),
                    };
                    if badge(ui, kind, &tip).clicked() {
                        self.wants = Some(PanelWish::Setup(camera.name.clone()));
                    }
                });
            })
            .body(|ui| {
                if let CameraStatus::Failed(e) = &live.status {
                    widgets::band(ui, Band::Error, &format!("No picture source: {e}"));
                }
                if let Some(texture) = self.textures.get(ui.ctx(), &live) {
                    let grid = Some((camera, project));
                    let (response, ..) = picture(ui, &texture, ui.available_width(), grid);
                    response.context_menu(|ui| {
                        if ui.button("Edit…").clicked() {
                            self.wants = Some(PanelWish::Setup(camera.name.clone()));
                            ui.close();
                        }
                    });
                }
                egui::CollapsingHeader::new("Picture settings")
                    .id_salt(("picture-settings", &camera.name))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("This camera has no adjustable controls.")
                                .color(t.text_muted),
                        );
                    });
                egui::CollapsingHeader::new("Advanced")
                    .id_salt(("camera-advanced", &camera.name))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Preview rate");
                            egui::ComboBox::from_id_salt(("rate", &camera.name))
                                .selected_text(format!("{} per second", view.rate))
                                .show_ui(ui, |ui| {
                                    for r in [1.0, 2.0, 5.0, 10.0, 15.0] {
                                        if ui
                                            .selectable_label(
                                                view.rate == r,
                                                format!("{r} per second"),
                                            )
                                            .clicked()
                                        {
                                            actions.push(Action::Camera(CameraAction::Rate(r)));
                                        }
                                    }
                                });
                            if let CameraStatus::Connected { width, height } = live.status {
                                ui.label(
                                    egui::RichText::new(format!("at {width} × {height}"))
                                        .color(t.text_muted),
                                );
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Insert picture");
                            let ready = camera.alignment().is_some();
                            if ui
                                .add_enabled(
                                    ready && live.picture.is_some(),
                                    egui::Button::new("Corrected"),
                                )
                                .on_disabled_hover_text("Needs a picture and a bed alignment.")
                                .clicked()
                            {
                                actions.push(Action::Camera(CameraAction::InsertPicture {
                                    camera: camera.name.clone(),
                                    corrected: true,
                                }));
                            }
                            if ui
                                .add_enabled(
                                    live.picture.is_some(),
                                    egui::Button::new("As captured"),
                                )
                                .clicked()
                            {
                                actions.push(Action::Camera(CameraAction::InsertPicture {
                                    camera: camera.name.clone(),
                                    corrected: false,
                                }));
                            }
                        });
                    });
            });
    }
}

fn ago(seconds: u64) -> String {
    match seconds {
        0..=59 => format!("{seconds} s"),
        60..=3599 => format!("{} min", seconds / 60),
        _ => format!("{} h", seconds / 3600),
    }
}
