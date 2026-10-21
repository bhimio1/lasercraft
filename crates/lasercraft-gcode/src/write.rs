//! Turns a plan into G-code text (`docs/spec/gcode.md`, "Output").

use std::fmt::Write as _;

use lasercraft_geom::Point;
use lasercraft_plan::{Move, MoveKind, Plan, duration};
use lasercraft_project::{Color, Project, WorkArea};

use crate::{Air, Options};

/// A number with at most `decimals` decimals, trailing zeros dropped, never
/// `-0`.
pub(crate) fn num(value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        &text
    };
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// Document point to machine coordinates: distances from the origin corner.
fn machine(area: &WorkArea, p: Point) -> Point {
    crate::place::to_machine(area, p)
}

/// A comment line, kept to safe characters and within the line limit.
fn comment(text: &str) -> String {
    let safe: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " -_.,:×".contains(c) {
                if c == '×' { 'x' } else { c }
            } else {
                '_'
            }
        })
        .take(crate::MAX_LINE - 2)
        .collect();
    format!("({safe})")
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Off,
    Constant,
    Dynamic,
}

struct Writer<'a> {
    out: String,
    area: &'a WorkArea,
    s_max: f64,
    air: Air,
    /// Unknown until the first move: the head may be anywhere.
    pos: Option<Point>,
    s: Option<f64>,
    f: Option<f64>,
    mode: Mode,
    air_on: bool,
    used_air: bool,
    layer: Option<Color>,
}

impl Writer<'_> {
    fn line(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Sets laser mode and air for a layer before its first move.
    fn enter_layer(&mut self, project: &Project, color: Color) {
        if self.layer == Some(color) {
            return;
        }
        self.layer = Some(color);
        let Ok(layer) = project.layer(color) else {
            return;
        };
        let mode = if layer.constant_power {
            Mode::Constant
        } else {
            Mode::Dynamic
        };
        if mode != self.mode {
            self.line(if mode == Mode::Constant {
                "M3 S0"
            } else {
                "M4 S0"
            });
            self.mode = mode;
            self.s = Some(0.0);
        }
        if layer.air != self.air_on {
            let code = match (layer.air, self.air) {
                (true, Air::M7) => "M7",
                (true, Air::M8) => "M8",
                (false, _) => "M9",
            };
            self.line(code);
            self.air_on = layer.air;
            self.used_air |= layer.air;
        }
    }

    fn motion(&mut self, m: &Move) {
        let to = machine(self.area, m.to);
        if Some(to) == self.pos {
            return;
        }
        let mut text = String::new();
        let rapid = m.kind == MoveKind::Travel;
        text.push_str(if rapid { "G0" } else { "G1" });
        let _ = write!(text, " X{} Y{}", num(to.x, 3), num(to.y, 3));
        if !rapid {
            let s = if m.kind == MoveKind::Burn {
                // Power is already clamped to the layer's and machine's max;
                // rounding down keeps S from creeping over that ceiling.
                (m.power.clamp(0.0, 100.0) / 100.0 * self.s_max * 100.0).floor() / 100.0
            } else {
                0.0
            };
            if self.s != Some(s) {
                let _ = write!(text, " S{}", num(s, 2));
                self.s = Some(s);
            }
            let f = (m.speed * 60.0 * 10.0).round() / 10.0;
            if self.f != Some(f) {
                let _ = write!(text, " F{}", num(f, 1));
                self.f = Some(f);
            }
        }
        self.line(&text);
        self.pos = Some(to);
    }
}

/// The whole program for `plan`.
pub(crate) fn program(project: &Project, plan: &Plan, options: &Options) -> String {
    let area = project.work_area();
    let mut w = Writer {
        out: String::new(),
        area,
        s_max: options.s_max,
        air: options.air,
        pos: None,
        s: None,
        f: None,
        mode: Mode::Off,
        air_on: false,
        used_air: false,
        layer: None,
    };
    w.line(&comment("LaserCraft G-code for GRBL laser mode"));
    w.line(&comment(&format!("Job: {}", project.name())));
    if let Some(b) = plan.stats.bounds {
        let (a, c) = (machine(area, b.min), machine(area, b.max));
        w.line(&comment(&format!(
            "Bounds: X{}..{} Y{}..{} mm",
            num(a.x.min(c.x), 3),
            num(a.x.max(c.x), 3),
            num(a.y.min(c.y), 3),
            num(a.y.max(c.y), 3)
        )));
    }
    w.line(&comment(&format!(
        "Estimated time: {}",
        duration(plan.stats.total_time)
    )));
    w.line("G90 G21");
    w.line("M5 S0");
    w.s = Some(0.0);
    if let Some(first) = plan.moves.first()
        && first.kind != MoveKind::Travel
    {
        // The head may be anywhere: get to the first cut with the laser off.
        let to = machine(area, first.from);
        w.line(&format!("G0 X{} Y{}", num(to.x, 3), num(to.y, 3)));
        w.pos = Some(to);
    }
    for m in &plan.moves {
        w.enter_layer(project, m.layer);
        w.motion(m);
    }
    w.line("M5 S0");
    if w.used_air {
        w.line("M9");
    }
    if options.finish_at_origin {
        w.line("G0 X0 Y0");
    }
    w.out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_short_and_never_negative_zero() {
        assert_eq!(num(12.0, 3), "12");
        assert_eq!(num(12.3456, 3), "12.346");
        assert_eq!(num(-0.0001, 3), "0");
        assert_eq!(num(600.0, 1), "600");
        assert_eq!(num(0.5, 2), "0.5");
    }

    #[test]
    fn comments_are_sanitized_and_short() {
        assert_eq!(comment("Job: a(b);c"), "(Job: a_b__c)");
        assert!(comment(&"x".repeat(200)).len() <= crate::MAX_LINE);
    }

    #[test]
    fn machine_coordinates_grow_away_from_the_origin_corner() {
        use lasercraft_project::Origin;
        let p = Point::new(10.0, 30.0);
        for (origin, want) in [
            (Origin::FrontLeft, Point::new(10.0, 70.0)),
            (Origin::FrontRight, Point::new(190.0, 70.0)),
            (Origin::RearLeft, Point::new(10.0, 30.0)),
            (Origin::RearRight, Point::new(190.0, 30.0)),
        ] {
            let area = WorkArea::new(200.0, 100.0, origin).unwrap();
            assert_eq!(machine(&area, p), want, "{origin}");
        }
    }
}
