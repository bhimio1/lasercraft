//! GRBL-dialect G-code output for LaserCraft (`docs/spec/gcode.md`).
//!
//! [`export`] plans the job, refuses it when it leaves the bed, writes G-code
//! and checks the result with the independent [`verify`] before returning
//! it. Pure and deterministic: no I/O.

mod frame;
mod place;
mod verify;
mod write;

pub use frame::{FrameShape, convex_hull, frame};
pub use place::{Anchor, Placement, job_box, machine_delta, place, to_document, to_machine};
pub use verify::{Limits, MAX_LINE, Report, Violation, verify};

use std::fmt;
use std::str::FromStr;

use lasercraft_plan::{Machine, MoveKind, OutOfBounds, Plan, PlanError, plan};
use lasercraft_project::Project;

/// Allowed `s_max`.
pub const S_MAX_RANGE: (f64, f64) = (1.0, 100_000.0);

/// Which coolant command switches air assist on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Air {
    /// `M7` (mist coolant output).
    M7,
    /// `M8` (flood coolant output).
    #[default]
    M8,
}

impl FromStr for Air {
    type Err = ExportError;

    fn from_str(s: &str) -> Result<Self, ExportError> {
        match s.to_ascii_lowercase().as_str() {
            "m7" => Ok(Self::M7),
            "m8" => Ok(Self::M8),
            other => Err(ExportError::Invalid(format!(
                "air must be m7 or m8, not `{other}`"
            ))),
        }
    }
}

/// Export settings beyond the machine's motion limits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// The `S` value for 100% power; must equal GRBL's `$30`.
    pub s_max: f64,
    /// Air assist command.
    pub air: Air,
    /// End with `G0 X0 Y0`.
    pub finish_at_origin: bool,
    /// Where the job runs (`docs/spec/laser.md`).
    pub placement: Placement,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            s_max: 1000.0,
            air: Air::M8,
            finish_at_origin: false,
            placement: Placement::Absolute,
        }
    }
}

/// Why a job was not exported.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportError {
    /// Bad options or machine parameters.
    Invalid(String),
    /// The job leaves the work area.
    OutOfBounds(OutOfBounds),
    /// No layer burns anything.
    Nothing,
    /// The verifier rejected the generated text (a generator bug).
    Verify(Violation),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => f.write_str(reason),
            Self::OutOfBounds(err) => write!(f, "job does not fit the work area: {err}"),
            Self::Nothing => f.write_str("nothing to export: no output layer burns anything"),
            Self::Verify(v) => write!(
                f,
                "the generated G-code failed its safety check ({v}); this is a LaserCraft bug, nothing was written"
            ),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<PlanError> for ExportError {
    fn from(err: PlanError) -> Self {
        Self::Invalid(err.to_string())
    }
}

/// G-code text with what the verifier measured, and the plan it came from.
#[derive(Debug, Clone)]
pub struct Exported {
    /// The program.
    pub gcode: String,
    /// The verifier's measurements.
    pub report: Report,
    /// The plan behind it.
    pub plan: Plan,
}

impl Options {
    /// Checks every field.
    ///
    /// # Errors
    ///
    /// Names the first field out of range.
    pub fn validate(&self) -> Result<(), ExportError> {
        let (min, max) = S_MAX_RANGE;
        if self.s_max.is_finite() && (min..=max).contains(&self.s_max) {
            Ok(())
        } else {
            Err(ExportError::Invalid(format!(
                "s_max must be between {min} and {max} (GRBL's $30), got {}",
                self.s_max
            )))
        }
    }
}

/// Plans `project` and writes it as GRBL G-code.
///
/// # Errors
///
/// See [`ExportError`]: bad options, a job off the bed, nothing to burn, or a
/// verifier rejection.
pub fn export(
    project: &Project,
    machine: &Machine,
    options: &Options,
) -> Result<Exported, ExportError> {
    options.validate()?;
    let plan = place(
        plan(project, machine)?,
        project.work_area(),
        options.placement,
        machine.max_speed,
    );
    plan.check_bounds(project.work_area())
        .map_err(ExportError::OutOfBounds)?;
    if !plan.moves.iter().any(|m| m.kind == MoveKind::Burn) {
        return Err(ExportError::Nothing);
    }
    let gcode = write::program(project, &plan, options);
    let area = project.work_area();
    let limits = Limits {
        width: area.width_mm,
        height: area.height_mm,
        s_ceiling: options.s_max * machine.max_power.clamp(0.0, 100.0) / 100.0,
    };
    let report = verify(&gcode, &limits).map_err(ExportError::Verify)?;
    Ok(Exported {
        gcode,
        report,
        plan,
    })
}

#[cfg(test)]
mod tests;
