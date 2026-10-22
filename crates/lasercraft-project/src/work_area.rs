use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ProjectError;

/// Smallest allowed work area side, in millimetres.
pub const MIN_WORK_AREA_MM: f64 = 1.0;
/// Largest allowed work area side, in millimetres.
pub const MAX_WORK_AREA_MM: f64 = 10_000.0;

/// The bed corner where the machine's coordinates are zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Front left (nearest the operator, on the left).
    #[default]
    FrontLeft,
    /// Front right.
    FrontRight,
    /// Rear left.
    RearLeft,
    /// Rear right.
    RearRight,
}

impl Origin {
    /// Every origin, in a stable order.
    pub const ALL: [Self; 4] = [
        Self::FrontLeft,
        Self::FrontRight,
        Self::RearLeft,
        Self::RearRight,
    ];

    /// The argument spelling, such as `front-left`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FrontLeft => "front-left",
            Self::FrontRight => "front-right",
            Self::RearLeft => "rear-left",
            Self::RearRight => "rear-right",
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Origin {
    type Err = ProjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|o| o.as_str().eq_ignore_ascii_case(s))
            .ok_or_else(|| {
                ProjectError::invalid(
                    "origin",
                    format!("`{s}` is not one of front-left, front-right, rear-left, rear-right"),
                )
            })
    }
}

/// The rectangle the laser head can reach. Every job is checked against it
/// before it is sent (`docs/spec/safety.md`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkArea {
    /// Width (x travel), in millimetres.
    pub width_mm: f64,
    /// Height (y travel), in millimetres.
    pub height_mm: f64,
    /// Bed corner that is the machine origin.
    pub origin: Origin,
}

impl Default for WorkArea {
    fn default() -> Self {
        Self {
            width_mm: 400.0,
            height_mm: 400.0,
            origin: Origin::FrontLeft,
        }
    }
}

impl WorkArea {
    /// The origin corner as a document point.
    #[must_use]
    pub fn origin_point(&self) -> lasercraft_geom::Point {
        let (w, h) = (self.width_mm, self.height_mm);
        match self.origin {
            Origin::FrontLeft => lasercraft_geom::Point::new(0.0, h),
            Origin::FrontRight => lasercraft_geom::Point::new(w, h),
            Origin::RearLeft => lasercraft_geom::Point::new(0.0, 0.0),
            Origin::RearRight => lasercraft_geom::Point::new(w, 0.0),
        }
    }

    fn signs(&self) -> (f64, f64) {
        let o = self.origin_point();
        (
            if o.x == 0.0 { 1.0 } else { -1.0 },
            if o.y == 0.0 { 1.0 } else { -1.0 },
        )
    }

    /// A document point in machine coordinates: signed distances from the
    /// origin corner, growing into the bed.
    #[must_use]
    pub fn to_machine(&self, p: lasercraft_geom::Point) -> lasercraft_geom::Point {
        let (o, (sx, sy)) = (self.origin_point(), self.signs());
        lasercraft_geom::Point::new(sx * (p.x - o.x), sy * (p.y - o.y))
    }

    /// A machine position as a document point (inverse of `to_machine`).
    #[must_use]
    pub fn to_document(&self, m: lasercraft_geom::Point) -> lasercraft_geom::Point {
        let (o, (sx, sy)) = (self.origin_point(), self.signs());
        lasercraft_geom::Point::new(o.x + sx * m.x, o.y + sy * m.y)
    }

    /// A validated work area.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] when a side is not finite or lies
    /// outside [`MIN_WORK_AREA_MM`]..=[`MAX_WORK_AREA_MM`].
    pub fn new(width_mm: f64, height_mm: f64, origin: Origin) -> Result<Self, ProjectError> {
        check_side("work area width", width_mm)?;
        check_side("work area height", height_mm)?;
        Ok(Self {
            width_mm,
            height_mm,
            origin,
        })
    }
}

fn check_side(field: &str, value: f64) -> Result<(), ProjectError> {
    if value.is_finite() && (MIN_WORK_AREA_MM..=MAX_WORK_AREA_MM).contains(&value) {
        Ok(())
    } else {
        Err(ProjectError::invalid(
            field,
            format!("must be between {MIN_WORK_AREA_MM} and {MAX_WORK_AREA_MM} mm, got {value}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_parse_case_insensitively_and_round_trip() {
        for origin in Origin::ALL {
            assert_eq!(origin.as_str().parse::<Origin>().unwrap(), origin);
        }
        assert_eq!("REAR-LEFT".parse::<Origin>().unwrap(), Origin::RearLeft);
        assert!("middle".parse::<Origin>().is_err());
    }

    #[test]
    fn sides_are_range_checked() {
        assert!(WorkArea::new(1.0, 10_000.0, Origin::FrontLeft).is_ok());
        for bad in [0.5, 10_000.5, f64::NAN, f64::INFINITY, -3.0] {
            assert!(
                WorkArea::new(bad, 100.0, Origin::FrontLeft).is_err(),
                "{bad}"
            );
            assert!(
                WorkArea::new(100.0, bad, Origin::FrontLeft).is_err(),
                "{bad}"
            );
        }
    }
}
