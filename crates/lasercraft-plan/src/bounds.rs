//! The work-area check every job must pass before it is sent
//! (`docs/spec/safety.md` rule 5, `docs/spec/planning.md`).

use std::fmt;

use lasercraft_geom::Point;
use lasercraft_project::{Color, WorkArea};

use crate::{Move, MoveKind, Plan};

/// Points this close outside the bed still count as inside, mm.
const SLACK: f64 = 1e-9;

/// A bed edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// x below 0.
    Left,
    /// x above the width.
    Right,
    /// y below 0 (document y grows downwards).
    Top,
    /// y above the height.
    Bottom,
}

impl Edge {
    /// The axis that crosses this edge.
    #[must_use]
    pub const fn axis(self) -> char {
        match self {
            Self::Left | Self::Right => 'x',
            Self::Top | Self::Bottom => 'y',
        }
    }

    /// Lower-case name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// A job that leaves the work area.
#[derive(Debug, Clone, PartialEq)]
pub struct OutOfBounds {
    /// The edge crossed.
    pub edge: Edge,
    /// How far past it, mm.
    pub distance: f64,
    /// The point furthest out.
    pub point: Point,
    /// What kind of move reaches it.
    pub kind: MoveKind,
    /// Layer of the move.
    pub layer: Color,
    /// Shape the move cuts or leads to.
    pub shape: Option<u32>,
}

impl fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.shape {
            Some(id) => format!("shape {id} on layer {}", self.layer),
            None => format!("layer {}", self.layer),
        };
        let how = match self.kind {
            MoveKind::Burn => "",
            MoveKind::Unlit => " (laser-off run-on: overscan or a tab)",
            MoveKind::Travel => " (travel move)",
        };
        let value = match self.edge.axis() {
            'x' => self.point.x,
            _ => self.point.y,
        };
        write!(
            f,
            "{what} goes {:.3} mm past the {} edge of the work area ({} = {:.3} mm){how}",
            self.distance,
            self.edge.as_str(),
            self.edge.axis(),
            value
        )
    }
}

impl std::error::Error for OutOfBounds {}

/// How far `p` lies past each edge, the worst one first; with `x_only`,
/// the left and right edges only.
fn excess(p: Point, area: &WorkArea, x_only: bool) -> Option<(Edge, f64)> {
    [
        (Edge::Left, -p.x),
        (Edge::Right, p.x - area.width_mm),
        (Edge::Top, -p.y),
        (Edge::Bottom, p.y - area.height_mm),
    ]
    .into_iter()
    .filter(|(e, d)| *d > SLACK && (!x_only || e.axis() == 'x'))
    .fold(None, |best: Option<(Edge, f64)>, (e, d)| match best {
        Some((_, bd)) if bd >= d => best,
        _ => Some((e, d)),
    })
}

fn worst(m: &Move, area: &WorkArea, x_only: bool) -> Option<OutOfBounds> {
    [m.from, m.to]
        .into_iter()
        .filter_map(|p| excess(p, area, x_only).map(|(edge, distance)| (p, edge, distance)))
        .fold(
            None,
            |best: Option<OutOfBounds>, (point, edge, distance)| match best {
                Some(b) if b.distance >= distance => Some(b),
                _ => Some(OutOfBounds {
                    edge,
                    distance,
                    point,
                    kind: m.kind,
                    layer: m.layer,
                    shape: m.shape,
                }),
            },
        )
}

impl Plan {
    /// Checks every move against the bed (0, 0)–(width, height).
    ///
    /// # Errors
    ///
    /// The point furthest outside, naming the edge, distance, shape and
    /// layer. On a tie the first cutting (burn or unlit) move wins over the
    /// travel move that leads to it, then the earliest move.
    pub fn check_bounds(&self, area: &WorkArea) -> Result<(), OutOfBounds> {
        self.check_bounds_on(area, false)
    }

    /// As [`Plan::check_bounds`], the left and right edges only: on a rotary
    /// Y is a rotation (`docs/spec/rotary.md`).
    ///
    /// # Errors
    ///
    /// As [`Plan::check_bounds`].
    pub fn check_bounds_x(&self, area: &WorkArea) -> Result<(), OutOfBounds> {
        self.check_bounds_on(area, true)
    }

    fn check_bounds_on(&self, area: &WorkArea, x_only: bool) -> Result<(), OutOfBounds> {
        let rank = |o: &OutOfBounds| (o.distance, o.kind != MoveKind::Travel);
        let found = self
            .moves
            .iter()
            .filter_map(|m| worst(m, area, x_only))
            .fold(None, |best: Option<OutOfBounds>, o| match best {
                Some(b) if rank(&b) >= rank(&o) => Some(b),
                _ => Some(o),
            });
        found.map_or(Ok(()), Err)
    }
}
