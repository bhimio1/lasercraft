//! Where the job runs: Start From modes and the job origin
//! (`docs/spec/laser.md`, "Start From and job origin").

use std::fmt;
use std::str::FromStr;

use lasercraft_geom::{Point, Rect};
use lasercraft_plan::{Move, MoveKind, Plan, Stats, origin_point};
use lasercraft_project::WorkArea;

use crate::ExportError;

/// One of the nine job origin points on the job's box, named as seen on
/// screen (top is the rear of the bed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// Top-left corner.
    TopLeft,
    /// Middle of the top edge.
    Top,
    /// Top-right corner.
    TopRight,
    /// Middle of the left edge.
    Left,
    /// Center.
    Center,
    /// Middle of the right edge.
    Right,
    /// Bottom-left corner (the default).
    #[default]
    BottomLeft,
    /// Middle of the bottom edge.
    Bottom,
    /// Bottom-right corner.
    BottomRight,
}

impl Anchor {
    /// All nine, row by row from the top left.
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Left,
        Self::Center,
        Self::Right,
        Self::BottomLeft,
        Self::Bottom,
        Self::BottomRight,
    ];

    /// The name used in commands and on the command line.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::Top => "top",
            Self::TopRight => "top-right",
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::BottomLeft => "bottom-left",
            Self::Bottom => "bottom",
            Self::BottomRight => "bottom-right",
        }
    }

    /// The anchor's point on `rect` (document coordinates, y down).
    #[must_use]
    pub fn point(self, rect: &Rect) -> Point {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        let fx = (i % 3) as f64 / 2.0;
        let fy = (i / 3) as f64 / 2.0;
        Point::new(
            rect.min.x + rect.width() * fx,
            rect.min.y + rect.height() * fy,
        )
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Anchor {
    type Err = ExportError;

    fn from_str(s: &str) -> Result<Self, ExportError> {
        Self::ALL
            .into_iter()
            .find(|a| a.as_str() == s)
            .ok_or_else(|| {
                ExportError::Invalid(format!(
                    "`{s}` is not a job origin (top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right)"
                ))
            })
    }
}

/// Where the job is placed on the bed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Placement {
    /// Where it is drawn.
    #[default]
    Absolute,
    /// With the job origin at a machine position.
    Relative {
        /// The head's position (Current position) or the user origin,
        /// machine mm.
        at: Point,
        /// The point of the job's box placed there.
        anchor: Anchor,
        /// Travel back to `at` at the end (Current position mode).
        return_to_start: bool,
    },
}

fn signs(area: &WorkArea) -> (f64, f64) {
    let o = origin_point(area);
    (
        if o.x == 0.0 { 1.0 } else { -1.0 },
        if o.y == 0.0 { 1.0 } else { -1.0 },
    )
}

/// Document point to machine coordinates: signed distances from the origin
/// corner, growing into the bed.
#[must_use]
pub fn to_machine(area: &WorkArea, p: Point) -> Point {
    let o = origin_point(area);
    let (sx, sy) = signs(area);
    Point::new(sx * (p.x - o.x), sy * (p.y - o.y))
}

/// Machine coordinates to a document point (the inverse of [`to_machine`]).
#[must_use]
pub fn to_document(area: &WorkArea, m: Point) -> Point {
    let o = origin_point(area);
    let (sx, sy) = signs(area);
    Point::new(o.x + sx * m.x, o.y + sy * m.y)
}

/// A document-space direction as a machine-space one.
#[must_use]
pub fn machine_delta(area: &WorkArea, d: Point) -> Point {
    let (sx, sy) = signs(area);
    Point::new(sx * d.x, sy * d.y)
}

/// The box around everything that burns, document mm.
#[must_use]
pub fn job_box(plan: &Plan) -> Option<Rect> {
    Rect::bounding(
        plan.moves
            .iter()
            .filter(|m| m.kind == MoveKind::Burn)
            .flat_map(|m| [m.from, m.to]),
    )
}

/// Shifts the plan to `placement`. The result still has to pass
/// `check_bounds`.
#[must_use]
pub fn place(mut plan: Plan, area: &WorkArea, placement: Placement, travel_speed: f64) -> Plan {
    let Placement::Relative {
        at,
        anchor,
        return_to_start,
    } = placement
    else {
        return plan;
    };
    let Some(job) = job_box(&plan) else {
        return plan;
    };
    let target = to_document(area, at);
    let delta = target - anchor.point(&job);
    for m in &mut plan.moves {
        m.from = m.from + delta;
        m.to = m.to + delta;
    }
    // The head starts at `at` (current position) or somewhere on the bed
    // (user origin); either way not at the origin corner the plan assumed.
    // This only affects the first travel's estimate.
    if let Some(first) = plan.moves.first_mut() {
        first.from = target;
    }
    plan.start = target;
    if return_to_start && let Some(last) = plan.moves.last().cloned() {
        let length = last.to.distance(target);
        plan.moves.push(Move {
            kind: MoveKind::Travel,
            from: last.to,
            to: target,
            speed: travel_speed,
            power: 0.0,
            layer: last.layer,
            pass: last.pass,
            shape: None,
            time: if travel_speed > 0.0 {
                length / travel_speed
            } else {
                0.0
            },
        });
    }
    plan.stats = Stats::of(&plan.moves);
    plan
}

#[cfg(test)]
mod tests {
    use lasercraft_project::Origin;

    use super::*;

    #[test]
    fn anchors_cover_the_box() {
        let r = Rect::from_corners(Point::new(10.0, 20.0), Point::new(30.0, 60.0));
        assert_eq!(Anchor::TopLeft.point(&r), Point::new(10.0, 20.0));
        assert_eq!(Anchor::Center.point(&r), Point::new(20.0, 40.0));
        assert_eq!(Anchor::BottomRight.point(&r), Point::new(30.0, 60.0));
        assert_eq!(Anchor::Bottom.point(&r), Point::new(20.0, 60.0));
        for a in Anchor::ALL {
            assert_eq!(a.as_str().parse::<Anchor>().unwrap(), a);
        }
        assert!("middle".parse::<Anchor>().is_err());
        assert_eq!(Anchor::default(), Anchor::BottomLeft);
    }

    #[test]
    fn machine_mapping_round_trips_for_every_corner() {
        for origin in [
            Origin::FrontLeft,
            Origin::FrontRight,
            Origin::RearLeft,
            Origin::RearRight,
        ] {
            let area = WorkArea::new(200.0, 100.0, origin).unwrap();
            let p = Point::new(30.0, 40.0);
            let m = to_machine(&area, p);
            assert!(m.x >= 0.0 && m.y >= 0.0, "{origin}");
            assert_eq!(to_document(&area, m), p, "{origin}");
            // Screen "up" (document -y) is towards the rear.
            let up = machine_delta(&area, Point::new(0.0, -1.0));
            let rear = matches!(origin, Origin::FrontLeft | Origin::FrontRight);
            assert_eq!(up.y > 0.0, rear, "{origin}");
        }
    }
}
