//! The project's shapes, selection and clipboard (`docs/spec/shapes.md`).

use std::collections::{BTreeSet, HashSet};

use lasercraft_geom::{Affine, Point, Rect};

use crate::layer::{Color, LAYER_COUNT};
use crate::shape::validate_transform;
use crate::{Geometry, Project, ProjectError, Shape};

/// Selection and clipboard. View state: never saved or undone, and two
/// projects compare equal whatever their view state.
#[derive(Debug, Clone, Default)]
pub struct ViewState {
    selection: BTreeSet<u32>,
    clipboard: Vec<Shape>,
    /// A shape of the object added to the selection last
    /// (`docs/spec/arrange.md`, "Anchor object").
    anchor: Option<u32>,
}

impl PartialEq for ViewState {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Which shapes a selection box picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxMode {
    /// Shapes whose bounds lie entirely inside the box.
    Contained,
    /// Shapes whose outline crosses or lies inside the box.
    Touching,
}

/// A top-level object: an outermost group, or an ungrouped shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObjectKey {
    /// All shapes whose outermost group is this id.
    Group(u32),
    /// One ungrouped shape.
    Shape(u32),
}

impl ObjectKey {
    /// The object a shape belongs to.
    #[must_use]
    pub fn of(shape: &Shape) -> Self {
        shape
            .top_group()
            .map_or(Self::Shape(shape.id()), Self::Group)
    }
}

fn unknown(id: u32) -> ProjectError {
    ProjectError::invalid("shape", format!("{id} does not exist"))
}

impl Project {
    /// Shapes in drawing order.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    /// The shape with this id.
    #[must_use]
    pub fn shape(&self, id: u32) -> Option<&Shape> {
        self.shapes.iter().find(|s| s.id() == id)
    }

    /// Drawing-order position of a shape.
    #[must_use]
    pub fn index_of(&self, id: u32) -> Option<usize> {
        self.shapes.iter().position(|s| s.id() == id)
    }

    /// Adds a shape on top with a fresh id and returns the id.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for invalid geometry or transform.
    pub fn add_shape(
        &mut self,
        geometry: Geometry,
        transform: Affine,
    ) -> Result<u32, ProjectError> {
        self.add_shape_on(geometry, transform, 0)
    }

    /// Adds a shape on top of the layer `color` with a fresh id and returns
    /// the id.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for invalid geometry, transform
    /// or color.
    pub fn add_shape_on(
        &mut self,
        geometry: Geometry,
        transform: Affine,
        color: Color,
    ) -> Result<u32, ProjectError> {
        let mut shape = Shape::new(self.next_id.max(1), geometry, transform)?;
        shape.set_layer(color);
        shape.validate()?;
        let id = self.allocate_id();
        self.shapes.push(shape);
        Ok(id)
    }

    /// Inserts a shape, keeping its id, at a drawing-order position.
    ///
    /// # Errors
    ///
    /// Fails if the shape is invalid, the id is taken, or the index is past
    /// the end.
    pub fn insert_shape(&mut self, index: usize, shape: Shape) -> Result<(), ProjectError> {
        shape.validate()?;
        if self.shape(shape.id()).is_some() {
            return Err(ProjectError::invalid(
                "shape",
                format!("{} already exists", shape.id()),
            ));
        }
        if index > self.shapes.len() {
            return Err(ProjectError::invalid(
                "index",
                format!("{index} is past the end ({} shapes)", self.shapes.len()),
            ));
        }
        let highest = shape.groups().iter().copied().fold(shape.id(), u32::max);
        if highest >= self.next_id {
            self.set_next_id(highest.saturating_add(1));
        }
        self.shapes.insert(index, shape);
        Ok(())
    }

    /// Adds a copy of `shape` on top with a fresh id; returns the id.
    pub fn add_copy(&mut self, shape: &Shape) -> u32 {
        let mut copy = shape.clone();
        let id = self.allocate_id();
        copy.set_id(id);
        self.shapes.push(copy);
        id
    }

    /// Removes a shape, returning its former index and the shape.
    ///
    /// # Errors
    ///
    /// Fails if no shape has this id.
    pub fn remove_shape(&mut self, id: u32) -> Result<(usize, Shape), ProjectError> {
        let index = self.index_of(id).ok_or_else(|| unknown(id))?;
        self.view.selection.remove(&id);
        if self.view.anchor == Some(id) {
            self.view.anchor = None;
        }
        Ok((index, self.shapes.remove(index)))
    }

    /// Replaces a shape's transform.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id or an invalid transform.
    pub fn set_shape_transform(&mut self, id: u32, transform: Affine) -> Result<(), ProjectError> {
        validate_transform(&transform)?;
        let index = self.index_of(id).ok_or_else(|| unknown(id))?;
        self.shapes[index].set_transform(transform);
        Ok(())
    }

    /// Replaces a shape's geometry.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id or invalid geometry.
    pub fn set_shape_geometry(&mut self, id: u32, geometry: Geometry) -> Result<(), ProjectError> {
        geometry.validate()?;
        let index = self.index_of(id).ok_or_else(|| unknown(id))?;
        self.shapes[index].set_geometry(geometry);
        Ok(())
    }

    /// Sets the groups a shape belongs to, outermost first. Moves the id
    /// allocator past them so new ids never collide.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id or an invalid chain.
    pub fn set_shape_groups(&mut self, id: u32, groups: Vec<u32>) -> Result<(), ProjectError> {
        crate::shape::validate_groups(&groups)?;
        let index = self.index_of(id).ok_or_else(|| unknown(id))?;
        if let Some(highest) = groups.iter().max()
            && *highest >= self.next_id
        {
            self.set_next_id(highest.saturating_add(1));
        }
        self.shapes[index].set_groups(groups);
        Ok(())
    }

    /// Gives the shapes `ids` (fresh copies) new group ids: shapes that
    /// shared a group share its replacement. Ids are allocated in order of
    /// first appearance, so the result is deterministic.
    pub fn regroup(&mut self, ids: &[u32]) {
        let mut fresh: Vec<(u32, u32)> = Vec::new();
        for id in ids {
            let Some(index) = self.index_of(*id) else {
                continue;
            };
            let old = self.shapes[index].groups().to_vec();
            let mut new = Vec::with_capacity(old.len());
            for g in old {
                let mapped = if let Some((_, to)) = fresh.iter().find(|(from, _)| *from == g) {
                    *to
                } else {
                    let to = self.allocate_id();
                    fresh.push((g, to));
                    to
                };
                new.push(mapped);
            }
            self.shapes[index].set_groups(new);
        }
    }

    /// The top-level objects that `ids` belong to, each as all of its
    /// shapes in drawing order; objects ordered by their lowest shape.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id.
    pub fn objects_of(&self, ids: &[u32]) -> Result<Vec<Vec<u32>>, ProjectError> {
        let mut keys = Vec::new();
        for id in ids {
            let key = ObjectKey::of(self.shape(*id).ok_or_else(|| unknown(*id))?);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        let mut objects: Vec<(ObjectKey, Vec<u32>)> = Vec::new();
        for shape in &self.shapes {
            let key = ObjectKey::of(shape);
            if !keys.contains(&key) {
                continue;
            }
            match objects.iter_mut().find(|(k, _)| *k == key) {
                Some((_, members)) => members.push(shape.id()),
                None => objects.push((key, vec![shape.id()])),
            }
        }
        Ok(objects.into_iter().map(|(_, members)| members).collect())
    }

    /// `ids` widened to every shape of their top-level objects.
    fn widened(&self, ids: &BTreeSet<u32>) -> BTreeSet<u32> {
        let keys: HashSet<ObjectKey> = ids
            .iter()
            .filter_map(|id| self.shape(*id))
            .map(ObjectKey::of)
            .collect();
        self.shapes
            .iter()
            .filter(|s| keys.contains(&ObjectKey::of(s)))
            .map(Shape::id)
            .collect()
    }

    /// A shape of the anchor object: the object added to the selection
    /// last, else the first selected shape in drawing order.
    #[must_use]
    pub fn anchor(&self) -> Option<u32> {
        self.view
            .anchor
            .filter(|a| self.view.selection.contains(a))
            .or_else(|| self.selection().first().copied())
    }

    /// Moves a shape to the layer `color`.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id or a color outside 0–29.
    pub fn set_shape_layer(&mut self, id: u32, color: Color) -> Result<(), ProjectError> {
        if usize::from(color) >= LAYER_COUNT {
            return Err(ProjectError::invalid("layer", "must be a color 0–29"));
        }
        let index = self.index_of(id).ok_or_else(|| unknown(id))?;
        self.shapes[index].set_layer(color);
        Ok(())
    }

    /// The current drawing order as ids.
    #[must_use]
    pub fn order(&self) -> Vec<u32> {
        self.shapes.iter().map(Shape::id).collect()
    }

    /// Reorders shapes.
    ///
    /// # Errors
    ///
    /// Fails unless `ids` names every shape exactly once.
    pub fn set_order(&mut self, ids: &[u32]) -> Result<(), ProjectError> {
        let current: HashSet<u32> = self.shapes.iter().map(Shape::id).collect();
        let wanted: HashSet<u32> = ids.iter().copied().collect();
        if ids.len() != self.shapes.len() || wanted != current {
            return Err(ProjectError::invalid(
                "order",
                "must list every shape exactly once",
            ));
        }
        let mut taken = std::mem::take(&mut self.shapes);
        for id in ids {
            if let Some(i) = taken.iter().position(|s| s.id() == *id) {
                self.shapes.push(taken.swap_remove(i));
            }
        }
        Ok(())
    }

    /// Combined bounds of the given shapes, or `None` if `ids` is empty.
    ///
    /// # Errors
    ///
    /// Fails for an unknown id.
    pub fn bounds_of(&self, ids: &[u32]) -> Result<Option<Rect>, ProjectError> {
        let mut bounds: Option<Rect> = None;
        for id in ids {
            let b = self.shape(*id).ok_or_else(|| unknown(*id))?.bounds();
            bounds = Some(bounds.map_or(b, |r| r.union(b)));
        }
        Ok(bounds)
    }

    /// Whether a shape's layer is shown. Hidden shapes cannot be picked.
    #[must_use]
    pub fn is_visible(&self, shape: &Shape) -> bool {
        self.layer(shape.layer()).map_or(true, |l| l.show)
    }

    /// The topmost visible shape hit at `p` within `tolerance` mm.
    #[must_use]
    pub fn shape_at(&self, p: Point, tolerance: f64) -> Option<u32> {
        self.shapes
            .iter()
            .rev()
            .filter(|s| self.is_visible(s))
            .find(|s| s.hit(p, tolerance))
            .map(Shape::id)
    }

    /// Visible shapes picked by a selection box, in drawing order.
    #[must_use]
    pub fn shapes_in_rect(&self, rect: &Rect, mode: BoxMode, tolerance: f64) -> Vec<u32> {
        self.shapes
            .iter()
            .filter(|s| self.is_visible(s))
            .filter(|s| match mode {
                BoxMode::Contained => rect.contains_rect(&s.bounds()),
                BoxMode::Touching => s.touches(rect, tolerance),
            })
            .map(Shape::id)
            .collect()
    }

    /// Selected shape ids in drawing order.
    #[must_use]
    pub fn selection(&self) -> Vec<u32> {
        self.shapes
            .iter()
            .map(Shape::id)
            .filter(|id| self.view.selection.contains(id))
            .collect()
    }

    /// Whether a shape is selected.
    #[must_use]
    pub fn is_selected(&self, id: u32) -> bool {
        self.view.selection.contains(&id)
    }

    /// Replaces the selection.
    ///
    /// # Errors
    ///
    /// Fails, leaving the selection alone, if an id is unknown.
    pub fn set_selection(
        &mut self,
        ids: impl IntoIterator<Item = u32>,
    ) -> Result<(), ProjectError> {
        let ids: BTreeSet<u32> = ids.into_iter().collect();
        if let Some(missing) = ids.iter().find(|id| self.shape(**id).is_none()) {
            return Err(unknown(*missing));
        }
        let ids = self.widened(&ids);
        let added = self
            .shapes
            .iter()
            .map(Shape::id)
            .find(|id| ids.contains(id) && !self.view.selection.contains(id));
        if added.is_some() {
            self.view.anchor = added;
        }
        self.view.selection = ids;
        Ok(())
    }

    /// The clipboard contents.
    #[must_use]
    pub fn clipboard(&self) -> &[Shape] {
        &self.view.clipboard
    }

    /// Replaces the clipboard.
    pub fn set_clipboard(&mut self, shapes: Vec<Shape>) {
        self.view.clipboard = shapes;
    }

    pub(crate) fn validate_shapes(&self) -> Result<(), ProjectError> {
        let mut seen = HashSet::new();
        for shape in &self.shapes {
            shape.validate()?;
            if !seen.insert(shape.id()) {
                return Err(ProjectError::invalid(
                    "shape",
                    format!("id {} is used twice", shape.id()),
                ));
            }
            if shape.id() >= self.next_id {
                return Err(ProjectError::invalid(
                    "next_id",
                    format!("must be greater than every shape id ({})", shape.id()),
                ));
            }
            if let Some(g) = shape.groups().iter().find(|g| **g >= self.next_id) {
                return Err(ProjectError::invalid(
                    "next_id",
                    format!("must be greater than every group id ({g})"),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(p: &mut Project, x: f64, y: f64) -> u32 {
        p.add_shape(
            Geometry::Rect {
                width: 10.0,
                height: 10.0,
            },
            Affine::translate(x, y),
        )
        .unwrap()
    }

    #[test]
    fn ids_are_fresh_and_insert_restores_exactly() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let b = square(&mut p, 20.0, 0.0);
        assert_eq!((a, b), (1, 2));
        let (index, shape) = p.remove_shape(a).unwrap();
        assert_eq!(index, 0);
        assert!(p.insert_shape(5, shape.clone()).is_err());
        p.insert_shape(index, shape.clone()).unwrap();
        assert_eq!(p.order(), [a, b]);
        assert!(p.insert_shape(0, shape).is_err(), "duplicate id");
        assert_eq!(square(&mut p, 0.0, 0.0), 3);
    }

    #[test]
    fn hit_testing_prefers_the_top_shape() {
        let mut p = Project::default();
        let below = square(&mut p, 0.0, 0.0);
        let above = square(&mut p, 5.0, 5.0);
        assert_eq!(p.shape_at(Point::new(7.0, 7.0), 0.1), Some(above));
        assert_eq!(p.shape_at(Point::new(2.0, 2.0), 0.1), Some(below));
        assert_eq!(p.shape_at(Point::new(50.0, 50.0), 0.1), None);
    }

    #[test]
    fn box_selection_modes() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let b = square(&mut p, 20.0, 0.0);
        let r = Rect::from_corners(Point::new(-1.0, -1.0), Point::new(25.0, 11.0));
        assert_eq!(p.shapes_in_rect(&r, BoxMode::Contained, 0.01), [a]);
        assert_eq!(p.shapes_in_rect(&r, BoxMode::Touching, 0.01), [a, b]);
    }

    #[test]
    fn hidden_layers_cannot_be_picked() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        p.set_layer_setting(0, "show", "off").unwrap();
        assert_eq!(p.shape_at(Point::new(5.0, 5.0), 0.1), None);
        let all = Rect::from_corners(Point::new(-1.0, -1.0), Point::new(20.0, 20.0));
        assert!(p.shapes_in_rect(&all, BoxMode::Touching, 0.01).is_empty());
        p.set_layer_setting(0, "show", "on").unwrap();
        assert_eq!(p.shape_at(Point::new(5.0, 5.0), 0.1), Some(a));
    }

    #[test]
    fn order_must_be_a_permutation() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let b = square(&mut p, 0.0, 0.0);
        p.set_order(&[b, a]).unwrap();
        assert_eq!(p.order(), [b, a]);
        assert!(p.set_order(&[a]).is_err());
        assert!(p.set_order(&[a, a]).is_err());
        assert!(p.set_order(&[a, 99]).is_err());
        assert_eq!(p.order(), [b, a]);
    }

    #[test]
    fn selection_is_view_state() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let before = p.clone();
        p.set_selection([a]).unwrap();
        assert_eq!(p, before, "selection does not affect equality");
        assert!(p.set_selection([a, 42]).is_err());
        assert_eq!(p.selection(), [a]);
        p.remove_shape(a).unwrap();
        assert!(p.selection().is_empty());
    }

    #[test]
    fn selection_widens_to_groups_and_tracks_the_anchor() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let b = square(&mut p, 20.0, 0.0);
        let c = square(&mut p, 40.0, 0.0);
        let g = p.allocate_id();
        p.set_shape_groups(a, vec![g]).unwrap();
        p.set_shape_groups(b, vec![g]).unwrap();
        p.set_selection([b]).unwrap();
        assert_eq!(p.selection(), [a, b]);
        assert_eq!(p.anchor(), Some(a), "first of the added shapes");
        p.set_selection([a, b, c]).unwrap();
        assert_eq!(p.anchor(), Some(c), "last added object");
        p.set_selection([a, b]).unwrap();
        assert_eq!(p.anchor(), Some(a), "falls back once the anchor leaves");
        assert_eq!(p.objects_of(&[c, b]).unwrap(), [vec![a, b], vec![c]]);
        assert!(p.objects_of(&[99]).is_err());
    }

    #[test]
    fn group_chains_are_validated_and_regrouped() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        assert!(p.set_shape_groups(a, vec![0]).is_err());
        assert!(p.set_shape_groups(a, vec![5, 5]).is_err());
        p.set_shape_groups(a, vec![10, 7]).unwrap();
        assert_eq!(p.next_id(), 11, "allocator moves past group ids");
        let copy = p.add_copy(&p.shape(a).unwrap().clone());
        p.regroup(&[copy]);
        assert_eq!(p.shape(copy).unwrap().groups(), [12, 13]);
        let json = p.to_json();
        assert!(json.contains("\"groups\": [10, 7]"), "{json}");
        assert_eq!(Project::from_json(&json).unwrap(), p);
        let broken = json.replace("\"next_id\": 14", "\"next_id\": 12");
        assert!(Project::from_json(&broken).is_err());
    }

    #[test]
    fn geometry_can_be_replaced_and_converted() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        let path = p.shape(a).unwrap().geometry().to_path();
        assert_eq!(path.subpaths[0].segments.len(), 3);
        p.set_shape_geometry(a, Geometry::Path(path)).unwrap();
        assert_eq!(p.shape(a).unwrap().geometry().kind(), "path");
        assert!(
            p.set_shape_geometry(a, Geometry::Path(lasercraft_geom::Path::default()))
                .is_err()
        );
        let e = Geometry::Ellipse { rx: 2.0, ry: 1.0 }.to_path();
        let b = e.bounds().unwrap();
        assert!(b.max.approx_eq(Point::new(2.0, 1.0), 1e-9));
    }

    #[test]
    fn transforms_are_validated() {
        let mut p = Project::default();
        let a = square(&mut p, 0.0, 0.0);
        assert!(p.set_shape_transform(a, Affine::scale(0.0, 1.0)).is_err());
        assert!(p.set_shape_transform(9, Affine::IDENTITY).is_err());
        p.set_shape_transform(a, Affine::translate(1.0, 2.0))
            .unwrap();
        assert_eq!(
            p.bounds_of(&[a]).unwrap().unwrap().min,
            Point::new(1.0, 2.0)
        );
        assert_eq!(p.bounds_of(&[]).unwrap(), None);
    }
}
