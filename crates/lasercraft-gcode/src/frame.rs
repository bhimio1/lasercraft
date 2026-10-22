//! Framing programs: laser-off moves around the job (`docs/spec/laser.md`,
//! "Framing").

use lasercraft_geom::Point;
use lasercraft_plan::{Machine, Move, MoveKind, Plan, Stats, plan};
use lasercraft_project::Project;

use crate::place::{job_box, place, to_document, to_machine};
use crate::write::num;
use crate::{ExportError, Limits, Options, Placement, verify};

/// What the frame traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameShape {
    /// The job's box.
    #[default]
    Box,
    /// The convex hull of everything that burns ("rubber band").
    Outline,
}

/// The convex hull of `points`, counter-clockwise in a y-up frame (Andrew's
/// monotone chain). Fewer than three distinct points come back as they are.
#[must_use]
pub fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut pts: Vec<Point> = points.to_vec();
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup();
    if pts.len() < 3 {
        return pts;
    }
    let cross =
        |o: Point, a: Point, b: Point| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut hull: Vec<Point> = Vec::with_capacity(pts.len() * 2);
    for pass in [false, true] {
        let start = hull.len();
        let iter: Box<dyn Iterator<Item = &Point>> = if pass {
            Box::new(pts.iter().rev())
        } else {
            Box::new(pts.iter())
        };
        for &p in iter {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
    }
    hull
}

fn outline(plan: &Plan, shape: FrameShape) -> Vec<Point> {
    match shape {
        FrameShape::Box => job_box(plan).map_or_else(Vec::new, |b| {
            vec![
                b.min,
                Point::new(b.max.x, b.min.y),
                b.max,
                Point::new(b.min.x, b.max.y),
            ]
        }),
        FrameShape::Outline => {
            let points: Vec<Point> = plan
                .moves
                .iter()
                .filter(|m| m.kind == MoveKind::Burn)
                .flat_map(|m| [m.from, m.to])
                .collect();
            convex_hull(&points)
        }
    }
}

/// The framing program for the job as `options` would place it, traced at
/// `speed` mm/s. `head` is the head's machine position, if known; the frame
/// ends there.
///
/// # Errors
///
/// Bad speed or options, nothing that burns, a frame that leaves the work
/// area, or a verifier rejection.
pub fn frame(
    project: &Project,
    machine: &Machine,
    options: &Options,
    shape: FrameShape,
    speed: f64,
    head: Option<Point>,
) -> Result<String, ExportError> {
    frame_plan(
        project,
        plan(project, machine)?,
        machine,
        options,
        shape,
        speed,
        head,
    )
}

/// As [`frame`], for a plan made elsewhere (a test pattern) on `project`'s
/// work area.
///
/// # Errors
///
/// As [`frame`].
pub fn frame_plan(
    project: &Project,
    planned: Plan,
    machine: &Machine,
    options: &Options,
    shape: FrameShape,
    speed: f64,
    head: Option<Point>,
) -> Result<String, ExportError> {
    options.validate()?;
    if !(speed.is_finite() && speed > 0.0 && speed <= machine.max_speed) {
        return Err(ExportError::Invalid(format!(
            "framing speed must be above 0 and at most the machine's {} mm/s, got {speed}",
            machine.max_speed
        )));
    }
    let area = project.work_area();
    let placement = match options.placement {
        Placement::Relative { at, anchor, .. } => Placement::Relative {
            at,
            anchor,
            return_to_start: false,
        },
        Placement::Absolute => Placement::Absolute,
    };
    let planned = place(planned, area, placement, machine.max_speed);
    let corners = outline(&planned, shape);
    let Some(&first) = corners.first() else {
        return Err(ExportError::Nothing);
    };
    let Some(layer) = planned
        .moves
        .iter()
        .find(|m| m.kind == MoveKind::Burn)
        .map(|m| m.layer)
    else {
        return Err(ExportError::Nothing);
    };
    let start = head.map_or(first, |h| to_document(area, h));
    let edge = |kind, from: Point, to: Point| Move {
        kind,
        from,
        to,
        speed,
        power: 0.0,
        layer,
        pass: 1,
        shape: None,
        time: 0.0,
    };
    let mut moves = vec![edge(MoveKind::Travel, start, first)];
    let ring = corners.iter().skip(1).chain(std::iter::once(&first));
    let mut at = first;
    for &p in ring {
        moves.push(edge(MoveKind::Unlit, at, p));
        at = p;
    }
    if let Some(h) = head {
        moves.push(edge(MoveKind::Travel, at, to_document(area, h)));
    }
    let mut path = Plan {
        start,
        layers: Vec::new(),
        stats: Stats::of(&moves),
        moves,
        warnings: Vec::new(),
    };
    let rotary = project.rotary_on().then(|| project.rotary());
    if let Some(r) = &rotary {
        // Only the outline turns; the head's own position stays where it is.
        let about = crate::rotary::about_y(&planned, area, placement);
        let ends = (path.moves.first().map(|m| m.from), head.is_some());
        crate::rotary::apply(&mut path, r, about);
        if let (Some(from), true) = ends {
            if let Some(m) = path.moves.first_mut() {
                m.from = from;
            }
            if let (Some(m), Some(h)) = (path.moves.last_mut(), head) {
                m.to = to_document(area, h);
            }
        }
        path.check_bounds_x(area)
    } else {
        path.check_bounds(area)
    }
    .map_err(ExportError::OutOfBounds)?;

    let mut out = String::from("(LaserCraft frame: laser off)\nG90 G21\nM5 S0\n");
    let xy = |p: Point| {
        let m = to_machine(area, p);
        format!("X{} Y{}", num(m.x, 3), num(m.y, 3))
    };
    let feed = num(speed * 60.0, 1);
    for (i, m) in path.moves.iter().enumerate() {
        match m.kind {
            MoveKind::Travel => out.push_str(&format!("G0 {}\n", xy(m.to))),
            _ if i == 1 => out.push_str(&format!("G1 {} F{feed}\n", xy(m.to))),
            _ => out.push_str(&format!("G1 {}\n", xy(m.to))),
        }
    }
    out.push_str("M5 S0\n");
    let limits = Limits {
        width: area.width_mm,
        height: area.height_mm,
        s_ceiling: 0.0,
        y_free: rotary.is_some(),
    };
    let report = verify(&out, &limits).map_err(ExportError::Verify)?;
    if report.burn_length > 0.0 {
        return Err(ExportError::Verify(crate::Violation {
            line: 0,
            reason: "the frame would fire the laser".to_owned(),
        }));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_of_a_triangle_with_inner_points() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 8.0),
            Point::new(5.0, 2.0),
            Point::new(4.0, 1.0),
            Point::new(10.0, 0.0),
        ];
        let hull = convex_hull(&pts);
        assert_eq!(hull.len(), 3);
        for p in [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 8.0),
        ] {
            assert!(hull.contains(&p));
        }
        assert_eq!(convex_hull(&pts[..2]).len(), 2);
        // Collinear points collapse to the two ends.
        let line = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
        ];
        assert_eq!(convex_hull(&line).len(), 2);
    }
}
