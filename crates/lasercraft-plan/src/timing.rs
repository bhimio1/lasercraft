//! Time estimate: trapezoidal speed profiles joined at corner-limited
//! junction speeds (`docs/spec/planning.md`, statistics).

use lasercraft_geom::Point;

use crate::{Machine, Move, MoveKind};

fn direction(m: &Move) -> Point {
    let d = m.to - m.from;
    let len = d.length();
    if len > 0.0 { d * (1.0 / len) } else { d }
}

/// The speed a move cruises at.
fn cruise(m: &Move, machine: &Machine) -> f64 {
    match m.kind {
        MoveKind::Travel => machine.max_speed,
        MoveKind::Burn | MoveKind::Unlit => machine.feed(m.speed),
    }
}

/// Time to cover `length` starting at `v0`, ending at `v1`, cruising at most
/// at `v` with acceleration `a`. Expects `v0, v1 <= v` and that `v1` is
/// reachable from `v0` over the length.
#[must_use]
pub(crate) fn trapezoid(length: f64, v0: f64, v1: f64, v: f64, a: f64) -> f64 {
    if length <= 0.0 || v <= 0.0 {
        return 0.0;
    }
    let d0 = (v * v - v0 * v0) / (2.0 * a);
    let d1 = (v * v - v1 * v1) / (2.0 * a);
    if d0 + d1 <= length {
        (v - v0) / a + (v - v1) / a + (length - d0 - d1) / v
    } else {
        let peak = ((2.0 * a * length + v0 * v0 + v1 * v1) / 2.0).sqrt();
        ((peak - v0) / a + (peak - v1) / a).max(0.0)
    }
}

/// Fills in every move's `time`.
pub(crate) fn estimate(moves: &mut [Move], machine: &Machine) {
    let n = moves.len();
    if n == 0 {
        return;
    }
    let a = machine.acceleration;
    let speeds: Vec<f64> = moves.iter().map(|m| cruise(m, machine)).collect();
    let lengths: Vec<f64> = moves.iter().map(Move::length).collect();
    // entry[i]: speed entering move i; entry[n]: speed at the end (rest).
    let mut entry = vec![0.0; n + 1];
    for i in 1..n {
        let (d0, d1) = (direction(&moves[i - 1]), direction(&moves[i]));
        let turn = d0.x * d1.x + d0.y * d1.y;
        entry[i] = speeds[i - 1].min(speeds[i]) * turn.max(0.0);
    }
    // Backward: each move must be able to slow down to its exit speed.
    for i in (0..n).rev() {
        let reachable = (entry[i + 1] * entry[i + 1] + 2.0 * a * lengths[i]).sqrt();
        entry[i] = entry[i].min(reachable);
    }
    // Forward: each move can only speed up so much.
    for i in 0..n {
        let reachable = (entry[i] * entry[i] + 2.0 * a * lengths[i]).sqrt();
        entry[i + 1] = entry[i + 1].min(reachable);
    }
    for (i, m) in moves.iter_mut().enumerate() {
        m.time = trapezoid(lengths[i], entry[i], entry[i + 1], speeds[i], a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(from: (f64, f64), to: (f64, f64), speed: f64) -> Move {
        Move {
            kind: MoveKind::Burn,
            from: Point::new(from.0, from.1),
            to: Point::new(to.0, to.1),
            speed,
            power: 10.0,
            layer: 0,
            pass: 1,
            shape: Some(1),
            time: 0.0,
        }
    }

    #[test]
    fn trapezoid_with_and_without_cruise() {
        // 100 mm at 10 mm/s, a = 100: ramps of 0.5 mm, 0.1 s each.
        let t = trapezoid(100.0, 0.0, 0.0, 10.0, 100.0);
        assert!((t - (0.1 + 0.1 + 99.0 / 10.0)).abs() < 1e-12, "{t}");
        // Too short to reach speed: triangle, peak sqrt(a·L).
        let t = trapezoid(1.0, 0.0, 0.0, 100.0, 100.0);
        assert!((t - 2.0 * 10.0 / 100.0).abs() < 1e-12, "{t}");
        // Already at speed both ends: length / speed.
        assert!((trapezoid(50.0, 10.0, 10.0, 10.0, 100.0) - 5.0).abs() < 1e-12);
        assert_eq!(trapezoid(0.0, 0.0, 0.0, 10.0, 100.0), 0.0);
    }

    #[test]
    fn straight_moves_blend_and_corners_stop() {
        let machine = Machine {
            acceleration: 100.0,
            ..Machine::default()
        };
        // Two collinear 50 mm moves time like one 100 mm move.
        let mut straight = vec![
            mv((0.0, 0.0), (50.0, 0.0), 10.0),
            mv((50.0, 0.0), (100.0, 0.0), 10.0),
        ];
        estimate(&mut straight, &machine);
        let total: f64 = straight.iter().map(|m| m.time).sum();
        assert!((total - trapezoid(100.0, 0.0, 0.0, 10.0, 100.0)).abs() < 1e-9);
        // A right angle stops: two separate trapezoids.
        let mut corner = vec![
            mv((0.0, 0.0), (50.0, 0.0), 10.0),
            mv((50.0, 0.0), (50.0, 50.0), 10.0),
        ];
        estimate(&mut corner, &machine);
        let total: f64 = corner.iter().map(|m| m.time).sum();
        assert!((total - 2.0 * trapezoid(50.0, 0.0, 0.0, 10.0, 100.0)).abs() < 1e-9);
    }

    #[test]
    fn speed_is_capped_at_the_machine_maximum() {
        let machine = Machine {
            max_speed: 20.0,
            acceleration: 1e6,
            ..Machine::default()
        };
        let mut fast = vec![mv((0.0, 0.0), (100.0, 0.0), 500.0)];
        estimate(&mut fast, &machine);
        assert!((fast[0].time - 5.0).abs() < 1e-3, "{}", fast[0].time);
    }
}
