//! Shapes: an id, local geometry and a transform (`docs/spec/shapes.md`).

use lasercraft_geom::{Affine, Path, Point, Rect, SubPath, flatten, polyline};
use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::image::Image;
use crate::layer::{Color, LAYER_COUNT};

/// Largest allowed coordinate or size, in millimetres. Keeps every value far
/// from overflow while allowing generous off-bed scratch space.
pub const MAX_COORD_MM: f64 = 1.0e6;

/// Most points a polyline may have.
pub const MAX_POINTS: usize = 1_000_000;

/// A shape's geometry in its own coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Geometry {
    /// Rectangle with corners (0, 0) and (`width`, `height`).
    Rect {
        /// Width in mm.
        width: f64,
        /// Height in mm.
        height: f64,
    },
    /// Ellipse centered on the origin.
    Ellipse {
        /// Horizontal radius in mm.
        rx: f64,
        /// Vertical radius in mm.
        ry: f64,
    },
    /// Straight segments through `points`; a line has two points.
    Polyline {
        /// The vertices, at least two.
        points: Vec<Point>,
        /// Whether the last point joins back to the first.
        closed: bool,
    },
    /// Lines and cubic Béziers in one or more subpaths: imported art, text,
    /// edited nodes and boolean results.
    Path(Path),
    /// A grayscale bitmap on the rectangle (0, 0)–(width, height)
    /// (`docs/spec/images.md`, M7).
    Image(Image),
}

fn check_size(field: &str, value: f64) -> Result<(), ProjectError> {
    if value.is_finite() && value > 0.0 && value <= MAX_COORD_MM {
        Ok(())
    } else {
        Err(ProjectError::invalid(
            field,
            format!("must be greater than 0 and at most {MAX_COORD_MM} mm, got {value}"),
        ))
    }
}

fn check_point(p: Point) -> Result<(), ProjectError> {
    if p.is_finite() && p.x.abs() <= MAX_COORD_MM && p.y.abs() <= MAX_COORD_MM {
        Ok(())
    } else {
        Err(ProjectError::invalid(
            "point",
            format!(
                "({}, {}) is not a finite coordinate within ±{MAX_COORD_MM} mm",
                p.x, p.y
            ),
        ))
    }
}

impl Geometry {
    /// Checks sizes and point counts.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] describing the first problem.
    pub fn validate(&self) -> Result<(), ProjectError> {
        match self {
            Self::Rect { width, height } => {
                check_size("width", *width)?;
                check_size("height", *height)
            }
            Self::Ellipse { rx, ry } => {
                check_size("x radius", *rx)?;
                check_size("y radius", *ry)
            }
            Self::Polyline { points, .. } => {
                if points.len() < 2 {
                    return Err(ProjectError::invalid("polyline", "needs at least 2 points"));
                }
                if points.len() > MAX_POINTS {
                    return Err(ProjectError::invalid(
                        "polyline",
                        format!("has more than {MAX_POINTS} points"),
                    ));
                }
                points.iter().try_for_each(|p| check_point(*p))
            }
            Self::Path(path) => {
                if !path.is_valid() {
                    return Err(ProjectError::invalid(
                        "path",
                        "needs at least one segment in every subpath and finite points",
                    ));
                }
                if path.points().count() > MAX_POINTS {
                    return Err(ProjectError::invalid(
                        "path",
                        format!("has more than {MAX_POINTS} points"),
                    ));
                }
                path.points().try_for_each(check_point)
            }
            Self::Image(image) => {
                check_size("width", image.width)?;
                check_size("height", image.height)?;
                image.validate_adjust()
            }
        }
    }

    /// Whether this is an image (no vector outline).
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    /// The image, if this is one.
    #[must_use]
    pub fn image(&self) -> Option<&Image> {
        match self {
            Self::Image(image) => Some(image),
            _ => None,
        }
    }

    /// The local rectangle of a rect or image: (0, 0)–(width, height).
    fn frame(width: f64, height: f64) -> Vec<Point> {
        vec![
            Point::new(0.0, 0.0),
            Point::new(width, 0.0),
            Point::new(width, height),
            Point::new(0.0, height),
        ]
    }

    /// The outline used for selection and hit testing: the vector outlines,
    /// or an image's rectangle.
    #[must_use]
    pub fn local_frame(&self, tolerance: f64) -> Vec<(Vec<Point>, bool)> {
        match self {
            Self::Image(image) => vec![(Self::frame(image.width, image.height), true)],
            other => other.local_outlines(tolerance),
        }
    }

    /// Whether the outline is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Rect { .. } | Self::Ellipse { .. } => true,
            Self::Polyline { closed, .. } => *closed,
            Self::Path(path) => path.is_closed(),
            Self::Image(_) => false,
        }
    }

    /// The outlines in local coordinates, within `tolerance` mm of the true
    /// curve, each with its closed flag. Only paths have more than one;
    /// images have none.
    #[must_use]
    pub fn local_outlines(&self, tolerance: f64) -> Vec<(Vec<Point>, bool)> {
        match self {
            Self::Rect { width, height } => vec![(Self::frame(*width, *height), true)],
            Self::Image(_) => Vec::new(),
            Self::Ellipse { rx, ry } => vec![(flatten::ellipse_points(*rx, *ry, tolerance), true)],
            Self::Polyline { points, closed } => vec![(points.clone(), *closed)],
            Self::Path(path) => path.flatten(tolerance),
        }
    }

    /// The same outline as a path: rectangles and polylines become lines,
    /// ellipses four cubic quarter arcs (`docs/spec/nodes.md`).
    #[must_use]
    pub fn to_path(&self) -> Path {
        match self {
            Self::Rect { width, height } => Path::from_contours(&[vec![
                Point::new(0.0, 0.0),
                Point::new(*width, 0.0),
                Point::new(*width, *height),
                Point::new(0.0, *height),
            ]])
            .unwrap_or_default(),
            Self::Ellipse { rx, ry } => Path::ellipse(*rx, *ry),
            Self::Polyline { points, closed } => SubPath::from_points(points, *closed)
                .map(Path::single)
                .unwrap_or_default(),
            Self::Path(path) => path.clone(),
            Self::Image(_) => Path::default(),
        }
    }

    /// Short name of the kind, for listings.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Rect { .. } => "rect",
            Self::Ellipse { .. } => "ellipse",
            Self::Polyline { points, closed } => match (points.len(), closed) {
                (2, false) => "line",
                (_, true) => "polygon",
                _ => "polyline",
            },
            Self::Path(_) => "path",
            Self::Image(_) => "image",
        }
    }
}

/// A shape in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    id: u32,
    /// Palette color of the shape's layer.
    #[serde(default)]
    layer: Color,
    /// Groups the shape belongs to, outermost first (`docs/spec/arrange.md`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    groups: Vec<u32>,
    geometry: Geometry,
    transform: Affine,
}

/// Checks that a transform is finite, invertible and moves nothing absurdly far.
///
/// # Errors
///
/// Returns [`ProjectError::InvalidValue`] if not.
pub fn validate_transform(t: &Affine) -> Result<(), ProjectError> {
    if !t.is_finite() || t.m.iter().any(|v| v.abs() > MAX_COORD_MM) {
        return Err(ProjectError::invalid(
            "transform",
            "must be finite and within range",
        ));
    }
    if t.inverse().is_none() {
        return Err(ProjectError::invalid(
            "transform",
            "would flatten the shape to a line or point",
        ));
    }
    Ok(())
}

/// Checks a group chain: ids at least 1, none repeated.
///
/// # Errors
///
/// Returns [`ProjectError::InvalidValue`] if not.
pub fn validate_groups(groups: &[u32]) -> Result<(), ProjectError> {
    if groups.contains(&0) {
        return Err(ProjectError::invalid("group", "ids must be at least 1"));
    }
    let unique: std::collections::HashSet<u32> = groups.iter().copied().collect();
    if unique.len() != groups.len() {
        return Err(ProjectError::invalid(
            "group",
            "a shape lists a group twice",
        ));
    }
    Ok(())
}

impl Shape {
    /// A validated shape.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] for bad geometry, a bad
    /// transform, or id 0.
    pub fn new(id: u32, geometry: Geometry, transform: Affine) -> Result<Self, ProjectError> {
        let shape = Self {
            id,
            layer: 0,
            groups: Vec::new(),
            geometry,
            transform,
        };
        shape.validate()?;
        Ok(shape)
    }

    pub(crate) fn validate(&self) -> Result<(), ProjectError> {
        if self.id == 0 {
            return Err(ProjectError::invalid("shape id", "must be at least 1"));
        }
        if usize::from(self.layer) >= LAYER_COUNT {
            return Err(ProjectError::invalid("layer", "must be a color 0–29"));
        }
        validate_groups(&self.groups)?;
        self.geometry.validate()?;
        validate_transform(&self.transform)
    }

    /// Groups the shape belongs to, outermost first; empty when ungrouped.
    #[must_use]
    pub fn groups(&self) -> &[u32] {
        &self.groups
    }

    /// The outermost group, if any.
    #[must_use]
    pub fn top_group(&self) -> Option<u32> {
        self.groups.first().copied()
    }

    pub(crate) fn set_groups(&mut self, groups: Vec<u32>) {
        self.groups = groups;
    }

    pub(crate) fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    /// Identifier, unique in the project.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Palette color of the shape's layer.
    #[must_use]
    pub fn layer(&self) -> Color {
        self.layer
    }

    pub(crate) fn set_layer(&mut self, layer: Color) {
        self.layer = layer;
    }

    /// Local geometry.
    #[must_use]
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Local-to-document transform.
    #[must_use]
    pub fn transform(&self) -> &Affine {
        &self.transform
    }

    pub(crate) fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub(crate) fn set_transform(&mut self, transform: Affine) {
        self.transform = transform;
    }

    /// The outlines in document coordinates within `tolerance` mm, each
    /// with its closed flag.
    #[must_use]
    pub fn outlines(&self, tolerance: f64) -> Vec<(Vec<Point>, bool)> {
        let local_tolerance = tolerance / self.transform.max_stretch().max(f64::EPSILON);
        self.geometry
            .local_outlines(local_tolerance)
            .into_iter()
            .map(|(points, closed)| {
                let points = points
                    .into_iter()
                    .map(|p| self.transform.apply(p))
                    .collect();
                (points, closed)
            })
            .collect()
    }

    /// The selection outline in document coordinates: the vector outlines,
    /// or an image's rectangle.
    #[must_use]
    pub fn frame(&self, tolerance: f64) -> Vec<(Vec<Point>, bool)> {
        let local_tolerance = tolerance / self.transform.max_stretch().max(f64::EPSILON);
        self.geometry
            .local_frame(local_tolerance)
            .into_iter()
            .map(|(points, closed)| {
                let points = points
                    .into_iter()
                    .map(|p| self.transform.apply(p))
                    .collect();
                (points, closed)
            })
            .collect()
    }

    /// The closed outlines in document coordinates (open ones are left out).
    #[must_use]
    pub fn closed_contours(&self, tolerance: f64) -> Vec<Vec<Point>> {
        self.outlines(tolerance)
            .into_iter()
            .filter_map(|(points, closed)| closed.then_some(points))
            .collect()
    }

    /// Exact bounding box in document coordinates.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        let t = &self.transform;
        match &self.geometry {
            Geometry::Ellipse { rx, ry } => {
                let [a, b, c, d, ..] = t.m;
                let half = Point::new((a * rx).hypot(c * ry), (b * rx).hypot(d * ry));
                let center = t.apply(Point::ZERO);
                Rect::from_corners(center - half, center + half)
            }
            Geometry::Rect { width, height } => {
                Rect::from_corners(Point::ZERO, Point::new(*width, *height)).transformed(t)
            }
            Geometry::Image(image) => {
                Rect::from_corners(Point::ZERO, Point::new(image.width, image.height))
                    .transformed(t)
            }
            Geometry::Polyline { points, .. } => Rect::bounding(points.iter().map(|p| t.apply(*p)))
                .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO)),
            Geometry::Path(path) => path
                .transformed(t)
                .bounds()
                .unwrap_or_else(|| Rect::from_corners(Point::ZERO, Point::ZERO)),
        }
    }

    /// Whether `p` is within `tolerance` of the outline, or inside a closed
    /// shape.
    #[must_use]
    pub fn hit(&self, p: Point, tolerance: f64) -> bool {
        let b = self.bounds();
        let grown = Rect::from_corners(
            b.min - Point::new(tolerance, tolerance),
            b.max + Point::new(tolerance, tolerance),
        );
        if !grown.contains(p) {
            return false;
        }
        let outlines = self.frame(tolerance * 0.25);
        // Even-odd across the closed outlines, so holes are not hits.
        let inside = outlines
            .iter()
            .filter(|(points, closed)| *closed && polyline::contains(points, p))
            .count()
            % 2
            == 1;
        inside
            || outlines.iter().any(|(points, closed)| {
                polyline::distance_to_polyline(points, *closed, p) <= tolerance
            })
    }

    /// Whether an outline crosses or lies inside `rect`, or the shape's
    /// inside (even-odd across its closed outlines) contains the box.
    #[must_use]
    pub fn touches(&self, rect: &Rect, tolerance: f64) -> bool {
        let outlines = self.frame(tolerance);
        // Points inside or segments crossing, without containment; the
        // closing segment is checked separately.
        let crosses = outlines.iter().any(|(points, closed)| {
            polyline::touches_rect(points, false, rect) || (*closed && edges_touch(points, rect))
        });
        let inside = outlines
            .iter()
            .filter(|(points, closed)| *closed && polyline::contains(points, rect.center()))
            .count()
            % 2
            == 1;
        crosses || inside
    }
}

/// Whether a closed outline's segments (including the closing one) touch
/// `rect`, ignoring containment.
fn edges_touch(points: &[Point], rect: &Rect) -> bool {
    let corners = rect.corners();
    let edges = [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[3]),
        (corners[3], corners[0]),
    ];
    polyline::segments(points, true).any(|(a, b)| {
        edges
            .iter()
            .any(|(c, d)| polyline::segments_intersect(a, b, *c, *d))
    })
}

/// Snap targets of a box: corners, edge midpoints and center.
#[must_use]
pub fn snap_points(bounds: &Rect) -> [Point; 9] {
    let [a, b, c, d] = bounds.corners();
    [
        a,
        b,
        c,
        d,
        (a + b) * 0.5,
        (b + c) * 0.5,
        (c + d) * 0.5,
        (d + a) * 0.5,
        bounds.center(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    fn rect(w: f64, h: f64, t: Affine) -> Shape {
        Shape::new(
            1,
            Geometry::Rect {
                width: w,
                height: h,
            },
            t,
        )
        .unwrap()
    }

    #[test]
    fn validation_rejects_bad_geometry_and_transforms() {
        let bad = [
            Geometry::Rect {
                width: 0.0,
                height: 1.0,
            },
            Geometry::Ellipse {
                rx: 1.0,
                ry: f64::NAN,
            },
            Geometry::Polyline {
                points: vec![Point::ZERO],
                closed: false,
            },
            Geometry::Polyline {
                points: vec![Point::ZERO, Point::new(f64::INFINITY, 0.0)],
                closed: false,
            },
        ];
        for g in bad {
            assert!(Shape::new(1, g.clone(), Affine::IDENTITY).is_err(), "{g:?}");
        }
        let ok = Geometry::Rect {
            width: 1.0,
            height: 1.0,
        };
        assert!(Shape::new(0, ok.clone(), Affine::IDENTITY).is_err());
        assert!(Shape::new(1, ok.clone(), Affine::scale(0.0, 1.0)).is_err());
        assert!(Shape::new(1, ok, Affine::translate(f64::NAN, 0.0)).is_err());
    }

    #[test]
    fn rotated_ellipse_bounds_are_exact() {
        let e = Shape::new(
            1,
            Geometry::Ellipse { rx: 20.0, ry: 10.0 },
            Affine::translate(50.0, 50.0) * Affine::rotate_degrees(90.0),
        )
        .unwrap();
        let b = e.bounds();
        assert!(b.min.approx_eq(Point::new(40.0, 30.0), TOL), "{b:?}");
        assert!(b.max.approx_eq(Point::new(60.0, 70.0), TOL), "{b:?}");
        // Flattened outline stays inside the exact bounds.
        let (pts, closed) = e.outlines(0.01).remove(0);
        assert!(closed);
        let grown = Rect::from_corners(b.min - Point::new(TOL, TOL), b.max + Point::new(TOL, TOL));
        assert!(pts.iter().all(|p| grown.contains(*p)));
    }

    #[test]
    fn rect_bounds_follow_transform() {
        let r = rect(
            10.0,
            5.0,
            Affine::translate(3.0, 4.0) * Affine::scale(-1.0, 2.0),
        );
        let b = r.bounds();
        assert!(b.min.approx_eq(Point::new(-7.0, 4.0), TOL));
        assert!(b.max.approx_eq(Point::new(3.0, 14.0), TOL));
    }

    #[test]
    fn hits_inside_closed_and_near_open_shapes() {
        let r = rect(10.0, 10.0, Affine::IDENTITY);
        assert!(r.hit(Point::new(5.0, 5.0), 0.5));
        assert!(r.hit(Point::new(10.3, 5.0), 0.5));
        assert!(!r.hit(Point::new(11.0, 5.0), 0.5));
        let line = Shape::new(
            2,
            Geometry::Polyline {
                points: vec![Point::ZERO, Point::new(10.0, 10.0)],
                closed: false,
            },
            Affine::IDENTITY,
        )
        .unwrap();
        assert!(line.hit(Point::new(5.2, 5.0), 0.5));
        assert!(!line.hit(Point::new(2.0, 8.0), 0.5));
        assert_eq!(line.geometry().kind(), "line");
    }

    #[test]
    fn path_shapes_round_trip_and_hit_test_with_holes() {
        use lasercraft_geom::{Segment, SubPath};
        let outer = SubPath::from_points(
            &[
                Point::new(0.0, 0.0),
                Point::new(20.0, 0.0),
                Point::new(20.0, 20.0),
                Point::new(0.0, 20.0),
            ],
            true,
        )
        .unwrap();
        let hole = SubPath {
            start: Point::new(5.0, 5.0),
            segments: vec![
                Segment::Line(Point::new(15.0, 5.0)),
                Segment::Cubic(
                    Point::new(15.0, 10.0),
                    Point::new(15.0, 15.0),
                    Point::new(10.0, 15.0),
                ),
                Segment::Line(Point::new(5.0, 15.0)),
            ],
            closed: true,
        };
        let path = Path {
            subpaths: vec![outer, hole],
        };
        let shape = Shape::new(1, Geometry::Path(path), Affine::translate(100.0, 0.0)).unwrap();
        assert_eq!(shape.geometry().kind(), "path");
        let json = serde_json::to_string(&shape).unwrap();
        assert!(json.contains(r#""kind":"path","subpaths""#), "{json}");
        assert_eq!(serde_json::from_str::<Shape>(&json).unwrap(), shape);
        let b = shape.bounds();
        assert_eq!(
            (b.min, b.max),
            (Point::new(100.0, 0.0), Point::new(120.0, 20.0))
        );
        // Solid ring is a hit; the hole is not, except near its edge.
        assert!(shape.hit(Point::new(102.0, 2.0), 0.1));
        assert!(!shape.hit(Point::new(108.0, 10.0), 0.1));
        assert!(shape.hit(Point::new(105.05, 10.0), 0.1));
        assert_eq!(shape.outlines(0.01).len(), 2);
        assert_eq!(shape.closed_contours(0.01).len(), 2);
        let empty = Geometry::Path(Path::default());
        assert!(Shape::new(2, empty, Affine::IDENTITY).is_err());
    }

    #[test]
    fn serializes_compactly() {
        let line = Shape::new(
            7,
            Geometry::Polyline {
                points: vec![Point::new(1.0, 2.0), Point::new(3.0, 4.0)],
                closed: false,
            },
            Affine::IDENTITY,
        )
        .unwrap();
        let json = serde_json::to_string(&line).unwrap();
        assert_eq!(
            json,
            r#"{"id":7,"layer":0,"geometry":{"kind":"polyline","points":[[1.0,2.0],[3.0,4.0]],"closed":false},"transform":[1.0,0.0,0.0,1.0,0.0,0.0]}"#
        );
        assert_eq!(serde_json::from_str::<Shape>(&json).unwrap(), line);
        let unknown = json.replace("\"closed\"", "\"extra\":1,\"closed\"");
        assert!(serde_json::from_str::<Shape>(&unknown).is_err());
    }
}
