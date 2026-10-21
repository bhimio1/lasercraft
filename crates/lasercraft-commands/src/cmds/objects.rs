//! Helpers shared by the M3 commands: objects and anchors, replacing shapes
//! by new paths, moving shapes by different amounts.

use lasercraft_geom::{Affine, Path, Point};
use lasercraft_project::{Color, Geometry, Project, Shape, validate_transform};

use super::combine;
use super::shape::{remove, set_transform_invocation};
use crate::args::Args;
use crate::{CommandError, Invocation};

/// Shape ids from argument `from` on (default: the selection) and the shape
/// that marks the anchor object: the last id given, else the selection's
/// anchor (`docs/spec/arrange.md`).
pub(crate) fn ids_and_anchor(
    args: &Args<'_>,
    from: usize,
    project: &Project,
) -> Result<(Vec<u32>, Option<u32>), CommandError> {
    let ids = args.ids(from, project)?;
    let anchor = if args.len() > from {
        Some(args.parse::<u32>(args.len() - 1, "shape id")?)
    } else {
        project.anchor()
    };
    Ok((ids, anchor))
}

/// The top-level objects of `ids` and the index of the anchor's object.
pub(crate) fn objects_with_anchor(
    project: &Project,
    ids: &[u32],
    anchor: Option<u32>,
) -> Result<(Vec<Vec<u32>>, usize), CommandError> {
    let objects = project.objects_of(ids)?;
    let index = anchor
        .and_then(|a| objects.iter().position(|o| o.contains(&a)))
        .unwrap_or(0);
    Ok((objects, index))
}

/// Refuses images in a vector-only operation (`docs/spec/images.md`).
pub(crate) fn vector_only(
    project: &Project,
    ids: &[u32],
    args: &crate::args::Args<'_>,
) -> Result<(), CommandError> {
    match ids
        .iter()
        .find(|id| project.shape(**id).is_some_and(|s| s.geometry().is_image()))
    {
        Some(id) => Err(args.invalid(format!(
            "shape {id} is an image; this works on vector shapes only"
        ))),
        None => Ok(()),
    }
}

/// A shape's outline as a path in document coordinates.
pub(crate) fn doc_path(shape: &Shape) -> Path {
    shape.geometry().to_path().transformed(shape.transform())
}

/// The shape lowest in drawing order among `ids`.
pub(crate) fn bottom_layer(project: &Project, ids: &[u32]) -> Color {
    project
        .shapes()
        .iter()
        .find(|s| ids.contains(&s.id()))
        .map_or(0, Shape::layer)
}

/// Adds `outputs` on top (document coordinates, identity transform),
/// removes `inputs`, selects the outputs, and returns their ids with the
/// inverse that restores everything exactly.
pub(crate) fn replace_with(
    project: &mut Project,
    inputs: &[u32],
    outputs: Vec<(Path, Color)>,
) -> Result<(Vec<u32>, Invocation), CommandError> {
    let outputs: Vec<(Geometry, Color)> = outputs
        .into_iter()
        .map(|(p, c)| (Geometry::Path(p), c))
        .collect();
    for (g, _) in &outputs {
        g.validate()?;
    }
    let restore = if inputs.is_empty() {
        None
    } else {
        Some(remove(project, inputs)?)
    };
    let mut ids = Vec::with_capacity(outputs.len());
    for (g, color) in outputs {
        match project.add_shape_on(g, Affine::IDENTITY, color) {
            Ok(id) => ids.push(id),
            Err(err) => {
                // Cannot happen after validation; undo what was done anyway.
                for id in &ids {
                    let _ = project.remove_shape(*id);
                }
                return Err(err.into());
            }
        }
    }
    project.set_selection(ids.iter().copied())?;
    let mut inverses = vec![Invocation::new("shape.delete", super::id_args(&ids))];
    inverses.extend(restore);
    Ok((ids, combine(inverses)))
}

/// Translates each shape by its own delta, all or nothing. Returns the
/// inverse, or `None` when nothing moves.
pub(crate) fn move_shapes(
    project: &mut Project,
    moves: &[(u32, Point)],
) -> Result<Option<Invocation>, CommandError> {
    let mut updates = Vec::new();
    for (id, d) in moves {
        if *d == Point::ZERO {
            continue;
        }
        let shape = project
            .shape(*id)
            .ok_or_else(|| CommandError::InvalidArgument {
                command: "arrange".to_owned(),
                reason: format!("shape {id} does not exist"),
            })?;
        let old = *shape.transform();
        let new = Affine::translate(d.x, d.y) * old;
        validate_transform(&new)?;
        updates.push((*id, old, new));
    }
    if updates.is_empty() {
        return Ok(None);
    }
    let mut inverses = Vec::with_capacity(updates.len());
    for (id, old, new) in updates {
        project.set_shape_transform(id, new)?;
        inverses.push(set_transform_invocation(id, &old));
    }
    Ok(Some(combine(inverses)))
}
