//! Job setups stored with the project (M9): the rotary attachment
//! (`docs/spec/rotary.md`) and print registration
//! (`docs/spec/registration.md`).

use std::f64::consts::PI;
use std::fmt;
use std::str::FromStr;

use lasercraft_geom::{Affine, Point};
use serde::{Deserialize, Serialize};

use crate::{ProjectError, WorkArea};

/// How the rotary holds the object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotaryKind {
    /// The object rests on driven rollers.
    #[default]
    Rollers,
    /// The object is held and turned.
    Chuck,
}

impl RotaryKind {
    /// The argument spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rollers => "rollers",
            Self::Chuck => "chuck",
        }
    }
}

impl fmt::Display for RotaryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RotaryKind {
    type Err = ProjectError;

    fn from_str(s: &str) -> Result<Self, ProjectError> {
        match s {
            "rollers" => Ok(Self::Rollers),
            "chuck" => Ok(Self::Chuck),
            _ => Err(ProjectError::invalid("kind", "must be rollers or chuck")),
        }
    }
}

/// Rotary attachment settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rotary {
    /// Rotary output is on.
    pub enabled: bool,
    /// Rollers or chuck.
    pub kind: RotaryKind,
    /// Y travel for one full turn of the driven part, mm.
    pub mm_per_turn: f64,
    /// Driven roller diameter, mm.
    pub roller_diameter: f64,
    /// Object diameter (the widest end), mm.
    pub object_diameter: f64,
    /// Turn the other way.
    pub mirror: bool,
}

impl Default for Rotary {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: RotaryKind::Rollers,
            mm_per_turn: 100.0,
            roller_diameter: 30.0,
            object_diameter: 80.0,
            mirror: false,
        }
    }
}

/// The keys `rotary.set` accepts, in display order.
pub const ROTARY_KEYS: [&str; 6] = [
    "enabled",
    "kind",
    "mm_per_turn",
    "roller_diameter",
    "object_diameter",
    "mirror",
];

fn number(key: &str, value: &str, (min, max): (f64, f64)) -> Result<f64, ProjectError> {
    let v: f64 = value
        .trim()
        .parse()
        .map_err(|_| ProjectError::invalid(key, format!("must be a number, not `{value}`")))?;
    if v.is_finite() && (min..=max).contains(&v) {
        Ok(v)
    } else {
        Err(ProjectError::invalid(
            key,
            format!("must be between {min} and {max}, got {v}"),
        ))
    }
}

fn on_off(key: &str, value: &str) -> Result<bool, ProjectError> {
    match value {
        "on" | "true" => Ok(true),
        "off" | "false" => Ok(false),
        _ => Err(ProjectError::invalid(key, "must be on or off")),
    }
}

impl Rotary {
    /// Diameter whose surface the Y axis drives, mm.
    #[must_use]
    pub fn surface_diameter(&self) -> f64 {
        match self.kind {
            RotaryKind::Rollers => self.roller_diameter,
            RotaryKind::Chuck => self.object_diameter,
        }
    }

    /// The object's circumference, mm.
    #[must_use]
    pub fn circumference(&self) -> f64 {
        PI * self.object_diameter
    }

    /// Axis millimetres per surface millimetre (negative when mirrored).
    #[must_use]
    pub fn scale(&self) -> f64 {
        let k = self.mm_per_turn / (PI * self.surface_diameter());
        if self.mirror { -k } else { k }
    }

    /// Degrees the object turns for `height` mm of design.
    #[must_use]
    pub fn degrees(&self, height: f64) -> f64 {
        height / self.circumference() * 360.0
    }

    /// Changes one setting.
    ///
    /// # Errors
    ///
    /// An unknown key or a value out of range; nothing changes then.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), ProjectError> {
        match key {
            "enabled" => self.enabled = on_off(key, value)?,
            "kind" => self.kind = value.parse()?,
            "mm_per_turn" => self.mm_per_turn = number(key, value, (1.0, 10_000.0))?,
            "roller_diameter" => self.roller_diameter = number(key, value, (1.0, 1000.0))?,
            "object_diameter" => self.object_diameter = number(key, value, (1.0, 1000.0))?,
            "mirror" => self.mirror = on_off(key, value)?,
            _ => {
                return Err(ProjectError::invalid(
                    key,
                    format!("is not a rotary setting ({})", ROTARY_KEYS.join(", ")),
                ));
            }
        }
        Ok(())
    }

    /// A setting as `set` takes it.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        let flag = |b: bool| if b { "on" } else { "off" }.to_owned();
        Some(match key {
            "enabled" => flag(self.enabled),
            "kind" => self.kind.as_str().to_owned(),
            "mm_per_turn" => self.mm_per_turn.to_string(),
            "roller_diameter" => self.roller_diameter.to_string(),
            "object_diameter" => self.object_diameter.to_string(),
            "mirror" => flag(self.mirror),
            _ => return None,
        })
    }

    /// Checks every value (for files).
    ///
    /// # Errors
    ///
    /// The first value out of range.
    pub fn validate(&self) -> Result<(), ProjectError> {
        let mut copy = Self::default();
        for key in ROTARY_KEYS {
            copy.set(key, &self.get(key).unwrap_or_default())?;
        }
        Ok(())
    }
}

/// One print registration target.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// The object: a shape id or a top-level group id.
    pub object: u32,
    /// Center of the object's box, document mm.
    pub design: Point,
    /// The head's position when the target was set, machine mm.
    pub machine: Point,
}

/// How the job is fitted to the targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fit {
    /// Not fitted yet.
    #[default]
    None,
    /// Move, turn and scale.
    Scaled,
    /// Move and turn only.
    Rigid,
}

impl Fit {
    /// The argument spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Scaled => "scaled",
            Self::Rigid => "rigid",
        }
    }
}

/// Print registration: up to two targets and the fit.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registration {
    /// Target 1, then target 2.
    pub targets: Vec<Target>,
    /// The fit in use.
    #[serde(default)]
    pub fit: Fit,
}

/// The similarity mapping `a₁ → b₁`, `a₂ → b₂` (scaled), or the rotation
/// that turns `a₂ − a₁` to the direction of `b₂ − b₁` about the midpoints
/// (rigid). `None` when the `a` points coincide.
#[must_use]
pub fn fit_transform(a: [Point; 2], b: [Point; 2], scaled: bool) -> Option<Affine> {
    let (da, db) = (a[1] - a[0], b[1] - b[0]);
    let la = (da.x * da.x + da.y * da.y).sqrt();
    let lb = (db.x * db.x + db.y * db.y).sqrt();
    if la < 1e-9 || lb < 1e-9 {
        return None;
    }
    let angle = db.y.atan2(db.x) - da.y.atan2(da.x);
    let s = if scaled { lb / la } else { 1.0 };
    let (c, n) = (angle.cos() * s, angle.sin() * s);
    // x' = c·x − n·y + e, y' = n·x + c·y + f.
    let (from, to) = if scaled {
        (a[0], b[0])
    } else {
        (
            Point::new((a[0].x + a[1].x) / 2.0, (a[0].y + a[1].y) / 2.0),
            Point::new((b[0].x + b[1].x) / 2.0, (b[0].y + b[1].y) / 2.0),
        )
    };
    let e = to.x - (c * from.x - n * from.y);
    let f = to.y - (n * from.x + c * from.y);
    Some(Affine {
        m: [c, n, -n, c, e, f],
    })
}

impl Registration {
    /// The document transform of the fit, if one is set and both targets
    /// are known.
    #[must_use]
    pub fn transform(&self, area: &WorkArea) -> Option<Affine> {
        let scaled = match self.fit {
            Fit::None => return None,
            Fit::Scaled => true,
            Fit::Rigid => false,
        };
        let [t1, t2] = self.targets.as_slice() else {
            return None;
        };
        fit_transform(
            [t1.design, t2.design],
            [area.to_document(t1.machine), area.to_document(t2.machine)],
            scaled,
        )
    }

    /// The measured distance over the design distance between the targets.
    #[must_use]
    pub fn scale(&self) -> Option<f64> {
        let [t1, t2] = self.targets.as_slice() else {
            return None;
        };
        let d = t1.design.distance(t2.design);
        (d > 1e-9).then(|| t1.machine.distance(t2.machine) / d)
    }

    /// Checks targets and coordinates (for files).
    ///
    /// # Errors
    ///
    /// More than two targets or a non-finite point.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.targets.len() > 2 {
            return Err(ProjectError::invalid(
                "registration",
                "has more than two targets",
            ));
        }
        let finite = |p: Point| p.x.is_finite() && p.y.is_finite();
        if self
            .targets
            .iter()
            .any(|t| !finite(t.design) || !finite(t.machine))
        {
            return Err(ProjectError::invalid(
                "registration",
                "has a point that is not a number",
            ));
        }
        if self.fit != Fit::None && self.targets.len() != 2 {
            return Err(ProjectError::invalid(
                "registration",
                "is fitted without two targets",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Origin;

    #[test]
    fn rotary_scale_follows_the_driven_diameter() {
        let mut r = Rotary::default();
        assert!((r.scale() - 100.0 / (PI * 30.0)).abs() < 1e-12);
        r.set("kind", "chuck").unwrap();
        assert!((r.scale() - 100.0 / (PI * 80.0)).abs() < 1e-12);
        r.set("mirror", "on").unwrap();
        assert!(r.scale() < 0.0);
        assert!((r.degrees(r.circumference() / 2.0) - 180.0).abs() < 1e-9);
        assert!(r.set("mm_per_turn", "0").is_err());
        assert!(r.set("speed", "1").is_err());
        assert!(r.set("enabled", "maybe").is_err());
        for key in ROTARY_KEYS {
            let v = r.get(key).unwrap();
            let mut copy = Rotary::default();
            copy.set(key, &v).unwrap();
            assert_eq!(copy.get(key).unwrap(), v);
        }
        r.validate().unwrap();
    }

    #[test]
    fn scaled_fit_maps_both_targets_and_rigid_keeps_lengths() {
        let a = [Point::new(10.0, 10.0), Point::new(110.0, 10.0)];
        let b = [Point::new(20.0, 20.0), Point::new(20.0, 122.0)];
        let t = fit_transform(a, b, true).unwrap();
        for i in 0..2 {
            assert!(t.apply(a[i]).distance(b[i]) < 1e-9);
        }
        let r = fit_transform(a, b, false).unwrap();
        let (p, q) = (r.apply(a[0]), r.apply(a[1]));
        assert!((p.distance(q) - 100.0).abs() < 1e-9);
        let mid = |u: Point, v: Point| Point::new((u.x + v.x) / 2.0, (u.y + v.y) / 2.0);
        assert!(mid(p, q).distance(mid(b[0], b[1])) < 1e-9);
        assert!(fit_transform([a[0], a[0]], b, true).is_none());
    }

    #[test]
    fn registration_uses_measured_points_on_the_bed() {
        let area = WorkArea::new(200.0, 100.0, Origin::FrontLeft).unwrap();
        let mut reg = Registration {
            targets: vec![
                Target {
                    object: 1,
                    design: Point::new(10.0, 10.0),
                    machine: Point::new(15.0, 90.0),
                },
                Target {
                    object: 2,
                    design: Point::new(110.0, 10.0),
                    machine: Point::new(115.0, 90.0),
                },
            ],
            fit: Fit::None,
        };
        assert!(reg.transform(&area).is_none());
        reg.fit = Fit::Scaled;
        let t = reg.transform(&area).unwrap();
        // Machine (15, 90) on a front-left bed 100 deep is document (15, 10).
        assert!(
            t.apply(Point::new(10.0, 10.0))
                .distance(Point::new(15.0, 10.0))
                < 1e-9
        );
        assert!((reg.scale().unwrap() - 1.0).abs() < 1e-12);
        reg.validate().unwrap();
        reg.targets.pop();
        assert!(reg.validate().is_err());
    }
}
