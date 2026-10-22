//! An independent check of G-code against GRBL laser-mode rules
//! (`docs/spec/gcode.md`, "Verifier"). It shares no code with the writer, so
//! a bug in one is caught by the other.

use std::fmt;

/// Longest line we write or accept.
pub const MAX_LINE: usize = 70;

/// Positions may exceed the work area by this much (rounding), mm.
const SLACK: f64 = 0.001;

/// What the verified program is allowed to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// Work area width (machine X range is 0..=width), mm.
    pub width: f64,
    /// Work area height (machine Y range is 0..=height), mm.
    pub height: f64,
    /// Highest `S` allowed anywhere.
    pub s_ceiling: f64,
    /// Y is a rotary axis: no Y limits (`docs/spec/rotary.md`).
    pub y_free: bool,
}

/// A rule the program breaks, with its 1-based line number.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    /// Line number, 1-based (0 for whole-file problems).
    pub line: usize,
    /// What is wrong.
    pub reason: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            f.write_str(&self.reason)
        } else {
            write!(f, "line {}: {}", self.line, self.reason)
        }
    }
}

impl std::error::Error for Violation {}

/// What the program does, as GRBL would run it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Report {
    /// Lines in the file.
    pub lines: usize,
    /// Length of moves with the laser on, mm.
    pub burn_length: f64,
    /// Length of `G1` moves with the laser off, mm.
    pub unlit_length: f64,
    /// Length of `G0` moves, mm.
    pub travel_length: f64,
    /// Highest `S` used on a burning move.
    pub max_s: f64,
    /// Smallest and largest machine X and Y reached: `[x0, y0, x1, y1]`.
    pub bounds: Option<[f64; 4]>,
}

#[derive(Clone, Copy, PartialEq)]
enum Spindle {
    Off,
    Constant,
    Dynamic,
}

struct State {
    units: bool,
    absolute: bool,
    rapid: bool,
    spindle: Spindle,
    s: f64,
    f: Option<f64>,
    x: f64,
    y: f64,
}

fn fail(line: usize, reason: impl Into<String>) -> Violation {
    Violation {
        line,
        reason: reason.into(),
    }
}

/// Splits a line into `(letter, number)` words, dropping `( … )` comments.
fn words(line: &str, n: usize) -> Result<Vec<(char, f64)>, Violation> {
    let mut out = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\r' => {}
            '(' => loop {
                match chars.next() {
                    Some(')') => break,
                    Some('(') | None => return Err(fail(n, "comment is not closed")),
                    Some(_) => {}
                }
            },
            letter if letter.is_ascii_alphabetic() => {
                let mut number = String::new();
                while let Some(d) = chars.next_if(|d| d.is_ascii_digit() || *d == '.' || *d == '-')
                {
                    number.push(d);
                }
                let value: f64 = number
                    .parse()
                    .ok()
                    .filter(|v: &f64| v.is_finite())
                    .ok_or_else(|| fail(n, format!("`{letter}{number}` has no valid number")))?;
                out.push((letter.to_ascii_uppercase(), value));
            }
            other => return Err(fail(n, format!("unexpected character `{other}`"))),
        }
    }
    Ok(out)
}

fn command(state: &mut State, letter: char, value: f64, n: usize) -> Result<(), Violation> {
    // Whole numbers only, from a short list.
    let code = (value.fract() == 0.0).then_some(value as i64);
    match (letter, code) {
        ('G', Some(0)) => state.rapid = true,
        ('G', Some(1)) => state.rapid = false,
        ('G', Some(21)) => state.units = true,
        ('G', Some(90)) => state.absolute = true,
        ('M', Some(3)) => state.spindle = Spindle::Constant,
        ('M', Some(4)) => state.spindle = Spindle::Dynamic,
        ('M', Some(5)) => state.spindle = Spindle::Off,
        ('M', Some(7..=9)) => {}
        _ => return Err(fail(n, format!("`{letter}{value}` is not allowed"))),
    }
    Ok(())
}

/// Checks `text` against `limits` and reports what it does.
///
/// # Errors
///
/// Returns the first [`Violation`].
pub fn verify(text: &str, limits: &Limits) -> Result<Report, Violation> {
    let mut state = State {
        units: false,
        absolute: false,
        rapid: true,
        spindle: Spindle::Off,
        s: 0.0,
        f: None,
        x: 0.0,
        y: 0.0,
    };
    let mut report = Report::default();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        report.lines = n;
        if line.chars().count() > MAX_LINE {
            return Err(fail(n, format!("longer than {MAX_LINE} characters")));
        }
        let mut target = (state.x, state.y);
        let mut moves = false;
        for (letter, value) in words(line, n)? {
            match letter {
                'G' | 'M' => command(&mut state, letter, value, n)?,
                'X' => {
                    target.0 = value;
                    moves = true;
                }
                'Y' => {
                    target.1 = value;
                    moves = true;
                }
                'S' => {
                    if !(0.0..=limits.s_ceiling).contains(&value) {
                        return Err(fail(
                            n,
                            format!("S{value} is outside 0–{}", limits.s_ceiling),
                        ));
                    }
                    state.s = value;
                }
                'F' => {
                    if value <= 0.0 {
                        return Err(fail(n, "F must be positive"));
                    }
                    state.f = Some(value);
                }
                other => return Err(fail(n, format!("`{other}` words are not allowed"))),
            }
        }
        if !moves {
            continue;
        }
        if !(state.units && state.absolute) {
            return Err(fail(n, "motion before G21 and G90"));
        }
        if !state.rapid && state.f.is_none() {
            return Err(fail(n, "G1 before any F"));
        }
        let (x, y) = target;
        let y_out = !limits.y_free && (y < -SLACK || y > limits.height + SLACK);
        if x < -SLACK || x > limits.width + SLACK || y_out {
            return Err(fail(
                n,
                format!(
                    "X{x} Y{y} is outside the work area {} × {}",
                    limits.width, limits.height
                ),
            ));
        }
        let length = (x - state.x).hypot(y - state.y);
        if state.rapid {
            report.travel_length += length;
        } else if state.spindle != Spindle::Off && state.s > 0.0 {
            report.burn_length += length;
            report.max_s = report.max_s.max(state.s);
        } else {
            report.unlit_length += length;
        }
        report.bounds = Some(match report.bounds {
            None => [
                x.min(state.x),
                y.min(state.y),
                x.max(state.x),
                y.max(state.y),
            ],
            Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
        });
        (state.x, state.y) = (x, y);
    }
    if state.spindle != Spindle::Off {
        return Err(fail(0, "the program does not end with the laser off (M5)"));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: Limits = Limits {
        width: 100.0,
        height: 50.0,
        s_ceiling: 1000.0,
        y_free: false,
    };

    fn check(text: &str) -> Result<Report, Violation> {
        verify(text, &LIMITS)
    }

    #[test]
    fn measures_burn_unlit_and_travel() {
        let r = check(
            "(job)\nG90 G21\nM5 S0\nG0 X10 Y10\nM4 S0\nG1 X20 S500 F600\nG1 X25 S0\nG1 Y20 S800\nM5 S0\n",
        )
        .unwrap();
        assert!((r.travel_length - 200f64.sqrt()).abs() < 1e-9);
        assert_eq!(r.burn_length, 20.0);
        assert_eq!(r.unlit_length, 5.0);
        assert_eq!(r.max_s, 800.0);
        assert_eq!(r.bounds, Some([0.0, 0.0, 25.0, 20.0]));
        assert_eq!(r.lines, 9);
    }

    #[test]
    fn g0_and_s0_and_m5_never_burn() {
        let r = check("G90 G21\nM4 S900\nG0 X10\nM5\nG1 X20 F100\nM4 S0\nG1 X30\nM5\n").unwrap();
        assert_eq!(r.burn_length, 0.0);
        assert_eq!(r.unlit_length, 20.0);
    }

    #[test]
    fn rejects_unsafe_or_unknown_programs() {
        for (text, wanted) in [
            ("G90 G21\nM4\nG1 X1 S1001 F100\nM5\n", "outside 0–1000"),
            ("G90 G21\nG0 X101\n", "outside the work area"),
            ("G90 G21\nG0 Y-1\n", "outside the work area"),
            ("G0 X1\n", "before G21 and G90"),
            ("G90 G21\nM4\nG1 X1 S10\nM5\n", "before any F"),
            ("G90 G21\nG91\n", "not allowed"),
            ("G90 G21\nG2 X1 Y1\n", "not allowed"),
            ("G90 G21\nZ5\n", "not allowed"),
            ("G90 G21\nM4 S10\n", "laser off"),
            ("(open comment\n", "not closed"),
            ("G90 G21\nG0 X1;note\n", "unexpected character"),
            ("G90 G21\nG1 X1 F0\n", "F must be positive"),
            ("G90 G21\nG0 X1.2.3\n", "no valid number"),
        ] {
            let err = check(text).unwrap_err().to_string();
            assert!(err.contains(wanted), "{text:?}: {err}");
        }
        let long = format!("({})\n", "x".repeat(MAX_LINE));
        assert!(
            check(&long)
                .unwrap_err()
                .to_string()
                .contains("longer than")
        );
    }
}
