//! Plain-text forms of a plan: the golden-test dump and human-friendly
//! numbers.

use std::fmt::Write as _;

use lasercraft_geom::Point;

use crate::{MoveKind, Plan};

/// A number with at most three decimals and no trailing zeros.
#[must_use]
pub fn num(value: f64) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

fn pt(p: Point) -> String {
    format!("({}, {})", num(p.x), num(p.y))
}

/// A duration such as `42.5 s`, `3 min 07 s` or `1 h 02 min`.
#[must_use]
pub fn duration(seconds: f64) -> String {
    let s = seconds.max(0.0);
    if s < 60.0 {
        return format!("{s:.1} s");
    }
    // Whole seconds from here on; the value is small and positive.
    let whole = s.round() as u64;
    let (h, m, sec) = (whole / 3600, whole / 60 % 60, whole % 60);
    if h > 0 {
        format!("{h} h {m:02} min")
    } else {
        format!("{m} min {sec:02} s")
    }
}

impl Plan {
    /// The statistics as lines of text.
    #[must_use]
    pub fn summary(&self) -> String {
        let s = &self.stats;
        let mut out = format!(
            "burn {} mm ({})\nunlit {} mm ({})\ntravel {} mm ({})\ntotal time {}\n",
            num(s.burn_length),
            duration(s.burn_time),
            num(s.unlit_length),
            duration(s.unlit_time),
            num(s.travel_length),
            duration(s.travel_time),
            duration(s.total_time)
        );
        match s.bounds {
            Some(b) => {
                let _ = writeln!(
                    out,
                    "bounds {} – {}, {} × {} mm",
                    pt(b.min),
                    pt(b.max),
                    num(b.width()),
                    num(b.height())
                );
            }
            None => out.push_str("bounds: nothing to cut\n"),
        }
        for w in &self.warnings {
            let _ = writeln!(out, "warning: {w}");
        }
        out
    }

    /// The whole plan as text: header, layers, every move, statistics.
    /// Used by golden tests and `lasercraft-cli preview --moves`.
    #[must_use]
    pub fn dump(&self) -> String {
        let mut out = self.header();
        for m in &self.moves {
            let _ = write!(
                out,
                "{:<6} {} -> {} @{} p{}",
                m.kind.as_str(),
                pt(m.from),
                pt(m.to),
                m.layer,
                m.pass
            );
            if let Some(id) = m.shape {
                let _ = write!(out, " #{id}");
            }
            if m.kind != MoveKind::Travel {
                let _ = write!(out, " F{}", num(m.speed));
            }
            if m.kind == MoveKind::Burn {
                let _ = write!(out, " S{}", num(m.power));
            }
            out.push('\n');
        }
        out.push_str(&self.summary());
        out
    }

    /// The start position, move count and one line per layer.
    #[must_use]
    pub fn header(&self) -> String {
        let mut out = format!("start {}, {} move(s)\n", pt(self.start), self.moves.len());
        for l in &self.layers {
            let _ = write!(
                out,
                "layer {} \"{}\": {}, {} pass(es), {} mm/s, power {}%",
                l.color,
                l.name,
                l.kind(),
                l.passes,
                num(l.speed),
                num(l.power)
            );
            if l.overscan > 0.0 {
                let _ = write!(out, ", overscan {} mm", num(l.overscan));
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_and_durations() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(-0.0001), "0");
        assert_eq!(num(2.125_04), "2.125");
        assert_eq!(duration(42.46), "42.5 s");
        assert_eq!(duration(187.0), "3 min 07 s");
        assert_eq!(duration(3725.0), "1 h 02 min");
    }
}
