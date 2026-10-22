//! Rotary output (`docs/spec/rotary.md`, "Output"): Y becomes a rotation,
//! scaled about the job's placement point, with feeds that keep the
//! surface speed.

use lasercraft_plan::{MoveKind, Plan, Stats, Warning};
use lasercraft_project::{Rotary, WorkArea};

use crate::Placement;
use crate::place::{job_box, to_document};

/// The document Y the rotation is scaled about: the placement point, or
/// the job box's top edge for Absolute coordinates.
#[must_use]
pub fn about_y(placed: &Plan, area: &WorkArea, placement: Placement) -> f64 {
    match placement {
        Placement::Relative { at, .. } => to_document(area, at).y,
        Placement::Absolute => job_box(placed).map_or(0.0, |b| b.min.y),
    }
}

/// Warnings for a placed rotary job.
#[must_use]
pub fn warnings(placed: &Plan, rotary: &Rotary, placement: Placement) -> Vec<Warning> {
    let layer = placed.moves.first().map_or(0, |m| m.layer);
    let mut out = Vec::new();
    let warn = |message: String| Warning {
        layer,
        shape: None,
        message,
    };
    if let Some(b) = job_box(placed)
        && b.height() > rotary.circumference() + 1e-9
    {
        out.push(warn(format!(
            "the design is {:.1} mm tall, more than the object's circumference {:.1} mm; it overlaps itself",
            b.height(),
            rotary.circumference()
        )));
    }
    if placement == Placement::Absolute {
        out.push(warn(
            "rotary jobs usually start from the current position".to_owned(),
        ));
    }
    out
}

/// Turns document Y into rotary axis millimetres about `about` and scales
/// the feeds of laser moves so the surface keeps its speed. Times stay the
/// surface times.
pub fn apply(plan: &mut Plan, rotary: &Rotary, about: f64) {
    let k = rotary.scale();
    let map =
        |p: lasercraft_geom::Point| lasercraft_geom::Point::new(p.x, about + k * (p.y - about));
    for m in &mut plan.moves {
        let surface = m.from.distance(m.to);
        let (from, to) = (map(m.from), map(m.to));
        if m.kind != MoveKind::Travel && surface > 1e-12 {
            m.speed *= from.distance(to) / surface;
        }
        m.from = from;
        m.to = to;
    }
    plan.start = map(plan.start);
    let time = plan.stats.total_time;
    plan.stats = Stats::of(&plan.moves);
    debug_assert!((plan.stats.total_time - time).abs() < 1e-6 * time.max(1.0));
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::Point;
    use lasercraft_plan::{Machine, plan};
    use lasercraft_project::{Origin, Project, RotaryKind};

    use super::*;
    use crate::{Options, export};

    fn project() -> Project {
        let mut p = Project::default();
        p.set_rotary(Rotary {
            enabled: true,
            kind: RotaryKind::Chuck,
            mm_per_turn: 100.0,
            object_diameter: 100.0 / std::f64::consts::PI,
            ..Rotary::default()
        })
        .unwrap();
        p
    }

    #[test]
    fn a_chuck_with_matching_travel_keeps_y_and_scales_feeds_otherwise() {
        let mut p = project();
        let id = p.allocate_id();
        let shape = lasercraft_project::Shape::new(
            id,
            lasercraft_project::Geometry::Rect {
                width: 20.0,
                height: 10.0,
            },
            lasercraft_geom::Affine::translate(100.0, 100.0),
        )
        .unwrap();
        p.insert_shape(0, shape).unwrap();
        let placed = plan(&p, &Machine::default()).unwrap();
        let mut same = placed.clone();
        apply(&mut same, &p.rotary(), 100.0);
        // k = 100 / (π · 100/π) = 1: nothing moves.
        for (a, b) in placed.moves.iter().zip(&same.moves) {
            assert!(a.to.distance(b.to) < 1e-9 && (a.speed - b.speed).abs() < 1e-9);
        }
        let mut r = p.rotary();
        r.mm_per_turn = 50.0;
        let mut half = placed.clone();
        apply(&mut half, &r, 100.0);
        let vertical = half
            .moves
            .iter()
            .find(|m| m.kind == MoveKind::Burn && (m.from.x - m.to.x).abs() < 1e-9)
            .unwrap();
        assert!((vertical.from.y - vertical.to.y).abs() - 5.0 < 1e-9);
        let original = placed
            .moves
            .iter()
            .find(|m| m.kind == MoveKind::Burn && (m.from.x - m.to.x).abs() < 1e-9)
            .unwrap();
        assert!((vertical.speed - original.speed / 2.0).abs() < 1e-9);
        assert!((half.stats.total_time - placed.stats.total_time).abs() < 1e-9);
        // Export: Y may leave the bed, X may not.
        p.set_work_area_size(400.0, 20.0).unwrap();
        p.set_origin(Origin::RearLeft);
        let out = export(&p, &Machine::default(), &Options::default()).unwrap();
        assert!(out.gcode.contains("G1"));
        assert!(
            out.plan
                .warnings
                .iter()
                .any(|w| w.message.contains("current position"))
        );
        // The rectangle's corner at document y 100 is 80 mm past the bed.
        assert!(
            out.gcode
                .lines()
                .any(|l| l.contains("Y100") || l.contains("Y110"))
        );
        assert!(Point::new(0.0, 0.0).x == 0.0);
    }
}
