//! Machine parameters the planner needs (`docs/spec/planning.md`). Device
//! profiles (M8) will fill these in; until then the defaults are our own.

use std::fmt;

/// Speed, acceleration and power limits of the laser.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Machine {
    /// Top speed of any move, mm/s; travel moves run at it.
    pub max_speed: f64,
    /// Acceleration, mm/s².
    pub acceleration: f64,
    /// Device power ceiling, %.
    pub max_power: f64,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            acceleration: 1000.0,
            max_power: 100.0,
        }
    }
}

/// Allowed `max_speed`, mm/s.
pub const MAX_SPEED_RANGE: (f64, f64) = (1.0, 10_000.0);
/// Allowed `acceleration`, mm/s².
pub const ACCELERATION_RANGE: (f64, f64) = (1.0, 100_000.0);

/// Bad machine parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError(pub String);

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PlanError {}

fn check(name: &str, value: f64, (min, max): (f64, f64)) -> Result<(), PlanError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(PlanError(format!(
            "machine {name} must be between {min} and {max}, got {value}"
        )))
    }
}

impl Machine {
    /// Checks every field.
    ///
    /// # Errors
    ///
    /// Names the first field out of range.
    pub fn validate(&self) -> Result<(), PlanError> {
        check("max_speed", self.max_speed, MAX_SPEED_RANGE)?;
        check("acceleration", self.acceleration, ACCELERATION_RANGE)?;
        check("max_power", self.max_power, (0.0, 100.0))
    }

    /// The speed a feed at `speed` actually reaches, mm/s.
    #[must_use]
    pub fn feed(&self, speed: f64) -> f64 {
        speed.min(self.max_speed)
    }

    /// Run-on needed to reach `speed` from rest: v² / 2a, mm.
    #[must_use]
    pub fn overscan(&self, speed: f64) -> f64 {
        let v = self.feed(speed);
        v * v / (2.0 * self.acceleration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_checked() {
        let m = Machine::default();
        m.validate().unwrap();
        for bad in [
            Machine {
                max_speed: 0.0,
                ..m
            },
            Machine {
                acceleration: f64::NAN,
                ..m
            },
            Machine {
                max_power: 101.0,
                ..m
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn overscan_is_the_distance_to_reach_speed() {
        let m = Machine::default();
        assert!((m.overscan(100.0) - 5.0).abs() < 1e-12);
        // Capped at the machine's top speed.
        assert!((m.overscan(400.0) - 5.0).abs() < 1e-12);
        assert!((m.overscan(10.0) - 0.05).abs() < 1e-12);
    }
}
