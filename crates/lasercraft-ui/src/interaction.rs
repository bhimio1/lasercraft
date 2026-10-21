//! Canvas gestures as a state machine over plain data (`docs/spec/canvas.md`).
//!
//! The egui layer feeds pointer positions in document millimetres; this module
//! decides what the gesture means and returns invocations. It never changes
//! the project itself.

use lasercraft_commands::Invocation;
use lasercraft_geom::manipulate::{Handle, Scale, rotation_degrees, scale_for_handle};
use lasercraft_geom::snap::{self, Snap};
use lasercraft_geom::{Affine, Path, Point, Rect, nodes};
use lasercraft_project::{BoxMode, Color, Geometry, Project, snap_points};

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
    /// Click to place text (`docs/spec/text.md`).
    Text,
    /// Edit path nodes and handles (`docs/spec/nodes.md`).
    Nodes,
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
            "text" => Self::Text,
            "nodes" => Self::Nodes,
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
    /// Dragging a path node or handle in the node tool.
    Part {
        id: u32,
        part: Part,
        start: Point,
        current: Point,
    },
}

/// A node or handle of a path shape (`docs/spec/nodes.md` numbering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// Node `node` of subpath `sub`.
    Node {
        /// Subpath.
        sub: usize,
        /// Node.
        node: usize,
    },
    /// Handle `which` (1 or 2) of segment `seg` of subpath `sub`.
    Handle {
        /// Subpath.
        sub: usize,
        /// Segment.
        seg: usize,
        /// 1 or 2.
        which: u8,
    },
}

/// A key the node tool acts on with the pointer over a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKey {
    /// `D`: delete the node, or the segment.
    Delete,
    /// `I`: insert a node at the pointer.
    Insert,
    /// `M`: insert a node at the segment's middle.
    Midpoint,
    /// `L`: straighten the segment.
    Line,
    /// `C`: make the segment a curve.
    Curve,
    /// `B`: break the path at the node.
    Break,
}

/// A selected path shape in document coordinates.
fn doc_paths(project: &Project) -> Vec<(u32, Path)> {
    project
        .selection()
        .into_iter()
        .filter_map(|id| project.shape(id))
        .filter_map(|s| match s.geometry() {
            Geometry::Path(p) => Some((s.id(), p.transformed(s.transform()))),
            _ => None,
        })
        .collect()
}

/// The node or handle of a selected path nearest to `p` within `reach`;
/// handles win over nodes at equal distance.
fn pick_part(project: &Project, p: Point, reach: f64) -> Option<(u32, Part, Point)> {
    let mut best: Option<(f64, u32, Part, Point)> = None;
    let mut consider = |d: f64, id: u32, part: Part, at: Point| {
        if d <= reach && best.is_none_or(|b| d < b.0) {
            best = Some((d, id, part, at));
        }
    };
    for (id, path) in doc_paths(project) {
        for (sub, seg, which, at, _) in nodes::handles(&path) {
            consider(at.distance(p), id, Part::Handle { sub, seg, which }, at);
        }
        for (sub, node, at) in nodes::nodes(&path) {
            consider(at.distance(p), id, Part::Node { sub, node }, at);
        }
    }
    best.map(|(_, id, part, at)| (id, part, at))
}

/// The path segment of a selected shape nearest to `p` within `reach`:
/// shape, subpath, segment, parameter.
fn pick_segment(project: &Project, p: Point, reach: f64) -> Option<(u32, usize, usize, f64)> {
    doc_paths(project)
        .into_iter()
        .filter_map(|(id, path)| {
            nodes::nearest_segment(&path, p).map(|(sub, seg, t, d)| (d, id, sub, seg, t))
        })
        .filter(|c| c.0 <= reach)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, sub, seg, t)| (id, sub, seg, t))
}

/// A path with one part moved to `to` (document coordinates), for preview
/// and tests. `None` when the edit does not apply.
#[must_use]
pub fn moved_part(project: &Project, id: u32, part: Part, to: Point) -> Option<Path> {
    let shape = project.shape(id)?;
    let Geometry::Path(path) = shape.geometry() else {
        return None;
    };
    let local = shape.transform().inverse()?.apply(to);
    let edited = match part {
        Part::Node { sub, node } => nodes::move_node(path, sub, node, local),
        Part::Handle { sub, seg, which } => nodes::set_handle(path, sub, seg, which, local),
    };
    edited.ok().map(|p| p.transformed(shape.transform()))
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
    /// Palette color for new shapes.
    color: Color,
    /// Where the text tool was last clicked, until the shell takes it.
    text_click: Option<Point>,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            tool: Tool::Select,
            drag: Drag::Idle,
            polyline: Vec::new(),
            pointer: None,
            color: 0,
            text_click: None,
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
    /// The palette color new shapes get.
    #[must_use]
    pub fn color(&self) -> Color {
        self.color
    }

    /// Sets the palette color for new shapes.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

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

    /// Takes the point where the text tool was clicked, if any.
    pub fn take_text_click(&mut self) -> Option<Point> {
        self.text_click.take()
    }

    /// The node or handle being dragged and where it is now, for preview.
    #[must_use]
    pub fn part_drag(&self) -> Option<(u32, Part, Point)> {
        match &self.drag {
            Drag::Part {
                id, part, current, ..
            } => Some((*id, *part, *current)),
            _ => None,
        }
    }

    /// A node-tool key pressed with the pointer over a selected path.
    pub fn node_key(&self, key: NodeKey, ctx: &Ctx<'_>) -> Vec<Invocation> {
        let Some(p) = self.pointer.filter(|_| self.tool == Tool::Nodes) else {
            return Vec::new();
        };
        let node =
            pick_part(ctx.project, p, ctx.mm(HANDLE_PX)).and_then(|(id, part, _)| match part {
                Part::Node { sub, node } => Some((id, sub, node)),
                Part::Handle { .. } => None,
            });
        let segment = pick_segment(ctx.project, p, ctx.mm(HIT_PX));
        let args = |id: u32, a: usize, b: usize| [id.to_string(), a.to_string(), b.to_string()];
        let invocation = match (key, node, segment) {
            (NodeKey::Delete, Some((id, s, n)), _) => {
                Invocation::new("node.delete", args(id, s, n))
            }
            (NodeKey::Break, Some((id, s, n)), _) => Invocation::new("node.break", args(id, s, n)),
            (NodeKey::Delete, None, Some((id, s, k, _))) => {
                Invocation::new("node.delete_segment", args(id, s, k))
            }
            (NodeKey::Insert, _, Some((id, s, k, t))) => {
                let mut a = args(id, s, k).to_vec();
                a.push(num(t));
                Invocation::new("node.insert", a)
            }
            (NodeKey::Midpoint, _, Some((id, s, k, _))) => {
                Invocation::new("node.insert", args(id, s, k))
            }
            (NodeKey::Line, _, Some((id, s, k, _))) => {
                Invocation::new("node.to_line", args(id, s, k))
            }
            (NodeKey::Curve, _, Some((id, s, k, _))) => {
                Invocation::new("node.to_curve", args(id, s, k))
            }
            _ => return Vec::new(),
        };
        vec![invocation]
    }

    /// Primary button pressed at `p`.
    pub fn press(&mut self, p: Point, mods: Mods, ctx: &Ctx<'_>) -> Vec<Invocation> {
        self.pointer = Some(p);
        match self.tool {
            Tool::Select => self.press_select(p, mods, ctx),
            Tool::Text => {
                self.text_click = Some(snap_free(p, ctx.snap(mods), &ctx.targets(&[])));
                Vec::new()
            }
            Tool::Nodes => {
                if let Some((id, part, at)) = pick_part(ctx.project, p, ctx.mm(HANDLE_PX)) {
                    self.drag = Drag::Part {
                        id,
                        part,
                        start: at,
                        current: at,
                    };
                    return Vec::new();
                }
                // Clicking selects as in the select tool, but never moves.
                let out = self.press_select(p, mods, ctx);
                if let Drag::Pending { move_ids, .. } = &mut self.drag {
                    move_ids.clear();
                } else {
                    self.drag = Drag::Idle;
                }
                out
            }
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
            Drag::Part {
                id, part, start, ..
            } => {
                let current = if mods.shift {
                    start + snap::constrain_45(p - start)
                } else {
                    snap_free(p, ctx.snap(mods), &ctx.targets(&[id]))
                };
                Drag::Part {
                    id,
                    part,
                    start,
                    current,
                }
            }
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
            Drag::Part {
                id,
                part,
                start,
                current,
            } => {
                if current == start {
                    return Vec::new();
                }
                let (name, mut args) = match part {
                    Part::Node { sub, node } => ("node.move", vec![id, sub as u32, node as u32]),
                    Part::Handle { sub, seg, which } => (
                        "node.set_handle",
                        vec![id, sub as u32, seg as u32, u32::from(which)],
                    ),
                };
                let mut text: Vec<String> = args.drain(..).map(|v| v.to_string()).collect();
                text.extend([num(current.x), num(current.y)]);
                vec![Invocation::new(name, text)]
            }
            Drag::Scale { scale: None, .. } | Drag::Idle => Vec::new(),
        }
    }

    /// A create command for `name` with numeric arguments, on the current
    /// color.
    fn create(&self, name: &str, numbers: &[f64], closed: bool) -> Invocation {
        let mut args: Vec<String> = Vec::new();
        if closed {
            args.push("closed".to_owned());
        }
        args.extend(numbers.iter().map(|v| num(*v)));
        args.push(format!("@{}", self.color));
        Invocation::new(name, args)
    }

    fn finish_box(&self, start: Point, current: Point, mods: Mods) -> Vec<Invocation> {
        let b = draw_box(start, current, mods);
        if b.width() < MIN_DRAW_MM || b.height() < MIN_DRAW_MM {
            return Vec::new();
        }
        match self.tool {
            Tool::Rect => vec![self.create(
                "shape.add_rect",
                &[b.min.x, b.min.y, b.width(), b.height()],
                false,
            )],
            Tool::Ellipse => {
                let c = b.center();
                vec![self.create(
                    "shape.add_ellipse",
                    &[c.x, c.y, b.width() / 2.0, b.height() / 2.0],
                    false,
                )]
            }
            Tool::Select | Tool::Line | Tool::Text | Tool::Nodes => Vec::new(),
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
            return vec![self.create("shape.add_line", &coords, false)];
        }
        vec![self.create("shape.add_polyline", &coords, closed && points.len() >= 3)]
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
        w.ui.set_color(4);
        let out = w.gesture(&[(10.3, 9.8), (40.0, 30.0), (50.2, 29.9)], PLAIN);
        assert_eq!(out, ["shape.add_rect 10 10 40 20 @4"]);
        w.ui.set_tool(Tool::Ellipse);
        w.ui.set_color(0);
        let out = w.gesture(
            &[(100.0, 100.0), (113.0, 104.0)],
            Mods {
                shift: true,
                ctrl: true,
            },
        );
        // From the center, square: radius = larger side = 13 (no grid point near).
        assert_eq!(out, ["shape.add_ellipse 100 100 13 13 @0"]);
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
        assert_eq!(w.apply(invs), ["shape.add_line 0 0 30 0 @0"]);
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
        assert_eq!(
            out,
            ["shape.add_polyline closed 100 100 130 100 115 120 @0"]
        );
        assert!(!w.ui.busy());
        // A single point draws nothing; Esc-style cancel clears.
        ctx_clicks(&mut w, &[(5.0, 5.0)]);
        assert!(w.ui.finish_line(false).is_empty());
    }

    #[test]
    fn text_tool_records_the_click() {
        let mut w = World::new(&[]);
        w.ui.set_tool(Tool::Text);
        assert!(w.gesture(&[(40.3, 60.2)], PLAIN).is_empty());
        assert_eq!(w.ui.take_text_click(), Some(Point::new(40.0, 60.0)));
        assert_eq!(w.ui.take_text_click(), None);
    }

    #[test]
    fn node_tool_drags_nodes_and_handles() {
        let mut w = World::new(&[
            "shape.add_rect 10 10 20 10",
            "shape.to_path",
            "node.to_curve 1 0 0",
        ]);
        w.ui.set_tool(Tool::Nodes);
        // Node 2 (bottom right, 30 20) dragged with snapping off.
        let out = w.gesture(&[(30.0, 20.0), (33.3, 24.4)], CTRL);
        assert_eq!(out, ["node.move 1 0 2 33.3 24.4"]);
        // Handle 1 of segment 0 sits a third along the top edge.
        let h = 10.0 + 20.0 / 3.0;
        let out = w.gesture(&[(h, 10.0), (h, 5.0)], SHIFT);
        assert_eq!(out.len(), 1);
        assert!(out[0].starts_with("node.set_handle 1 0 0 1 "), "{out:?}");
        // A preview applies the same edit.
        let preview = moved_part(
            w.session.project(),
            1,
            Part::Node { sub: 0, node: 0 },
            Point::new(0.0, 0.0),
        )
        .unwrap();
        assert_eq!(preview.subpaths[0].start, Point::new(0.0, 0.0));
        // Pressing on a shape away from nodes selects it but never moves it.
        w.session.run_line("select.clear").unwrap();
        let out = w.gesture(&[(20.0, 15.0), (40.0, 40.0)], PLAIN);
        assert_eq!(out, ["select.set 1"]);
    }

    #[test]
    fn node_keys_act_on_what_is_under_the_pointer() {
        let mut w = World::new(&["shape.add_rect 0 0 20 10", "shape.to_path"]);
        w.ui.set_tool(Tool::Nodes);
        let key = |w: &mut World, at: (f64, f64), key: NodeKey| {
            w.ui.hover(Some(Point::new(at.0, at.1)));
            let invs = {
                let ctx = Ctx {
                    project: w.session.project(),
                    px_per_mm: 10.0,
                };
                w.ui.node_key(key, &ctx)
            };
            w.apply(invs)
        };
        assert_eq!(
            key(&mut w, (5.0, 0.1), NodeKey::Midpoint),
            ["node.insert 1 0 0"]
        );
        assert_eq!(
            key(&mut w, (10.0, 0.0), NodeKey::Delete),
            ["node.delete 1 0 1"]
        );
        assert_eq!(
            key(&mut w, (20.0, 5.0), NodeKey::Curve),
            ["node.to_curve 1 0 1"]
        );
        assert_eq!(
            key(&mut w, (20.0, 5.0), NodeKey::Line),
            ["node.to_line 1 0 1"]
        );
        assert_eq!(
            key(&mut w, (0.0, 5.0), NodeKey::Delete),
            ["node.delete_segment 1 0 3"]
        );
        assert_eq!(
            key(&mut w, (20.0, 10.0), NodeKey::Break),
            ["node.break 1 0 2"]
        );
        let out = key(&mut w, (5.0, 10.0), NodeKey::Insert);
        // The break split the (already opened) path; the bottom edge is subpath 1.
        assert_eq!(out, ["node.insert 1 1 0 0.75"]);
        assert!(key(&mut w, (100.0, 100.0), NodeKey::Delete).is_empty());
    }
}
