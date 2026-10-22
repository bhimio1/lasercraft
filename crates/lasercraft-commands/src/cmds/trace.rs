//! Trace to vectors: `image.trace` and `image.trace_picture`
//! (`docs/spec/trace.md`).

use lasercraft_geom::path::Path;
use lasercraft_geom::{Affine, Point};
use lasercraft_project::raster::trace::{InkRule, area, clean, ink_map, outlines};
use lasercraft_project::raster::{Adjust, Gray, decode};
use lasercraft_project::{Geometry, Project};
use serde::{Deserialize, Serialize};

use super::objects::replace_with;
use crate::args::Args;
use crate::command::cmd;
use crate::{Command, CommandError, Outcome};

/// Most nodes one trace may add.
pub const MAX_NODES: usize = 200_000;

/// The trace settings (`docs/spec/trace.md`, "Settings").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TraceSettings {
    /// Darkest brightness that counts as ink.
    pub from: u8,
    /// Lightest brightness that counts as ink.
    pub to: u8,
    /// Trace what is not white.
    pub transparency: bool,
    /// Follow edges (sketches).
    pub edges: bool,
    /// Drop specks (and fill holes) under this many pixels.
    pub speck: u32,
    /// 0 straight lines … 1.33 most curves.
    pub smoothness: f64,
    /// Node reduction, pixels.
    pub reduction: f64,
    /// Trace only inside `[x0, y0, x1, y1]`, pixels.
    pub area: Option<[u32; 4]>,
    /// Delete the image when done.
    pub delete: bool,
}

impl Default for TraceSettings {
    fn default() -> Self {
        Self {
            from: 0,
            to: 128,
            transparency: false,
            edges: false,
            speck: 2,
            smoothness: 1.0,
            reduction: 0.2,
            area: None,
            delete: false,
        }
    }
}

impl TraceSettings {
    /// Reads settings from JSON (missing keys take their defaults).
    ///
    /// # Errors
    ///
    /// Bad JSON or values out of range.
    pub fn parse(json: &str) -> Result<Self, String> {
        let s: Self = serde_json::from_str(json).map_err(|e| format!("trace settings: {e}"))?;
        s.validate()?;
        Ok(s)
    }

    /// Checks the ranges.
    ///
    /// # Errors
    ///
    /// The first value out of range.
    pub fn validate(&self) -> Result<(), String> {
        if self.from > self.to {
            return Err("Brightness from must not be above Brightness to".to_owned());
        }
        if self.speck > 10_000 {
            return Err("Drop specks under must be 0–10000 pixels".to_owned());
        }
        if !(0.0..=1.34).contains(&self.smoothness) {
            return Err("Smoothness must be 0–1.33".to_owned());
        }
        if !(0.0..=5.0).contains(&self.reduction) {
            return Err("Node reduction must be 0–5".to_owned());
        }
        if let Some([x0, y0, x1, y1]) = self.area
            && (x1 <= x0 || y1 <= y0)
        {
            return Err("the trace area is empty".to_owned());
        }
        Ok(())
    }
}

/// What a trace found: paths in image pixels (corners at whole numbers).
#[derive(Debug, Clone, PartialEq)]
pub struct Traced {
    /// One path per outer outline with its holes.
    pub paths: Vec<Path>,
    /// Nodes in all.
    pub nodes: usize,
}

/// Traces a bitmap (`docs/spec/trace.md`, "Method"); the app uses this for
/// the window's preview too.
///
/// # Errors
///
/// Nothing found, or a trace with more than [`MAX_NODES`] nodes.
pub fn trace(gray: &Gray, adjust: &Adjust, s: &TraceSettings) -> Result<Traced, String> {
    let rule = InkRule {
        from: s.from,
        to: s.to,
        transparency: s.transparency,
        edges: s.edges,
    };
    let mut mask = ink_map(gray, adjust, &rule, s.area);
    clean(&mut mask, s.speck as usize);
    let loops: Vec<Vec<Point>> = outlines(&mask)
        .into_iter()
        .filter(|l| area(l) != 0)
        .map(|l| {
            let corners: Vec<Point> = l
                .into_iter()
                .map(|(x, y)| Point::new(x as f64, y as f64))
                .collect();
            lasercraft_geom::trace::edge_midpoints(&corners)
        })
        .collect();
    if loops.is_empty() {
        return Err("Nothing to trace: no pixels fall in the brightness range.".to_owned());
    }
    let points: usize = loops.iter().map(Vec::len).sum();
    if points > MAX_NODES * 50 {
        return Err(too_detailed());
    }
    let (paths, nodes) =
        lasercraft_geom::trace::paths_from_loops(&loops, 0.5 + s.reduction, s.smoothness);
    if nodes > MAX_NODES {
        return Err(too_detailed());
    }
    Ok(Traced { paths, nodes })
}

fn too_detailed() -> String {
    "The trace is too detailed: raise Drop specks or Node reduction, or trace a smaller area."
        .to_owned()
}

pub(super) fn commands() -> Vec<Box<dyn Command>> {
    vec![
        cmd(
            "image.trace",
            "image.trace <id> [json] [@color]",
            "Trace an image into paths (Trace to vectors).",
            trace_image,
        ),
        cmd(
            "image.trace_picture",
            "image.trace_picture <path> <x> <y> <w> <h> [json] [@color]",
            "Trace a picture file lying on a bed rectangle into paths (Trace camera image).",
            trace_picture,
        ),
    ]
}

fn settings(args: &Args<'_>, at: usize) -> Result<TraceSettings, CommandError> {
    if args.len() > at {
        TraceSettings::parse(args.str(at)?).map_err(|e| args.invalid(e))
    } else {
        Ok(TraceSettings::default())
    }
}

/// Adds the traced paths (pixels mapped by `to_doc`) on `color`, removing
/// `inputs`, as one group.
fn add(
    project: &mut Project,
    traced: Traced,
    to_doc: &Affine,
    color: u8,
    inputs: &[u32],
    what: String,
) -> Result<Outcome, CommandError> {
    let outputs = traced
        .paths
        .iter()
        .map(|p| (p.transformed(to_doc), color))
        .collect();
    let (ids, inverse) = replace_with(project, inputs, outputs)?;
    if ids.len() > 1 {
        let group = project.allocate_id();
        for id in &ids {
            project.set_shape_groups(*id, vec![group])?;
        }
    }
    Ok(Outcome::undoable(
        format!(
            "traced {what}: {} path(s), {} nodes",
            ids.len(),
            traced.nodes
        ),
        inverse,
    ))
}

fn trace_image(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(1, 2)?;
    let id: u32 = args.parse(0, "shape id")?;
    let shape = project
        .shape(id)
        .ok_or_else(|| args.invalid(format!("shape {id} does not exist")))?;
    let Geometry::Image(image) = shape.geometry() else {
        return Err(args.invalid(format!("shape {id} is not an image")));
    };
    let s = settings(args, 1)?;
    let traced = trace(&image.bitmap, &image.adjust, &s).map_err(|e| args.invalid(e))?;
    let to_doc = *shape.transform()
        * Affine::scale(
            image.width / f64::from(image.bitmap.width()),
            image.height / f64::from(image.bitmap.height()),
        );
    let color = args.color()?.unwrap_or_else(|| shape.layer());
    let inputs = if s.delete { vec![id] } else { Vec::new() };
    add(
        project,
        traced,
        &to_doc,
        color,
        &inputs,
        format!("image {id}"),
    )
}

fn trace_picture(project: &mut Project, args: &Args<'_>) -> Result<Outcome, CommandError> {
    args.expect(5, 6)?;
    let file = args.str(0)?;
    let at = args.point(1)?;
    let (w, h) = (args.f64(3, "width")?, args.f64(4, "height")?);
    if !(w > 0.0 && h > 0.0) {
        return Err(args.invalid("the picture needs a positive size"));
    }
    let s = settings(args, 5)?;
    if s.delete {
        return Err(args.invalid("a picture file has no image to delete"));
    }
    let bytes =
        std::fs::read(file).map_err(|e| args.invalid(format!("cannot read {file}: {e}")))?;
    let gray = decode(&bytes)
        .map_err(|e| args.invalid(format!("{file}: {e}")))?
        .gray;
    let traced = trace(&gray, &Adjust::default(), &s).map_err(|e| args.invalid(e))?;
    let to_doc = Affine::translate(at.x, at.y)
        * Affine::scale(w / f64::from(gray.width()), h / f64::from(gray.height()));
    let color = args.color()?.unwrap_or(0);
    add(
        project,
        traced,
        &to_doc,
        color,
        &[],
        "the picture".to_owned(),
    )
}

#[cfg(test)]
mod tests;
