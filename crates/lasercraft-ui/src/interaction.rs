//! Canvas gestures as a state machine over plain data (`docs/spec/canvas.md`).
//!
//! The egui layer feeds pointer positions in document millimetres; this module
//! decides what the gesture means and returns invocations. It never changes
//! the project itself.

use lasercraft_commands::Invocation;
use lasercraft_geom::manipulate::{Handle, Scale, rotation_degrees, scale_for_handle};
use lasercraft_geom::snap::{self, Snap};
use lasercraft_geom::{Affine, Point, Rect};
use lasercraft_project::{BoxMode, Project, snap_points};

/// Grid spacing in millimetres.
pub const GRID_MM: f64 = 10.0;
/// Screen distances, in pixels.
const HIT_PX: f64 = 5.0;
const HANDLE_PX: f64 = 6.0;
/// How far the rotate handle sits above the selection box.
pub const ROTATE_OFFSET_PX: f64 = 24.0;
const SNAP_PX: f64 = 8.0;
const CLICK_PX: f64 = 3.0;
/// Smallest rectangle or ellipse side that gets drawn, in mm.
const MIN_DRAW_MM: f64 = 0.5;
/// Rotation snap step in degrees.
const ROTATE_STEP: f64 = 15.0;

/// Modifier keys during a gesture.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mods {
    /// Shift.
    pub shift: bool,
    /// Ctrl (Cmd on macOS).
    pub ctrl: bool,
}

/// The active drawing tool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    /// Select, move, scale and rotate.
    #[default]
    Select,
    /// Draw rectangles.
    Rect,
    /// Draw ellipses.
    Ellipse,
    /// Draw lines and polylines.
    Line,
}

impl Tool {
    /// Parses the argument of `ui.tool`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "select" => Self::Select,
            "rect" => Self::Rect,
            "ellipse" => Self::Ellipse,
            "line" => Self::Line,
            _ => return None,
        })
    }
}

/// What the pointer is doing with the primary button held.
#[derive(Debug, Clone, PartialEq)]
enum Drag {
    Idle,
    /// Pressed in the select tool; a click or a move, decided on release or
    /// once the pointer travels.
    Pending {
        start: Point,
        hit: Option<u32>,
        move_ids: Vec<u32>,
    },
    Move {
        start: Point,
        ids: Vec<u32>,
        bounds: Rect,
        delta: Point,
    },
    Scale {
        handle: Handle,
        bounds: Rect,
        ids: Vec<u32>,
        scale: Option<Scale>,
    },
    Rotate {
        center: Point,
        start: Point,
        ids: Vec<u32>,
        degrees: f64,
    },
    Marquee {
        start: Point,
        current: Point,
    },
    Draw {
        start: Point,
        current: Point,
    },
}

/// Something under construction, for drawing.
#[derive(Debug, Clone, PartialEq)]
pub enum Draft {
    /// A rectangle being dragged out.
    Rect(Rect),
    /// An ellipse being dragged out, inscribed in this box.
    Ellipse(Rect),
    /// A polyline being clicked out, ending at the pointer.
    Polyline(Vec<Point>),
}

/// Context for one pointer event.
pub struct Ctx<'a> {
    /// The project, read-only.
    pub project: &'a Project,
    /// Current zoom.
    pub px_per_mm: f64,
}

impl Ctx<'_> {
    fn mm(&self, px: f64) -> f64 {
        px / self.px_per_mm.max(f64::EPSILON)
    }

    fn snap(&self, mods: Mods) -> Option<Snap> {
        (!mods.ctrl).then(|| Snap {
            grid: Some(GRID_MM),
            radius: self.mm(SNAP_PX),
        })
    }

    /// Snap targets: the bounds points of every shape not in `moving`.
    fn targets(&self, moving: &[u32]) -> Vec<Point> {
        self.project
            .shapes()
            .iter()
            .filter(|s| !moving.contains(&s.id()))
            .flat_map(|s| snap_points(&s.bounds()))
            .collect()
    }
}

/// Canvas gesture state.
#[derive(Debug, Clone)]
pub struct Interaction {
    tool: Tool,
    drag: Drag,
    polyline: Vec<Point>,
    pointer: Option<Point>,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            drag: Drag::Idle,
            polyline: Vec::new(),
            pointer: None,
        }
    }
}

fn num(v: f64) -> String {
    // Shortest exact form; `-0` reads badly in logs.
    if v == 0.0 {
        "0".to_owned()
    } else {
        v.to_string()
    }
}

fn inv(name: &str, numbers: &[f64], ids: &[u32]) -> Invocation {
    Invocation::new(
        name,
        numbers
            .iter()
            .map(|v| num(*v))
            .chain(ids.iter().map(ToString::to_string)),
    )
}

/// The rotate handle's position for `bounds` at this zoom.
#[must_use]
pub fn rotate_handle(bounds: &Rect, px_per_mm: f64) -> Point {
    let top = Handle::Top.position(bounds);
    Point::new(
        top.x,
        top.y - ROTATE_OFFSET_PX / px_per_mm.max(f64::EPSILON),
    )
}

impl Interaction {
    /// The active tool.
    #[must_use]
    pub fn tool(&self) -> Tool {
        self.tool
    }

    /// Switches tool, abandoning any gesture in progress.
    pub fn set_tool(&mut self, tool: Tool) {
        self.cancel();
        self.tool = tool;
    }

    /// Abandons the gesture in progress and any unfinished line.
    pub fn cancel(&mut self) {
        self.drag = Drag::Idle;
        self.polyline.clear();
    }

    /// Whether a gesture or unfinished line is in progress.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.drag != Drag::Idle || !self.polyline.is_empty()
    }

    /// Last known pointer position in mm, if over the canvas.
    #[must_use]
    pub fn pointer(&self) -> Option<Point> {
        self.pointer
    }

    /// Records the pointer position (or that it left the canvas).
    pub fn hover(&mut self, p: Option<Point>) {
        self.pointer = p;
    }

    /// The transform to preview on `ids` while a move, scale or rotation is
    /// in progress.
    #[must_use]
    pub fn preview(&self) -> Option<(&[u32], Affine)> {
        match &self.drag {
            Drag::Move { ids, delta, .. } => Some((ids, Affine::translate(delta.x, delta.y))),
            Drag::Scale {
                ids,
                scale: Some(s),
                ..
            } => Some((ids, scale_affine(s))),
            Drag::Rotate {
                ids,
                center,
                degrees,
                ..
            } => Some((ids, Affine::rotate_about(*degrees, *center))),
            _ => None,
        }
    }

    /// The selection box being dragged, and how it selects.
    #[must_use]
    pub fn marquee(&self) -> Option<(Rect, BoxMode)> {
        match &self.drag {
            Drag::Marquee { start, current } => {
                // Left to right selects contained shapes; right to left
                // selects touched ones.
                let mode = if current.x >= start.x {
                    BoxMode::Contained
                } else {
                    BoxMode::Touching
                };
                Some((Rect::from_corners(*start, *current), mode))
            }
            _ => None,
        }
    }

    /// The shape being drawn, for display.
    #[must_use]
    pub fn draft(&self, mods: Mods) -> Option<Draft> {
        match (&self.drag, self.tool) {
            (Drag::Draw { start, current }, Tool::Rect) => {
                Some(Draft::Rect(draw_box(*start, *current, mods)))
            }
            (Drag::Draw { start, current }, Tool::Ellipse) => {
                Some(Draft::Ellipse(draw_box(*start, *current, mods)))
            }
            _ if !self.polyline.is_empty() => {
                let mut points = self.polyline.clone();
                points.extend(self.pointer);
                Some(Draft::Polyline(points))
            }
            _ => None,
        }
    }

    /// Primary button pressed at `p`.
    pub fn press(&mut self, p: Point, mods: Mods, ctx: &Ctx<'_>) -> Vec<Invocation> {
        self.pointer = Some(p);
        match self.tool {
            Tool::Select => self.press_select(p, mods, ctx),
            Tool::Rect | Tool::Ellipse => {
                let start = snap_free(p, ctx.snap(Mods::default()), &ctx.targets(&[]));
                self.drag = Drag::Draw {
                    start,
                    current: start,
                };
                Vec::new()
            }
            Tool::Line => self.press_line(p, ctx),
        }
    }

    fn press_select(&mut self, p: Point, mods: Mods, ctx: &Ctx<'_>) -> Vec<Invocation> {
        let selection = ctx.project.selection();
        if let Some(bounds) = ctx.project.bounds_of(&selection).ok().flatten()
            && let Some(drag) = grab_handle(p, &bounds, &selection, ctx)
        {
            self.drag = drag;
            return Vec::new();
        }
        let hit = ctx.project.shape_at(p, ctx.mm(HIT_PX));
        let mut out = Vec::new();
        let move_ids = match hit {
            Some(id) if ctx.project.is_selected(id) => selection,
            // Toggled on release; a drag moves only what is already selected.
            Some(_) if mods.ctrl => selection,
            Some(id) if mods.shift => {
                out.push(Invocation::new("select.add", [id.to_string()]));
                let mut ids = selection;
                ids.push(id);
                ids
            }
            Some(id) => {
                out.push(Invocation::new("select.set", [id.to_string()]));
                vec![id]
            }
            None => Vec::new(),
        };
        self.drag = Drag::Pending {
            start: p,
            hit,
            move_ids,
        };
        out
    }

    fn press_line(&mut self, p: Point, ctx: &Ctx<'_>) -> Vec<Invocation> {
        let p = snap_free(p, ctx.snap(Mods::default()), &ctx.targets(&[]));
        if self.polyline.len() >= 3
            && self
                .polyline
                .first()
                .is_some_and(|first| first.distance(p) <= ctx.mm(HIT_PX))
        {
            return self.finish_line(true);
        }
        self.polyline.push(p);
        Vec::new()
    }

    /// Pointer moved to `p` with the primary button held.
    pub fn drag_to(&mut self, p: Point, mods: Mods, ctx: &Ctx<'_>) {
        self.pointer = Some(p);
        let drag = std::mem::replace(&mut self.drag, Drag::Idle);
        self.drag = match drag {
            Drag::Pending {
                start,
                hit,
                move_ids,
            } => {
                if start.distance(p) < ctx.mm(CLICK_PX) {
                    Drag::Pending {
                        start,
                        hit,
                        move_ids,
                    }
                } else if hit.is_some() && !move_ids.is_empty() {
                    let bounds = ctx
                        .project
                        .bounds_of(&move_ids)
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| Rect::from_corners(start, start));
                    let mut drag = Drag::Move {
                        start,
                        ids: move_ids,
                        bounds,
                        delta: Point::ZERO,
                    };
                    update_move(&mut drag, p, mods, ctx);
                    drag
                } else if hit.is_some() {
                    Drag::Idle
                } else {
                    Drag::Marquee { start, current: p }
                }
            }
            mut drag @ Drag::Move { .. } => {
                update_move(&mut drag, p, mods, ctx);
                drag
            }
            Drag::Scale {
                handle,
                bounds,
                ids,
                ..
            } => {
                let pointer = snap_free(p, ctx.snap(mods), &ctx.targets(&ids));
                let scale = scale_for_handle(&bounds, handle, pointer, !mods.shift, mods.ctrl);
                Drag::Scale {
                    handle,
                    bounds,
                    ids,
                    scale,
                }
            }
            Drag::Rotate {
                center, start, ids, ..
            } => {
                let raw = rotation_degrees(center, start, p);
                let degrees = if mods.ctrl {
                    raw
                } else {
                    snap::angle(raw, ROTATE_STEP)
                };
                Drag::Rotate {
                    center,
                    start,
                    ids,
                    degrees,
                }
            }
            Drag::Marquee { start, .. } => Drag::Marquee { start, current: p },
            Drag::Draw { start, .. } => Drag::Draw {
                start,
                current: snap_free(p, ctx.snap(Mods::default()), &ctx.targets(&[])),
            },
            Drag::Idle => Drag::Idle,
        };
    }

    /// Primary button released at `p`.
    pub fn release(&mut self, p: Point, mods: Mods, ctx: &Ctx<'_>) -> Vec<Invocation> {
        self.drag_to(p, mods, ctx);
        match std::mem::replace(&mut self.drag, Drag::Idle) {
            Drag::Pending { hit, .. } => click(hit, mods, ctx),
            Drag::Move { ids, delta, .. } => {
                if delta == Point::ZERO {
                    Vec::new()
                } else {
                    vec![inv("shape.move", &[delta.x, delta.y], &ids)]
                }
            }
            Drag::Scale {
                ids,
                scale: Some(s),
                ..
            } => {
                if (s.sx, s.sy) == (1.0, 1.0) {
                    Vec::new()
                } else {
                    vec![inv(
                        "shape.scale_about",
                        &[s.sx, s.sy, s.anchor.x, s.anchor.y],
                        &ids,
                    )]
                }
            }
            Drag::Rotate {
                ids,
                center,
                degrees,
                ..
            } => {
                if degrees == 0.0 {
                    Vec::new()
                } else {
                    vec![inv(
                        "shape.rotate_about",
                        &[degrees, center.x, center.y],
                        &ids,
                    )]
                }
            }
            Drag::Marquee { start, current } => {
                let mode = if current.x >= start.x {
                    "contained"
                } else {
                    "touching"
                };
                vec![Invocation::new(
                    "select.in_rect",
                    [
                        num(start.x),
                        num(start.y),
                        num(current.x),
                        num(current.y),
                        mode.to_owned(),
                        combine(mods).to_owned(),
                    ],
                )]
            }
            Drag::Draw { start, current } => self.finish_box(start, current, mods),
            Drag::Scale { scale: None, .. } | Drag::Idle => Vec::new(),
        }
    }

    fn finish_box(&self, start: Point, current: Point, mods: Mods) -> Vec<Invocation> {
        let b = draw_box(start, current, mods);
        if b.width() < MIN_DRAW_MM || b.height() < MIN_DRAW_MM {
            return Vec::new();
        }
        match self.tool {
            Tool::Rect => vec![inv(
                "shape.add_rect",
                &[b.min.x, b.min.y, b.width(), b.height()],
                &[],
            )],
            Tool::Ellipse => {
                let c = b.center();
                vec![inv(
                    "shape.add_ellipse",
                    &[c.x, c.y, b.width() / 2.0, b.height() / 2.0],
                    &[],
                )]
            }
            Tool::Select | Tool::Line => Vec::new(),
        }
    }

    /// Double click: finishes an open line in the line tool.
    pub fn double_click(&mut self) -> Vec<Invocation> {
        if self.tool == Tool::Line {
            self.finish_line(false)
        } else {
            Vec::new()
        }
    }

    /// Finishes the line being drawn, open or closed. Repeated points (from
    /// a double click) are dropped; fewer than two points draw nothing.
    pub fn finish_line(&mut self, closed: bool) -> Vec<Invocation> {
        let mut points = std::mem::take(&mut self.polyline);
        points.dedup_by(|a, b| a.distance(*b) < 1e-9);
        if points.len() < 2 {
            return Vec::new();
        }
        let coords: Vec<f64> = points.iter().flat_map(|p| [p.x, p.y]).collect();
        if points.len() == 2 && !closed {
            return vec![inv("shape.add_line", &coords, &[])];
        }
        let mut args: Vec<String> = Vec::new();
        if closed && points.len() >= 3 {
            args.push("closed".to_owned());
        }
        args.extend(coords.iter().map(|v| num(*v)));
        vec![Invocation::new("shape.add_polyline", args)]
    }
}

fn scale_affine(s: &Scale) -> Affine {
    Affine::translate(s.anchor.x, s.anchor.y)
        * Affine::scale(s.sx, s.sy)
        * Affine::translate(-s.anchor.x, -s.anchor.y)
}

fn snap_free(p: Point, snap: Option<Snap>, targets: &[Point]) -> Point {
    snap.map_or(p, |s| s.point(p, targets))
}

fn combine(mods: Mods) -> &'static str {
    if mods.ctrl {
        "toggle"
    } else if mods.shift {
        "add"
    } else {
        "replace"
    }
}

/// A click without dragging in the select tool.
fn click(hit: Option<u32>, mods: Mods, ctx: &Ctx<'_>) -> Vec<Invocation> {
    match hit {
        // Plain and Shift clicks on a shape were handled on press, except a
        // plain click on an already-selected shape, which selects just it.
        Some(id) if mods.ctrl => vec![Invocation::new("select.toggle", [id.to_string()])],
        Some(id) if !mods.shift && ctx.project.selection() != [id] => {
            vec![Invocation::new("select.set", [id.to_string()])]
        }
        Some(_) => Vec::new(),
        None if mods.shift || mods.ctrl || ctx.project.selection().is_empty() => Vec::new(),
        None => vec![Invocation::bare("select.clear")],
    }
}

/// A press on a scale or rotate handle of the selection, if any.
fn grab_handle(p: Point, bounds: &Rect, ids: &[u32], ctx: &Ctx<'_>) -> Option<Drag> {
    let reach = ctx.mm(HANDLE_PX);
    if rotate_handle(bounds, ctx.px_per_mm).distance(p) <= reach {
        return Some(Drag::Rotate {
            center: bounds.center(),
            start: p,
            ids: ids.to_vec(),
            degrees: 0.0,
        });
    }
    Handle::ALL
        .into_iter()
        .find(|h| h.position(bounds).distance(p) <= reach)
        .map(|handle| Drag::Scale {
            handle,
            bounds: *bounds,
            ids: ids.to_vec(),
            scale: None,
        })
}

/// Recomputes a move's delta: constrained with Shift, snapped unless Ctrl.
fn update_move(drag: &mut Drag, p: Point, mods: Mods, ctx: &Ctx<'_>) {
    let Drag::Move {
        start,
        ids,
        bounds,
        delta,
    } = drag
    else {
        return;
    };
    let mut d = p - *start;
    if mods.shift {
        d = snap::constrain_45(d);
    } else if let Some(s) = ctx.snap(mods) {
        let anchors = snap_points(bounds);
        d = s.delta(d, &anchors, &ctx.targets(ids));
    }
    *delta = d;
}

/// The box drawn from `start` to `current`: from the center with Ctrl, square
/// with Shift.
fn draw_box(start: Point, current: Point, mods: Mods) -> Rect {
    let mut d = current - start;
    if mods.shift {
        let side = d.x.abs().max(d.y.abs());
        d = Point::new(side.copysign(d.x), side.copysign(d.y));
    }
    if mods.ctrl {
        Rect::from_corners(start - d, start + d)
    } else {
        Rect::from_corners(start, start + d)
    }
}

#[cfg(test)]
mod tests {
    use lasercraft_commands::{Registry, Session};

    use super::*;

    const PLAIN: Mods = Mods {
        shift: false,
        ctrl: false,
    };
    const SHIFT: Mods = Mods {
        shift: true,
        ctrl: false,
    };
    const CTRL: Mods = Mods {
        shift: false,
        ctrl: true,
    };

    struct World {
        session: Session,
        ui: Interaction,
    }

    impl World {
        fn new(lines: &[&str]) -> Self {
            let mut session = Session::new(Project::default(), Registry::with_builtin());
            for line in lines {
                session.run_line(line).unwrap();
            }
            Self {
                session,
                ui: Interaction::default(),
            }
        }

        fn apply(&mut self, invocations: Vec<Invocation>) -> Vec<String> {
            invocations
                .into_iter()
                .map(|inv| {
                    self.session.run(&inv).unwrap();
                    inv.to_string()
                })
                .collect()
        }

        /// Press, drag through `path`, release at its end. 10 px per mm.
        fn gesture(&mut self, path: &[(f64, f64)], mods: Mods) -> Vec<String> {
            let pts: Vec<Point> = path.iter().map(|(x, y)| Point::new(*x, *y)).collect();
            let mut out = Vec::new();
            let ctx_inv = {
                let ctx = Ctx {
                    project: self.session.project(),
                    px_per_mm: 10.0,
                };
                self.ui.press(pts[0], mods, &ctx)
            };
            out.extend(self.apply(ctx_inv));
            for p in &pts[1..] {
                let ctx = Ctx {
                    project: self.session.project(),
                    px_per_mm: 10.0,
                };
                self.ui.drag_to(*p, mods, &ctx);
            }
            let last = *pts.last().unwrap();
            let released = {
                let ctx = Ctx {
                    project: self.session.project(),
                    px_per_mm: 10.0,
                };
                self.ui.release(last, mods, &ctx)
            };
            out.extend(self.apply(released));
            out
        }

        fn selection(&self) -> Vec<u32> {
            self.session.project().selection()
        }
    }

    #[test]
    fn click_selects_and_shift_click_adds_and_ctrl_click_toggles() {
        let mut w = World::new(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 50 0 10 10",
            "select.clear",
        ]);
        w.gesture(&[(5.0, 5.0)], PLAIN);
        assert_eq!(w.selection(), [1]);
        w.gesture(&[(55.0, 5.0)], SHIFT);
        assert_eq!(w.selection(), [1, 2]);
        w.gesture(&[(5.0, 5.0)], CTRL);
        assert_eq!(w.selection(), [2]);
        // Plain click on a selected shape in a multi-selection selects just it.
        w.session.run_line("select.all").unwrap();
        w.gesture(&[(55.0, 5.0)], PLAIN);
        assert_eq!(w.selection(), [2]);
        // Clicking empty space clears.
        w.gesture(&[(200.0, 200.0)], PLAIN);
        assert!(w.selection().is_empty());
    }

    #[test]
    fn dragging_a_shape_moves_it_once_with_grid_snap() {
        let mut w = World::new(&["shape.add_rect 0 0 10 10", "select.clear"]);
        let out = w.gesture(&[(5.0, 5.0), (12.0, 5.0), (24.6, 5.2)], PLAIN);
        // The box's left edge lands within 0.8 mm (8 px) of the 20 mm grid line.
        assert_eq!(out, ["select.set 1", "shape.move 20 0 1"]);
        assert_eq!(w.session.undo_depth(), 2);
        // Ctrl turns snapping off.
        let out = w.gesture(&[(25.0, 5.0), (26.3, 5.0)], CTRL);
        assert_eq!(out.last().unwrap(), "shape.move 1.3000000000000007 0 1");
    }

    #[test]
    fn shift_constrains_moves_to_45_degrees() {
        let mut w = World::new(&["shape.add_rect 0 0 10 10"]);
        let out = w.gesture(&[(5.0, 5.0), (35.0, 7.0)], SHIFT);
        assert_eq!(out.last().unwrap(), "shape.move 30 0 1");
    }

    #[test]
    fn marquee_direction_picks_the_mode() {
        let mut w = World::new(&[
            "shape.add_rect 0 0 10 10",
            "shape.add_rect 20 0 10 10",
            "select.clear",
        ]);
        let out = w.gesture(&[(-5.0, -5.0), (25.0, 15.0)], PLAIN);
        assert_eq!(out, ["select.in_rect -5 -5 25 15 contained replace"]);
        assert_eq!(w.selection(), [1]);
        w.gesture(&[(25.0, 15.0), (5.0, -5.0)], PLAIN);
        assert_eq!(w.selection(), [1, 2]);
    }

    #[test]
    fn corner_handle_scales_about_the_opposite_corner() {
        let mut w = World::new(&["shape.add_rect 0 0 20 10"]);
        // Bottom-right handle at (20, 10); drag to (40, 20) doubles it.
        let out = w.gesture(&[(20.0, 10.0), (30.0, 15.0), (40.0, 20.0)], PLAIN);
        assert_eq!(out, ["shape.scale_about 2 2 0 0 1"]);
        // Shift frees the aspect ratio; Ctrl scales about the center.
        let b = w.session.project().shape(1).unwrap().bounds();
        assert_eq!(b.max, Point::new(40.0, 20.0));
        let out = w.gesture(
            &[(40.0, 20.0), (60.0, 20.0)],
            Mods {
                shift: true,
                ctrl: true,
            },
        );
        assert_eq!(out, ["shape.scale_about 2 1 20 10 1"]);
    }

    #[test]
    fn rotate_handle_rotates_in_15_degree_steps() {
        let mut w = World::new(&["shape.add_rect 0 0 20 20"]);
        // Rotate handle sits 24 px (2.4 mm at 10 px/mm) above the top center.
        let handle = rotate_handle(
            &Rect::from_corners(Point::ZERO, Point::new(20.0, 20.0)),
            10.0,
        );
        assert_eq!(handle, Point::new(10.0, -2.4));
        // Drag from above the center to its right: a quarter turn clockwise.
        let out = w.gesture(&[(10.0, -2.4), (25.0, 9.0)], PLAIN);
        assert_eq!(out, ["shape.rotate_about 90 10 10 1"]);
    }

    #[test]
    fn rect_and_ellipse_tools_draw_snapped_boxes() {
        let mut w = World::new(&[]);
        w.ui.set_tool(Tool::Rect);
        let out = w.gesture(&[(10.3, 9.8), (40.0, 30.0), (50.2, 29.9)], PLAIN);
        assert_eq!(out, ["shape.add_rect 10 10 40 20"]);
        w.ui.set_tool(Tool::Ellipse);
        let out = w.gesture(
            &[(100.0, 100.0), (113.0, 104.0)],
            Mods {
                shift: true,
                ctrl: true,
            },
        );
        // From the center, square: radius = larger side = 13 (no grid point near).
        assert_eq!(out, ["shape.add_ellipse 100 100 13 13"]);
        // Too small: nothing.
        let out = w.gesture(&[(200.0, 200.0), (200.2, 205.0)], CTRL);
        assert!(out.is_empty());
    }

    #[test]
    fn line_tool_builds_open_and_closed_polylines() {
        let mut w = World::new(&[]);
        w.ui.set_tool(Tool::Line);
        let ctx_clicks = |w: &mut World, pts: &[(f64, f64)]| {
            let mut all = Vec::new();
            for (x, y) in pts {
                let invs = {
                    let ctx = Ctx {
                        project: w.session.project(),
                        px_per_mm: 10.0,
                    };
                    let p = Point::new(*x, *y);
                    let mut v = w.ui.press(p, PLAIN, &ctx);
                    v.extend(w.ui.release(p, PLAIN, &ctx));
                    v
                };
                all.extend(w.apply(invs));
            }
            all
        };
        ctx_clicks(&mut w, &[(0.0, 0.0), (30.0, 0.0)]);
        assert!(w.ui.busy());
        let invs = w.ui.double_click();
        assert_eq!(w.apply(invs), ["shape.add_line 0 0 30 0"]);
        // Clicking the first point again closes the shape.
        let out = ctx_clicks(
            &mut w,
            &[
                (100.0, 100.0),
                (130.0, 100.0),
                (115.0, 120.0),
                (100.2, 100.1),
            ],
        );
        assert_eq!(out, ["shape.add_polyline closed 100 100 130 100 115 120"]);
        assert!(!w.ui.busy());
        // A single point draws nothing; Esc-style cancel clears.
        ctx_clicks(&mut w, &[(5.0, 5.0)]);
        assert!(w.ui.finish_line(false).is_empty());
    }
}
