//! Builds the move list, tracking where the head is.

use lasercraft_geom::Point;
use lasercraft_project::Color;

use crate::{Move, MoveKind};

/// Moves shorter than this are dropped, mm.
const MIN_MOVE: f64 = 1e-9;

/// The feed settings of the layer pass being planned.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Feed {
    pub layer: Color,
    pub pass: u32,
    pub speed: f64,
    pub power: f64,
}

pub(crate) struct Emitter {
    pub moves: Vec<Move>,
    pub pos: Point,
    travel_speed: f64,
}

impl Emitter {
    pub fn new(start: Point, travel_speed: f64) -> Self {
        Self {
            moves: Vec::new(),
            pos: start,
            travel_speed,
        }
    }

    fn push(
        &mut self,
        kind: MoveKind,
        to: Point,
        speed: f64,
        power: f64,
        feed: &Feed,
        shape: Option<u32>,
    ) {
        if self.pos.distance(to) < MIN_MOVE {
            return;
        }
        self.moves.push(Move {
            kind,
            from: self.pos,
            to,
            speed,
            power,
            layer: feed.layer,
            pass: feed.pass,
            shape,
            time: 0.0,
        });
        self.pos = to;
    }

    /// Laser off at the machine's top speed.
    pub fn travel(&mut self, to: Point, feed: &Feed, shape: Option<u32>) {
        self.push(MoveKind::Travel, to, self.travel_speed, 0.0, feed, shape);
    }

    /// Laser on at `power` (images: grayscale levels), at the feed speed.
    pub fn burn(&mut self, to: Point, power: f64, feed: &Feed, shape: u32) {
        self.push(MoveKind::Burn, to, feed.speed, power, feed, Some(shape));
    }

    /// Along the toolpath at the feed speed, laser on or off.
    pub fn feed(&mut self, lit: bool, to: Point, feed: &Feed, shape: u32) {
        let (kind, power) = if lit {
            (MoveKind::Burn, feed.power)
        } else {
            (MoveKind::Unlit, 0.0)
        };
        self.push(kind, to, feed.speed, power, feed, Some(shape));
    }
}
