//! Toolpath planning for LaserCraft: layers to ordered moves, with time
//! estimates and the work-area check (`docs/spec/planning.md`).
//!
//! [`plan`] is pure and deterministic: the same project and [`Machine`]
//! always give the same [`Plan`]. Golden tests in `tests/` pin its output.

mod bounds;
mod dump;
mod emit;
mod fill;
mod image;
mod line;
mod machine;
mod model;
mod order;
mod timing;

pub use bounds::{Edge, OutOfBounds};
pub use dump::{duration, num};
pub use machine::{ACCELERATION_RANGE, MAX_SPEED_RANGE, Machine, PlanError};
pub use model::{LayerSummary, Move, MoveKind, Plan, Stats, Warning};

use lasercraft_geom::Point;
use lasercraft_project::{Layer, Mode, Origin, Project, WorkArea};

use crate::emit::{Emitter, Feed};
use crate::fill::FillSettings;
use crate::order::Outline;

/// Curves are flattened within this distance of the true shape, mm.
pub const TOLERANCE: f64 = 0.01;

/// The work area's origin corner in document coordinates: where the head
/// starts.
#[must_use]
pub fn origin_point(area: &WorkArea) -> Point {
    let (w, h) = (area.width_mm, area.height_mm);
    match area.origin {
        Origin::FrontLeft => Point::new(0.0, h),
        Origin::FrontRight => Point::new(w, h),
        Origin::RearLeft => Point::new(0.0, 0.0),
        Origin::RearRight => Point::new(w, 0.0),
    }
}

/// The outlines of a layer's shapes in drawing order, without repeated
/// points. Closed outlines with fewer than three points are treated as open.
fn outlines(project: &Project, layer: &Layer) -> Vec<Outline> {
    let mut out = Vec::new();
    for shape in project.shapes().iter().filter(|s| s.layer() == layer.color) {
        for (mut points, closed) in shape.outlines(TOLERANCE) {
            points.dedup();
            if closed && points.len() > 1 && points.first() == points.last() {
                points.pop();
            }
            if points.len() < 2 {
                continue;
            }
            let closed = closed && points.len() >= 3;
            out.push(Outline {
                shape: shape.id(),
                points,
                closed,
            });
        }
    }
    out
}

fn warn(warnings: &mut Vec<Warning>, layer: &Layer, message: impl Into<String>) {
    warnings.push(Warning {
        layer: layer.color,
        shape: None,
        message: message.into(),
    });
}

/// Plans a layer's images after its vector shapes, every pass.
fn plan_images(
    images: &[&lasercraft_project::Shape],
    layer: &Layer,
    feed: Feed,
    overscan: f64,
    out: &mut Emitter,
    warnings: &mut Vec<Warning>,
) {
    let mut rasters = Vec::new();
    for shape in images {
        match image::raster_of(shape, layer) {
            Ok(Some(raster)) if raster.lit() > 0 => rasters.push((shape.id(), raster)),
            Ok(_) => {}
            Err(reason) => warnings.push(Warning {
                layer: layer.color,
                shape: Some(shape.id()),
                message: format!("image skipped: {reason}"),
            }),
        }
    }
    let rows = image::Rows {
        bidirectional: layer.bidirectional,
        overscan,
        correction: layer.image.dot_correction,
        grayscale: layer.image.dither == lasercraft_project::Dither::Grayscale,
        min_power: layer.min_power.clamp(0.0, feed.power),
    };
    for pass in 1..=layer.passes {
        let feed = Feed { pass, ..feed };
        for (id, raster) in &rasters {
            image::emit(raster, *id, &rows, &feed, out);
        }
    }
}

/// Plans the job: every output layer in order, each pass, line or fill.
///
/// # Errors
///
/// Fails only for machine parameters out of range.
pub fn plan(project: &Project, machine: &Machine) -> Result<Plan, PlanError> {
    machine.validate()?;
    let start = origin_point(project.work_area());
    let mut out = Emitter::new(start, machine.max_speed);
    let mut layers = Vec::new();
    let mut warnings = Vec::new();
    for layer in project.layers().iter().filter(|l| l.output) {
        let outlines = outlines(project, layer);
        let images: Vec<&lasercraft_project::Shape> = project
            .shapes()
            .iter()
            .filter(|s| s.layer() == layer.color && s.geometry().is_image())
            .collect();
        if outlines.is_empty() && images.is_empty() {
            continue;
        }
        if layer.speed > machine.max_speed {
            warn(
                &mut warnings,
                layer,
                format!(
                    "speed {} mm/s is above the machine's {} mm/s; the estimate uses the machine's speed",
                    num(layer.speed),
                    num(machine.max_speed)
                ),
            );
        }
        // Clamped to the layer's and the device's maximum (safety rule 6).
        let power = layer.max_power.clamp(0.0, machine.max_power.min(100.0));
        let feed = Feed {
            layer: layer.color,
            pass: 1,
            speed: layer.speed,
            power,
        };
        let mut overscan = 0.0;
        if !outlines.is_empty() {
            match layer.mode {
                Mode::Line => {
                    let cuts = line::cuts(outlines, layer, &mut warnings);
                    line::plan_layer(&cuts, layer.passes, feed, &mut out);
                }
                Mode::Fill => {
                    let open = outlines.iter().filter(|o| !o.closed).count();
                    if open > 0 {
                        warn(
                            &mut warnings,
                            layer,
                            format!("fill needs closed shapes; {open} open outline(s) skipped"),
                        );
                    }
                    if layer.overscan {
                        overscan = machine.overscan(layer.speed);
                    }
                    let settings = FillSettings {
                        angle: layer.angle,
                        interval: layer.interval,
                        bidirectional: layer.bidirectional,
                        crosshatch: layer.crosshatch,
                        overscan,
                    };
                    let before = out.moves.len();
                    fill::plan_layer(&outlines, &settings, layer.passes, feed, &mut out);
                    if out.moves.len() == before {
                        warn(&mut warnings, layer, "nothing to fill");
                    }
                }
            }
        }
        if !images.is_empty() {
            if layer.overscan {
                overscan = machine.overscan(layer.speed);
            }
            plan_images(&images, layer, feed, overscan, &mut out, &mut warnings);
        }
        layers.push(LayerSummary {
            color: layer.color,
            name: layer.name.clone(),
            mode: layer.mode,
            passes: layer.passes,
            speed: layer.speed,
            power,
            overscan,
        });
    }
    let mut moves = out.moves;
    timing::estimate(&mut moves, machine);
    let stats = Stats::of(&moves);
    Ok(Plan {
        start,
        layers,
        moves,
        warnings,
        stats,
    })
}
