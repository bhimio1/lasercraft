//! The canvas: draws the bed, grid and shapes, and feeds pointer input to
//! [`Interaction`] (`docs/spec/canvas.md`).

use egui::{Color32, Pos2, Sense, Stroke, StrokeKind};
use lasercraft_commands::Invocation;
use lasercraft_geom::manipulate::Handle;
use lasercraft_geom::{Affine, Point, Rect, flatten};
use lasercraft_project::{BoxMode, Origin, Project};

use crate::colors::layer_color;
use crate::interaction::{Ctx, Draft, GRID_MM, Interaction, Mods, Tool, rotate_handle};
use crate::view::{View, ZOOM_STEP};

/// Major grid lines every this many millimetres.
const MAJOR_GRID_MM: f64 = 50.0;
/// Minor grid lines closer than this many pixels are hidden.
const MIN_GRID_PX: f64 = 6.0;
/// Margin around the bed when fitting, in pixels.
const FIT_MARGIN_PX: f64 = 24.0;
/// Outline flattening tolerance on screen, in pixels.
const FLATTEN_PX: f64 = 0.3;
/// Scroll pixels per factor-of-e zoom.
const SCROLL_PER_E: f64 = 400.0;

/// The bed is drawn as a light sheet in every theme so dark layer colors
/// (layer 0 is black) stay visible.
const BED: Color32 = Color32::from_gray(246);
const GRID: Color32 = Color32::from_gray(90);
const SHAPE: Color32 = Color32::from_rgb(70, 140, 235);
const SELECTED: Color32 = Color32::from_rgb(255, 150, 40);
const DRAFT: Color32 = Color32::from_rgb(120, 200, 120);
const ORIGIN: Color32 = Color32::from_rgb(220, 60, 40);

fn pos(p: Point) -> Pos2 {
    Pos2::new(p.x as f32, p.y as f32)
}

fn point(p: Pos2) -> Point {
    Point::new(f64::from(p.x), f64::from(p.y))
}

fn rect_of(r: egui::Rect) -> Rect {
    Rect::from_corners(point(r.min), point(r.max))
}

/// Canvas state that lives in the UI only: view, tool and gesture.
#[derive(Default)]
pub struct Canvas {
    view: Option<View>,
    fit_requested: bool,
    interaction: Interaction,
    /// The primary button went down on the canvas (so its release is ours).
    pressed_here: bool,
    screen: Option<Rect>,
}

impl Canvas {
    /// The active tool.
    #[must_use]
    pub fn tool(&self) -> Tool {
        self.interaction.tool()
    }

    /// The palette color for new shapes.
    #[must_use]
    pub fn color(&self) -> lasercraft_project::Color {
        self.interaction.color()
    }

    /// Sets the palette color for new shapes.
    pub fn set_color(&mut self, color: lasercraft_project::Color) {
        self.interaction.set_color(color);
    }

    /// Switches tool.
    pub fn set_tool(&mut self, tool: Tool) {
        self.interaction.set_tool(tool);
    }

    /// The current zoom and pan, once the canvas has been laid out.
    #[must_use]
    pub fn view(&self) -> Option<View> {
        self.view
    }

    /// Pointer position in document millimetres, if over the canvas.
    #[must_use]
    pub fn pointer(&self) -> Option<Point> {
        self.interaction.pointer()
    }

    /// Runs `view.zoom_in`, `view.zoom_out` or `view.zoom_fit`.
    pub fn zoom(&mut self, action: &str) {
        let Some(screen) = self.screen else {
            self.fit_requested = true;
            return;
        };
        match (action, &mut self.view) {
            ("view.zoom_in", Some(v)) => v.zoom_about(screen.center(), ZOOM_STEP),
            ("view.zoom_out", Some(v)) => v.zoom_about(screen.center(), 1.0 / ZOOM_STEP),
            _ => self.fit_requested = true,
        }
    }

    /// Draws the canvas and returns the commands the user's gestures produced.
    pub fn show(&mut self, ui: &mut egui::Ui, project: &Project) -> Vec<Invocation> {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let screen = rect_of(response.rect);
        self.screen = Some(screen);
        let area = project.work_area();
        let bed = Rect::from_corners(Point::ZERO, Point::new(area.width_mm, area.height_mm));
        if self.view.is_none() || self.fit_requested {
            self.view = Some(View::fit(&screen, &bed, FIT_MARGIN_PX));
            self.fit_requested = false;
        }
        let mut view = self
            .view
            .unwrap_or_else(|| View::fit(&screen, &bed, FIT_MARGIN_PX));
        let out = self.handle_input(ui, &response, &mut view, project);
        self.view = Some(view);
        self.paint(ui, &painter, &view, project, &bed);
        out
    }

    fn handle_input(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        view: &mut View,
        project: &Project,
    ) -> Vec<Invocation> {
        let hovered = response.contains_pointer();
        let input = ui.input(|i| {
            (
                i.pointer.latest_pos(),
                i.pointer.delta(),
                i.smooth_scroll_delta.y,
                i.zoom_delta(),
                i.modifiers,
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.primary_released(),
                i.pointer.middle_down(),
                i.key_down(egui::Key::Space),
                i.key_pressed(egui::Key::Enter),
            )
        });
        let (
            latest,
            delta,
            scroll,
            pinch,
            modifiers,
            pressed,
            down,
            released,
            middle,
            space,
            enter,
        ) = input;
        let mods = Mods {
            shift: modifiers.shift,
            ctrl: modifiers.command || modifiers.ctrl,
        };
        let pointer_doc = latest.map(|p| view.to_doc(point(p)));
        self.interaction.hover(pointer_doc.filter(|_| hovered));

        if hovered && let Some(p) = latest {
            let factor = (f64::from(scroll) / SCROLL_PER_E).exp() * f64::from(pinch);
            if factor != 1.0 {
                view.zoom_about(point(p), factor);
            }
        }
        let panning = middle || (space && down);
        if panning && (hovered || self.pressed_here) {
            view.pan(point(Pos2::ZERO + delta));
            return Vec::new();
        }

        let ctx = Ctx {
            project,
            px_per_mm: view.scale(),
        };
        let Some(p) = latest.map(|p| view.to_doc(point(p))) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if pressed && hovered && !space {
            self.pressed_here = true;
            out.extend(self.interaction.press(p, mods, &ctx));
        } else if released && self.pressed_here {
            self.pressed_here = false;
            out.extend(self.interaction.release(p, mods, &ctx));
        } else if down && self.pressed_here {
            self.interaction.drag_to(p, mods, &ctx);
        }
        if response.double_clicked() {
            out.extend(self.interaction.double_click());
        }
        if enter && self.tool() == Tool::Line && !ui.ctx().text_edit_focused() {
            out.extend(self.interaction.finish_line(false));
        }
        out
    }

    fn paint(
        &self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        view: &View,
        project: &Project,
        bed: &Rect,
    ) {
        let visuals = ui.visuals();
        painter.rect_filled(painter.clip_rect(), 0.0, visuals.extreme_bg_color);
        let bed_screen =
            egui::Rect::from_two_pos(pos(view.to_screen(bed.min)), pos(view.to_screen(bed.max)));
        painter.rect_filled(bed_screen, 0.0, BED);
        paint_grid(painter, view, bed, GRID);
        painter.rect_stroke(bed_screen, 0.0, Stroke::new(1.5, GRID), StrokeKind::Middle);
        let corner = match project.work_area().origin {
            Origin::FrontLeft => bed_screen.left_bottom(),
            Origin::FrontRight => bed_screen.right_bottom(),
            Origin::RearLeft => bed_screen.left_top(),
            Origin::RearRight => bed_screen.right_top(),
        };
        painter.circle_filled(corner, 5.0, ORIGIN);

        let preview = self.interaction.preview();
        let tolerance = FLATTEN_PX / view.scale();
        for shape in project.shapes() {
            let Ok(layer) = project.layer(shape.layer()) else {
                continue;
            };
            if !layer.show {
                continue;
            }
            let moved = preview.and_then(|(ids, t)| ids.contains(&shape.id()).then_some(t));
            let (points, closed) = shape.outline(tolerance);
            let screen: Vec<Pos2> = points
                .iter()
                .map(|p| pos(view.to_screen(moved.map_or(*p, |t| t.apply(*p)))))
                .collect();
            let mut color = layer_color(shape.layer());
            if !layer.output {
                color = color.gamma_multiply(0.3);
            }
            let width = if project.is_selected(shape.id()) {
                2.5
            } else {
                1.25
            };
            let stroke = Stroke::new(width, color);
            painter.add(if closed {
                egui::Shape::closed_line(screen, stroke)
            } else {
                egui::Shape::line(screen, stroke)
            });
        }
        paint_selection(
            painter,
            view,
            project,
            preview.map(|(_, t)| t),
            visuals.extreme_bg_color,
        );
        paint_gesture(&self.interaction, ui, painter, view);
    }
}

fn paint_selection(
    painter: &egui::Painter,
    view: &View,
    project: &Project,
    preview: Option<Affine>,
    handle_fill: Color32,
) {
    let selection = project.selection();
    let Ok(Some(bounds)) = project.bounds_of(&selection) else {
        return;
    };
    let bounds = preview.map_or(bounds, |t| bounds.transformed(&t));
    let stroke = Stroke::new(1.0, SELECTED);
    let a = pos(view.to_screen(bounds.min));
    let b = pos(view.to_screen(bounds.max));
    let box_points = [a, Pos2::new(b.x, a.y), b, Pos2::new(a.x, b.y), a];
    painter.extend(egui::Shape::dashed_line(&box_points, stroke, 5.0, 4.0));
    for handle in Handle::ALL {
        let c = pos(view.to_screen(handle.position(&bounds)));
        let square = egui::Rect::from_center_size(c, egui::vec2(8.0, 8.0));
        painter.rect_filled(square, 1.0, handle_fill);
        painter.rect_stroke(square, 1.0, stroke, StrokeKind::Middle);
    }
    let top = pos(view.to_screen(Handle::Top.position(&bounds)));
    let knob = pos(view.to_screen(rotate_handle(&bounds, view.scale())));
    painter.line_segment([top, knob], stroke);
    painter.circle_stroke(knob, 5.0, stroke);
}

fn paint_gesture(interaction: &Interaction, ui: &egui::Ui, painter: &egui::Painter, view: &View) {
    if let Some((rect, mode)) = interaction.marquee() {
        let r =
            egui::Rect::from_two_pos(pos(view.to_screen(rect.min)), pos(view.to_screen(rect.max)));
        match mode {
            BoxMode::Contained => {
                painter.rect_filled(r, 0.0, SHAPE.gamma_multiply(0.15));
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, SHAPE), StrokeKind::Middle);
            }
            BoxMode::Touching => {
                painter.rect_filled(r, 0.0, DRAFT.gamma_multiply(0.15));
                let pts = [
                    r.left_top(),
                    r.right_top(),
                    r.right_bottom(),
                    r.left_bottom(),
                    r.left_top(),
                ];
                painter.extend(egui::Shape::dashed_line(
                    &pts,
                    Stroke::new(1.0, DRAFT),
                    5.0,
                    4.0,
                ));
            }
        }
    }
    let mods = ui.input(|i| Mods {
        shift: i.modifiers.shift,
        ctrl: i.modifiers.command || i.modifiers.ctrl,
    });
    let stroke = Stroke::new(1.5, DRAFT);
    match interaction.draft(mods) {
        Some(Draft::Rect(r)) => {
            let r =
                egui::Rect::from_two_pos(pos(view.to_screen(r.min)), pos(view.to_screen(r.max)));
            painter.rect_stroke(r, 0.0, stroke, StrokeKind::Middle);
        }
        Some(Draft::Ellipse(r)) => {
            let c = r.center();
            let points = flatten::ellipse_points(
                r.width() / 2.0,
                r.height() / 2.0,
                FLATTEN_PX / view.scale(),
            )
            .into_iter()
            .map(|p| pos(view.to_screen(p + c)))
            .collect();
            painter.add(egui::Shape::closed_line(points, stroke));
        }
        Some(Draft::Polyline(points)) => {
            let points = points.into_iter().map(|p| pos(view.to_screen(p))).collect();
            painter.add(egui::Shape::line(points, stroke));
        }
        None => {}
    }
}

fn paint_grid(painter: &egui::Painter, view: &View, bed: &Rect, color: Color32) {
    let minor_visible = GRID_MM * view.scale() >= MIN_GRID_PX;
    let lines = |extent: f64| {
        // Clamped below, so the cast cannot overflow.
        (extent / GRID_MM).floor().clamp(0.0, 100_000.0) as usize
    };
    for (axis, count) in [(0, lines(bed.width())), (1, lines(bed.height()))] {
        for i in 1..=count {
            let at = i as f64 * GRID_MM;
            let major = (at / MAJOR_GRID_MM).fract() == 0.0;
            if !major && !minor_visible {
                continue;
            }
            let (a, b) = if axis == 0 {
                (Point::new(at, bed.min.y), Point::new(at, bed.max.y))
            } else {
                (Point::new(bed.min.x, at), Point::new(bed.max.x, at))
            };
            let alpha = if major { 0.5 } else { 0.2 };
            painter.line_segment(
                [pos(view.to_screen(a)), pos(view.to_screen(b))],
                Stroke::new(1.0, color.gamma_multiply(alpha)),
            );
        }
    }
}
