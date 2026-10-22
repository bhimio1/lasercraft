//! The Trace to vectors window (`docs/spec/trace.md`, "In the app";
//! `docs/spec/style/later-windows.md`, "Trace to vectors"). The app traces
//! and sends the picture and outlines back; the window only draws them and
//! edits the settings.

use std::sync::Arc;

use lasercraft_camera::Picture;
use lasercraft_commands::{Invocation, TraceSettings};
use lasercraft_geom::Point;

use crate::camera_panel::CameraAction;
use crate::shell::Action;
use crate::theme::{Tokens, size, weight};
use crate::widgets::{self, Band, Bounded, Fill};

/// What is traced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceSource {
    /// An image shape.
    Image(u32),
    /// The camera overlay.
    Camera,
}

/// What the window asks the app to trace.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRequest {
    /// The picture.
    pub source: TraceSource,
    /// The settings.
    pub settings: TraceSettings,
    /// Answer with [`TraceInfo::revision`] set to this.
    pub revision: u64,
}

/// The app's answer to a [`TraceRequest`].
#[derive(Debug, Clone, Default)]
pub struct TraceInfo {
    /// The request's revision.
    pub revision: u64,
    /// The picture to show (may be smaller than the traced one).
    pub picture: Option<Arc<Picture>>,
    /// Size of the traced bitmap, pixels; outlines are in these pixels.
    pub size: (u32, u32),
    /// The outlines, flattened.
    pub loops: Vec<Vec<Point>>,
    /// The nodes.
    pub nodes: Vec<Point>,
    /// Paths and nodes the trace would add.
    pub counts: (usize, usize),
    /// Why nothing can be traced.
    pub error: Option<String>,
}

/// The open window.
pub struct TraceWindow {
    source: TraceSource,
    settings: TraceSettings,
    revision: u64,
    asked: u64,
    info: TraceInfo,
    texture: Option<(usize, egui::TextureHandle)>,
    dim: bool,
    show_nodes: bool,
    drag_from: Option<Point>,
}

impl TraceWindow {
    /// A window on `source` with the default settings.
    #[must_use]
    pub fn new(source: TraceSource) -> Self {
        Self {
            source,
            settings: TraceSettings::default(),
            revision: 1,
            asked: 0,
            info: TraceInfo::default(),
            texture: None,
            dim: false,
            show_nodes: false,
            drag_from: None,
        }
    }

    /// A request when the settings changed since the last one.
    pub fn request(&mut self) -> Option<TraceRequest> {
        (self.asked != self.revision).then(|| {
            self.asked = self.revision;
            TraceRequest {
                source: self.source,
                settings: self.settings,
                revision: self.revision,
            }
        })
    }

    /// The app's answer.
    pub fn set_info(&mut self, info: TraceInfo) {
        if info.revision == self.revision {
            self.info = info;
        }
    }

    fn changed(&mut self) {
        self.revision += 1;
    }

    /// Draws the window; `false` once it is closed. OK adds the trace on
    /// layer `color`.
    pub fn show(&mut self, ctx: &egui::Context, color: u8, actions: &mut Vec<Action>) -> bool {
        let mut open = true;
        let mut keep = true;
        egui::Window::new("Trace to vectors")
            .id(egui::Id::unique("trace-window"))
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .default_size([760.0, 600.0])
            .bounded(ctx)
            .show(ctx, |ui| {
                let controls = 230.0;
                let preview = egui::vec2(
                    ui.available_width(),
                    (ui.available_height() - controls).max(160.0),
                );
                self.preview(ui, preview);
                ui.separator();
                self.controls(ui);
                ui.separator();
                keep = self.footer(ui, color, actions);
            });
        open && keep
    }

    fn preview(&mut self, ui: &mut egui::Ui, room: egui::Vec2) {
        let t = Tokens::of(ui.ctx());
        let (frame, response) = ui.allocate_exact_size(room, egui::Sense::drag());
        ui.painter().rect_filled(frame, 0.0, t.field);
        let Some(picture) = self.info.picture.clone() else {
            ui.painter().text(
                frame.center(),
                egui::Align2::CENTER_CENTER,
                "Tracing…",
                egui::FontId::proportional(14.0),
                t.text_muted,
            );
            return;
        };
        let (pw, ph) = self.info.size;
        if pw == 0 || ph == 0 {
            return;
        }
        let scale = (frame.width() / pw as f32).min(frame.height() / ph as f32);
        let shown = egui::vec2(pw as f32 * scale, ph as f32 * scale);
        let rect = egui::Rect::from_center_size(frame.center(), shown);
        let key = Arc::as_ptr(&picture) as usize;
        if self.texture.as_ref().is_none_or(|(k, _)| *k != key) {
            let image = egui::ColorImage::from_rgb(
                [picture.width() as usize, picture.height() as usize],
                picture.rgb(),
            );
            let handle =
                ui.ctx()
                    .load_texture("trace-picture", image, egui::TextureOptions::LINEAR);
            self.texture = Some((key, handle));
        }
        if let Some((_, texture)) = &self.texture {
            let tint = if self.dim {
                egui::Color32::from_white_alpha(90)
            } else {
                egui::Color32::WHITE
            };
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        }
        let to_screen = |p: Point| rect.min + egui::vec2(p.x as f32 * scale, p.y as f32 * scale);
        let to_pixel = |s: egui::Pos2| {
            let v = (s - rect.min) / scale;
            Point::new(
                f64::from(v.x).clamp(0.0, f64::from(pw)),
                f64::from(v.y).clamp(0.0, f64::from(ph)),
            )
        };
        let stroke = egui::Stroke::new(weight::NORMAL, t.drawing);
        for l in &self.info.loops {
            let mut pts: Vec<egui::Pos2> = l.iter().map(|p| to_screen(*p)).collect();
            if let Some(first) = pts.first().copied() {
                pts.push(first);
            }
            ui.painter().add(egui::Shape::line(pts, stroke));
        }
        if self.show_nodes {
            for n in &self.info.nodes {
                let c = to_screen(*n);
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(c, egui::vec2(4.0, 4.0)),
                    0.0,
                    t.node,
                );
            }
        }
        // The trace area: drawn by dragging, kept in the settings.
        // From where the press began, not where the drag was recognised.
        if response.drag_started()
            && let Some(p) = ui.input(|i| i.pointer.press_origin())
        {
            self.drag_from = Some(to_pixel(p));
        }
        let mut area = self.settings.area.map(|[x0, y0, x1, y1]| {
            (
                Point::new(f64::from(x0), f64::from(y0)),
                Point::new(f64::from(x1), f64::from(y1)),
            )
        });
        if let (Some(from), Some(now)) = (self.drag_from, response.interact_pointer_pos()) {
            area = Some((from, to_pixel(now)));
        }
        if response.drag_stopped()
            && let Some((a, b)) = area.take().filter(|_| self.drag_from.is_some())
        {
            self.drag_from = None;
            let (x0, x1) = (a.x.min(b.x).floor() as u32, a.x.max(b.x).ceil() as u32);
            let (y0, y1) = (a.y.min(b.y).floor() as u32, a.y.max(b.y).ceil() as u32);
            if x1 > x0 + 1 && y1 > y0 + 1 {
                self.settings.area = Some([x0, y0, x1, y1]);
                self.changed();
            }
            area = self.settings.area.map(|[x0, y0, x1, y1]| {
                (
                    Point::new(f64::from(x0), f64::from(y0)),
                    Point::new(f64::from(x1), f64::from(y1)),
                )
            });
        }
        if let Some((a, b)) = area {
            let r = egui::Rect::from_two_pos(to_screen(a), to_screen(b));
            ui.painter().rect_stroke(
                r,
                0.0,
                egui::Stroke::new(weight::NORMAL, t.marquee),
                egui::StrokeKind::Middle,
            );
            for c in [
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
            ] {
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(c, egui::vec2(6.0, 6.0)),
                    0.0,
                    t.handle_fill,
                );
            }
        }
    }

    /// A number field and slider; `true` when a change is final.
    fn level(ui: &mut egui::Ui, value: &mut u8) -> bool {
        let mut v = i32::from(*value);
        let field = ui.add(egui::DragValue::new(&mut v).range(0..=255));
        let slider = ui.add(egui::Slider::new(&mut v, 0..=255).show_value(false));
        *value = v.clamp(0, 255) as u8;
        settled(&field) || settled(&slider)
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        let mut done = false;
        let s = &mut self.settings;
        egui::Grid::new("trace-controls")
            .num_columns(4)
            .spacing([size::GAP, size::GAP_TIGHT])
            .show(ui, |ui| {
                ui.label("Brightness from").on_hover_text(
                    "Darkest brightness that counts as ink\n0 is black, 255 is white.",
                );
                ui.horizontal(|ui| done |= Self::level(ui, &mut s.from));
                ui.label("Brightness to")
                    .on_hover_text("Lightest brightness that counts as ink");
                ui.horizontal(|ui| done |= Self::level(ui, &mut s.to));
                ui.end_row();
                ui.label("");
                done |= widgets::switch(ui, &mut s.transparency, "Use transparency")
                    .on_hover_text(
                        "Trace everything that is not white instead of a brightness range",
                    )
                    .changed();
                ui.label("");
                done |= widgets::switch(ui, &mut s.edges, "Follow edges (sketches)")
                    .on_hover_text(
                        "Trace along local contrast; good for sketches and unevenly lit photos",
                    )
                    .changed();
                ui.end_row();
                ui.label("Drop specks under (px)")
                    .on_hover_text("Ignore traced bits smaller than this many image pixels");
                done |= settled(&ui.add(egui::DragValue::new(&mut s.speck).range(0..=10_000)));
                ui.label("Smoothness")
                    .on_hover_text("0 gives only straight lines; higher values give more curves");
                done |= settled(
                    &ui.add(
                        egui::DragValue::new(&mut s.smoothness)
                            .range(0.0..=1.33)
                            .speed(0.01)
                            .fixed_decimals(2),
                    ),
                );
                ui.end_row();
                ui.label("Node reduction").on_hover_text(
                    "Higher values use fewer nodes and follow the image less closely",
                );
                done |= settled(
                    &ui.add(
                        egui::DragValue::new(&mut s.reduction)
                            .range(0.0..=5.0)
                            .speed(0.01)
                            .fixed_decimals(2),
                    ),
                );
                ui.end_row();
            });
        if s.from > s.to {
            std::mem::swap(&mut s.from, &mut s.to);
        }
        ui.horizontal(|ui| {
            ui.toggle_value(&mut self.dim, "Dim image");
            ui.toggle_value(&mut self.show_nodes, "Show nodes");
            if ui
                .add_enabled(
                    self.settings.area.is_some(),
                    egui::Button::new("Clear trace area"),
                )
                .clicked()
            {
                self.settings.area = None;
                done = true;
            }
            if matches!(self.source, TraceSource::Image(_)) {
                widgets::switch(ui, &mut self.settings.delete, "Delete image when done")
                    .on_hover_text("Remove the image after the trace is added");
            }
        });
        if done {
            self.changed();
        }
    }

    /// Status and OK / Cancel; `false` when the window should close.
    fn footer(&mut self, ui: &mut egui::Ui, color: u8, actions: &mut Vec<Action>) -> bool {
        let current = self.info.revision == self.revision;
        match (&self.info.error, current) {
            (Some(e), true) => {
                widgets::band(ui, Band::Error, e);
            }
            (None, true) if self.info.picture.is_some() => {
                let (paths, nodes) = self.info.counts;
                ui.label(format!("{paths} path(s), {nodes} nodes"));
            }
            _ => {
                ui.label("Tracing…");
            }
        }
        let mut keep = true;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    keep = false;
                }
                let ready = current && self.info.error.is_none() && self.info.counts.0 > 0;
                let ok = ui
                    .add_enabled_ui(ready, |ui| {
                        widgets::filled(ui, "OK", Fill::Primary, egui::vec2(80.0, size::ROW))
                    })
                    .inner;
                if ready && ok.clicked() {
                    let json = serde_json::to_string(&self.settings).unwrap_or_default();
                    actions.push(match self.source {
                        TraceSource::Image(id) => Action::Run(Invocation::new(
                            "image.trace",
                            [id.to_string(), json, format!("@{color}")],
                        )),
                        TraceSource::Camera => Action::Camera(CameraAction::Trace {
                            settings: json,
                            color,
                        }),
                    });
                    keep = false;
                }
            });
        });
        keep
    }
}

/// Whether an edit is final: changed and not being dragged, or released.
fn settled(r: &egui::Response) -> bool {
    r.drag_stopped() || (r.changed() && !r.dragged())
}
