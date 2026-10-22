//! The canvas: draws the bed, grid and shapes, and feeds pointer input to
//! [`Interaction`] (`docs/spec/canvas.md`).

use egui::{Pos2, Sense, Stroke, StrokeKind};
use lasercraft_commands::Invocation;
use lasercraft_geom::manipulate::Handle;
use lasercraft_geom::{Affine, Point, Rect, flatten, nodes};
use lasercraft_project::{BoxMode, Geometry, Origin, Project};

use crate::colors::layer_color;
use crate::interaction::{
    Ctx, Draft, GRID_MM, Interaction, Mods, NodeKey, Tool, moved_part, rotate_handle,
};
use crate::textures::Textures;
use crate::theme::{DASH, Tokens, size, weight};
use crate::view::{View, ZOOM_STEP};

/// Major grid lines every this many millimetres.
const MAJOR_GRID_MM: f64 = 50.0;
/// Minor grid lines closer than this many pixels are hidden.
const MIN_GRID_PX: f64 = 6.0;
/// Margin around the bed when fitting, in pixels.
const FIT_MARGIN_PX: f64 = 24.0;
/// Outline flattening tolerance on screen, in pixels.
const FLATTEN_PX: f64 = 0.3;
/// Hover pick tolerance, pixels (as clicking).
const HIT_PX: f64 = 5.0;
/// Scroll pixels per factor-of-e zoom.
const SCROLL_PER_E: f64 = 400.0;

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
    /// The screen and bed the view was last fitted to, while the user has
    /// not zoomed or panned since; the view then follows size changes.
    fitted: Option<(Rect, Rect)>,
    interaction: Interaction,
    /// The primary button went down on the canvas (so its release is ours).
    pressed_here: bool,
    screen: Option<Rect>,
    /// Image textures (M7).
    textures: Textures,
}

impl Canvas {
    /// The shape the Select tool would pick under the pointer, drawn lifted
    /// (`docs/spec/canvas.md`, "View"); none while a gesture runs.
    fn hovered(&self, project: &Project, view: &View) -> Option<u32> {
        if self.tool() != Tool::Select || self.interaction.busy() {
            return None;
        }
        let p = self.interaction.pointer()?;
        project.shape_at(p, HIT_PX / view.scale())
    }

    /// Image textures held for drawing.
    #[must_use]
    pub fn image_textures(&self) -> usize {
        self.textures.count()
    }

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

    /// Takes the point where the text tool was clicked, if any.
    pub fn take_text_click(&mut self) -> Option<Point> {
        self.interaction.take_text_click()
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
            ("view.zoom_in", Some(v)) => {
                v.zoom_about(screen.center(), ZOOM_STEP);
                self.fitted = None;
            }
            ("view.zoom_out", Some(v)) => {
                v.zoom_about(screen.center(), 1.0 / ZOOM_STEP);
                self.fitted = None;
            }
            _ => self.fit_requested = true,
        }
    }

    /// Draws the canvas and returns the commands the user's gestures produced.
    pub fn show(&mut self, ui: &mut egui::Ui, project: &Project) -> Vec<Invocation> {
        self.textures.update(ui.ctx(), project);
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let screen = rect_of(response.rect);
        self.screen = Some(screen);
        let area = project.work_area();
        let bed = Rect::from_corners(Point::ZERO, Point::new(area.width_mm, area.height_mm));
        let moved = self.fitted.is_some_and(|fitted| fitted != (screen, bed));
        if self.view.is_none() || self.fit_requested || moved {
            self.view = Some(View::fit(&screen, &bed, FIT_MARGIN_PX));
            self.fit_requested = false;
            self.fitted = Some((screen, bed));
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
                self.fitted = None;
            }
        }
        let panning = middle || (space && down);
        if panning && (hovered || self.pressed_here) {
            if delta != egui::Vec2::ZERO {
                view.pan(point(Pos2::ZERO + delta));
                self.fitted = None;
            }
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
        if self.tool() == Tool::Nodes && hovered && !ui.ctx().text_edit_focused() {
            out.extend(
                node_keys(ui)
                    .into_iter()
                    .flat_map(|k| self.interaction.node_key(k, &ctx)),
            );
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
        let t = Tokens::of(ui.ctx());
        painter.rect_filled(painter.clip_rect(), 0.0, t.canvas);
        let bed_screen =
            egui::Rect::from_two_pos(pos(view.to_screen(bed.min)), pos(view.to_screen(bed.max)));
        painter.rect_filled(bed_screen, 0.0, t.bed);
        paint_grid(painter, view, bed, &t);
        painter.rect_stroke(
            bed_screen,
            0.0,
            Stroke::new(weight::THIN, t.bed_edge),
            StrokeKind::Middle,
        );
        paint_origin(ui, painter, bed_screen, project.work_area().origin, &t);
        let hovered = self.hovered(project, view);

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
            let mut color = layer_color(shape.layer());
            if !layer.output {
                color = color.gamma_multiply(0.3);
            }
            let lifted = project.is_selected(shape.id()) || hovered == Some(shape.id());
            let width = if lifted {
                weight::HEAVY
            } else {
                weight::NORMAL
            };
            let stroke = Stroke::new(width, color);
            let edited = self
                .interaction
                .part_drag()
                .filter(|(id, ..)| *id == shape.id())
                .and_then(|(id, part, to)| moved_part(project, id, part, to));
            let place = |p: Point| pos(view.to_screen(moved.map_or(p, |t| t.apply(p))));
            if let (Some(image), Some(texture)) =
                (shape.geometry().image(), self.textures.get(shape.id()))
            {
                let xf = shape.transform();
                let corners = [
                    Point::new(0.0, 0.0),
                    Point::new(image.width, 0.0),
                    Point::new(image.width, image.height),
                    Point::new(0.0, image.height),
                ]
                .map(|p| place(xf.apply(p)));
                // White leaves the texture's own grays as they are.
                let tint = if layer.output {
                    t.handle_fill
                } else {
                    t.handle_fill.gamma_multiply(0.4)
                };
                let mut mesh = egui::Mesh::with_texture(texture);
                for (corner, uv) in corners.iter().zip([
                    Pos2::new(0.0, 0.0),
                    Pos2::new(1.0, 0.0),
                    Pos2::new(1.0, 1.0),
                    Pos2::new(0.0, 1.0),
                ]) {
                    mesh.vertices.push(egui::epaint::Vertex {
                        pos: *corner,
                        uv,
                        color: tint,
                    });
                }
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                painter.add(egui::Shape::mesh(mesh));
                painter.add(egui::Shape::closed_line(
                    corners.to_vec(),
                    Stroke::new(1.0, color),
                ));
            }
            let outlines =
                edited.map_or_else(|| shape.outlines(tolerance), |p| p.flatten(tolerance));
            for (points, closed) in outlines {
                let screen: Vec<Pos2> = points.iter().map(|p| place(*p)).collect();
                painter.add(if closed {
                    egui::Shape::closed_line(screen, stroke)
                } else {
                    egui::Shape::line(screen, stroke)
                });
            }
        }
        if self.tool() == Tool::Nodes {
            paint_nodes(painter, view, project, &self.interaction, &t);
        } else {
            paint_selection(painter, view, project, preview.map(|(_, a)| a), &t);
        }
        paint_gesture(&self.interaction, ui, painter, view, &t);
    }
}

/// Node-tool keys pressed this frame (no modifiers).
fn node_keys(ui: &egui::Ui) -> Vec<NodeKey> {
    ui.input(|i| {
        if i.modifiers.any() {
            return Vec::new();
        }
        [
            (egui::Key::D, NodeKey::Delete),
            (egui::Key::I, NodeKey::Insert),
            (egui::Key::M, NodeKey::Midpoint),
            (egui::Key::L, NodeKey::Line),
            (egui::Key::C, NodeKey::Curve),
            (egui::Key::B, NodeKey::Break),
        ]
        .into_iter()
        .filter(|(key, _)| i.key_pressed(*key))
        .map(|(_, k)| k)
        .collect()
    })
}

/// Nodes (squares, subpath starts filled) and handles (circles on a line
/// to their node) of the selected paths, with the dragged edit applied.
fn paint_nodes(
    painter: &egui::Painter,
    view: &View,
    project: &Project,
    interaction: &Interaction,
    t: &Tokens,
) {
    let outline = Stroke::new(weight::THIN, t.selection);
    let dragged = interaction.part_drag();
    for id in project.selection() {
        let Some(shape) = project.shape(id) else {
            continue;
        };
        let Geometry::Path(local) = shape.geometry() else {
            continue;
        };
        let moving = dragged.filter(|(d, ..)| *d == id);
        let path = moving
            .and_then(|(d, part, to)| moved_part(project, d, part, to))
            .unwrap_or_else(|| local.transformed(shape.transform()));
        for (_, _, _, handle, node) in nodes::handles(&path) {
            let (h, n) = (pos(view.to_screen(handle)), pos(view.to_screen(node)));
            painter.line_segment([n, h], Stroke::new(weight::HAIRLINE, t.text_muted));
            painter.circle_filled(h, 3.0, t.handle_fill);
            painter.circle_stroke(h, 3.0, outline);
        }
        for (_, node, at) in nodes::nodes(&path) {
            let square =
                egui::Rect::from_center_size(pos(view.to_screen(at)), egui::vec2(7.0, 7.0));
            if node == 0 {
                // The start of a subpath.
                let big = square.expand(1.0);
                painter.rect_filled(big, 0.0, t.start_marker);
            } else {
                painter.rect_filled(square, 0.0, t.node);
                painter.rect_stroke(
                    square,
                    0.0,
                    Stroke::new(weight::THIN, t.text),
                    StrokeKind::Middle,
                );
            }
        }
    }
}

fn paint_selection(
    painter: &egui::Painter,
    view: &View,
    project: &Project,
    preview: Option<Affine>,
    t: &Tokens,
) {
    let selection = project.selection();
    let Ok(Some(bounds)) = project.bounds_of(&selection) else {
        return;
    };
    let bounds = preview.map_or(bounds, |a| bounds.transformed(&a));
    let stroke = Stroke::new(weight::NORMAL, t.selection);
    let a = pos(view.to_screen(bounds.min));
    let b = pos(view.to_screen(bounds.max));
    let box_points = [a, Pos2::new(b.x, a.y), b, Pos2::new(a.x, b.y), a];
    painter.extend(egui::Shape::dashed_line(
        &box_points,
        stroke,
        DASH.0,
        DASH.1,
    ));
    if preview.is_some() {
        // Handles hide while the selection is being dragged.
        return;
    }
    for handle in Handle::ALL {
        let c = pos(view.to_screen(handle.position(&bounds)));
        let square = egui::Rect::from_center_size(c, egui::vec2(8.0, 8.0));
        painter.rect_filled(square, 0.0, t.handle_fill);
        painter.rect_stroke(square, 0.0, stroke, StrokeKind::Middle);
    }
    let top = pos(view.to_screen(Handle::Top.position(&bounds)));
    let knob = pos(view.to_screen(rotate_handle(&bounds, view.scale())));
    painter.line_segment([top, knob], Stroke::new(weight::HAIRLINE, t.selection));
    painter.circle_filled(knob, 5.0, t.handle_fill);
    painter.circle_stroke(knob, 5.0, stroke);
}

/// The machine-origin marker: a filled square on the corner and two arms
/// along the bed edges pointing into the bed, with X and Y at their tips.
pub(crate) fn paint_origin(
    ui: &egui::Ui,
    painter: &egui::Painter,
    bed: egui::Rect,
    origin: Origin,
    t: &Tokens,
) {
    let (corner, dx, dy) = match origin {
        Origin::FrontLeft => (bed.left_bottom(), 1.0, -1.0),
        Origin::FrontRight => (bed.right_bottom(), -1.0, -1.0),
        Origin::RearLeft => (bed.left_top(), 1.0, 1.0),
        Origin::RearRight => (bed.right_top(), -1.0, 1.0),
    };
    let square = size::ICON_SMALL * 0.6;
    let inward = egui::vec2(dx * square / 2.0, dy * square / 2.0);
    painter.rect_filled(
        egui::Rect::from_center_size(corner + inward, egui::Vec2::splat(square)),
        0.0,
        t.origin_marker,
    );
    let arm = size::ICON_SMALL * 2.0;
    if bed.width() < arm * 3.0 || bed.height() < arm * 3.0 {
        return;
    }
    let stroke = Stroke::new(weight::HEAVY, t.origin_marker);
    let font = egui::TextStyle::Small.resolve(ui.style());
    for (dir, letter) in [(egui::vec2(dx, 0.0), "X"), (egui::vec2(0.0, dy), "Y")] {
        let tip = corner + dir * arm;
        painter.line_segment([corner, tip], stroke);
        let side = egui::vec2(-dir.y, dir.x) * 3.5;
        painter.add(egui::Shape::convex_polygon(
            vec![tip + dir * 5.0, tip + side, tip - side],
            t.origin_marker,
            Stroke::NONE,
        ));
        // Letters sit inside the bed, beside the arm's tip.
        let offset = dir * 12.0 + egui::vec2(dx * dir.y.abs(), dy * dir.x.abs()) * 9.0;
        painter.text(
            tip + offset,
            egui::Align2::CENTER_CENTER,
            letter,
            font.clone(),
            t.origin_marker,
        );
    }
}

fn paint_gesture(
    interaction: &Interaction,
    ui: &egui::Ui,
    painter: &egui::Painter,
    view: &View,
    t: &Tokens,
) {
    if let Some((rect, mode)) = interaction.marquee() {
        let r =
            egui::Rect::from_two_pos(pos(view.to_screen(rect.min)), pos(view.to_screen(rect.max)));
        painter.rect_filled(r, 0.0, t.marquee.gamma_multiply(0.08));
        let stroke = Stroke::new(weight::NORMAL, t.marquee);
        match mode {
            BoxMode::Contained => {
                painter.rect_stroke(r, 0.0, stroke, StrokeKind::Middle);
            }
            BoxMode::Touching => {
                let pts = [
                    r.left_top(),
                    r.right_top(),
                    r.right_bottom(),
                    r.left_bottom(),
                    r.left_top(),
                ];
                painter.extend(egui::Shape::dashed_line(&pts, stroke, DASH.0, DASH.1));
            }
        }
    }
    let mods = ui.input(|i| Mods {
        shift: i.modifiers.shift,
        ctrl: i.modifiers.command || i.modifiers.ctrl,
    });
    let stroke = Stroke::new(weight::NORMAL, t.drawing);
    let dashed = |points: Vec<Pos2>| egui::Shape::dashed_line(&points, stroke, DASH.0, DASH.1);
    match interaction.draft(mods) {
        Some(Draft::Rect(r)) => {
            let r =
                egui::Rect::from_two_pos(pos(view.to_screen(r.min)), pos(view.to_screen(r.max)));
            let pts = vec![
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ];
            painter.extend(dashed(pts));
        }
        Some(Draft::Ellipse(r)) => {
            let c = r.center();
            let mut points: Vec<Pos2> = flatten::ellipse_points(
                r.width() / 2.0,
                r.height() / 2.0,
                FLATTEN_PX / view.scale(),
            )
            .into_iter()
            .map(|p| pos(view.to_screen(p + c)))
            .collect();
            if let Some(first) = points.first().copied() {
                points.push(first);
            }
            painter.extend(dashed(points));
        }
        Some(Draft::Polyline(points)) => {
            let points: Vec<Pos2> = points.into_iter().map(|p| pos(view.to_screen(p))).collect();
            painter.extend(dashed(points.clone()));
            for (i, p) in points.iter().enumerate() {
                let square = egui::Rect::from_center_size(*p, egui::vec2(6.0, 6.0));
                let fill = if i == 0 { t.start_marker } else { t.node };
                painter.rect_filled(square, 0.0, fill);
                if i > 0 {
                    painter.rect_stroke(
                        square,
                        0.0,
                        Stroke::new(weight::THIN, t.text),
                        StrokeKind::Middle,
                    );
                }
            }
        }
        None => {}
    }
}

pub(crate) fn paint_grid(painter: &egui::Painter, view: &View, bed: &Rect, t: &Tokens) {
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
            let color = if major { t.grid_major } else { t.grid_minor };
            painter.line_segment(
                [pos(view.to_screen(a)), pos(view.to_screen(b))],
                Stroke::new(weight::HAIRLINE, color),
            );
        }
    }
}
