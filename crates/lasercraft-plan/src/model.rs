//! The plan's data: moves, per-layer summaries, warnings and statistics.

use std::fmt;

use lasercraft_geom::{Point, Rect};
use lasercraft_project::{Color, Mode};

/// What the laser does during a move (`docs/spec/planning.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    /// Laser off, moving between cuts at the machine's top speed.
    Travel,
    /// Laser off, moving along the toolpath at the layer's speed: overscan,
    /// fill gaps and tab gaps.
    Unlit,
    /// Laser on.
    Burn,
}

impl MoveKind {
    /// Name used in dumps.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Travel => "travel",
            Self::Unlit => "unlit",
            Self::Burn => "burn",
        }
    }
}

impl fmt::Display for MoveKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One straight move.
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    /// Laser on, off along the path, or travel.
    pub kind: MoveKind,
    /// Start point, document mm.
    pub from: Point,
    /// End point, document mm.
    pub to: Point,
    /// Feed speed, mm/s (the machine's top speed for travel).
    pub speed: f64,
    /// Laser power, %; always 0 unless the move burns.
    pub power: f64,
    /// The layer the move belongs to (travel: the layer it leads into).
    pub layer: Color,
    /// Pass number, from 1.
    pub pass: u32,
    /// The shape being cut (travel: the shape it leads to).
    pub shape: Option<u32>,
    /// Estimated duration, seconds.
    pub time: f64,
}

impl Move {
    /// Length in mm.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.from.distance(self.to)
    }
}

/// What a layer contributes to the job.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSummary {
    /// Palette color.
    pub color: Color,
    /// Layer name.
    pub name: String,
    /// Line or fill.
    pub mode: Mode,
    /// Times the layer runs.
    pub passes: u32,
    /// Feed speed, mm/s.
    pub speed: f64,
    /// Burn power after clamping, %.
    pub power: f64,
    /// Overscan distance per line end, mm (0 when off or in line mode).
    pub overscan: f64,
    /// The layer has vector shapes (planned in `mode`).
    pub vectors: bool,
    /// The layer has images (planned as rows of dots).
    pub images: bool,
}

impl LayerSummary {
    /// What the layer runs: its mode, `image`, or both.
    #[must_use]
    pub fn kind(&self) -> String {
        match (self.vectors, self.images) {
            (true, true) => format!("{} + image", self.mode),
            (false, true) => "image".to_owned(),
            _ => self.mode.to_string(),
        }
    }
}

/// Something the user should know about the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// The layer concerned.
    pub layer: Color,
    /// The shape concerned, if one.
    pub shape: Option<u32>,
    /// What happened.
    pub message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.shape {
            Some(id) => write!(f, "layer {}, shape {id}: {}", self.layer, self.message),
            None => write!(f, "layer {}: {}", self.layer, self.message),
        }
    }
}

/// Totals over the plan.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stats {
    /// Distance with the laser on, mm.
    pub burn_length: f64,
    /// Distance with the laser off along the toolpath, mm.
    pub unlit_length: f64,
    /// Distance travelling between cuts, mm.
    pub travel_length: f64,
    /// Time with the laser on, s.
    pub burn_time: f64,
    /// Time of unlit moves, s.
    pub unlit_time: f64,
    /// Time travelling, s.
    pub travel_time: f64,
    /// Whole job, s.
    pub total_time: f64,
    /// Box around every move end point except the starting position
    /// (overscan and kerf included); `None` for an empty job.
    pub bounds: Option<Rect>,
}

impl Stats {
    /// Totals of `moves`.
    #[must_use]
    pub fn of(moves: &[Move]) -> Self {
        let mut s = Self::default();
        for (i, m) in moves.iter().enumerate() {
            let (length, time) = match m.kind {
                MoveKind::Burn => (&mut s.burn_length, &mut s.burn_time),
                MoveKind::Unlit => (&mut s.unlit_length, &mut s.unlit_time),
                MoveKind::Travel => (&mut s.travel_length, &mut s.travel_time),
            };
            *length += m.length();
            *time += m.time;
            s.total_time += m.time;
            // The first move starts at the head's resting position.
            for p in [m.from, m.to].into_iter().skip(usize::from(i == 0)) {
                s.bounds = Some(
                    s.bounds
                        .map_or_else(|| Rect::from_corners(p, p), |b: Rect| b.including(p)),
                );
            }
        }
        s
    }
}

/// A planned job.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// Where the head starts: the work area's origin corner.
    pub start: Point,
    /// Layers in the order they run.
    pub layers: Vec<LayerSummary>,
    /// Every move, in order.
    pub moves: Vec<Move>,
    /// Things worth telling the user.
    pub warnings: Vec<Warning>,
    /// Totals.
    pub stats: Stats,
}

impl Plan {
    /// Where the head is after `time` seconds, and how many moves have
    /// started by then (for scrubbing a preview).
    #[must_use]
    pub fn position_at(&self, time: f64) -> (Point, usize) {
        let mut elapsed = 0.0;
        for (i, m) in self.moves.iter().enumerate() {
            if time < elapsed + m.time {
                let t = if m.time > 0.0 {
                    ((time - elapsed) / m.time).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return (m.from + (m.to - m.from) * t, i + 1);
            }
            elapsed += m.time;
        }
        let end = self.moves.last().map_or(self.start, |m| m.to);
        (end, self.moves.len())
    }
}
