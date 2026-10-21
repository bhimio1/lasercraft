//! Shapes: an id, local geometry and a transform (`docs/spec/shapes.md`).

use lasercraft_geom::{Affine, Point, Rect, flatten, polyline};
use serde::{Deserialize, Serialize};

use crate::ProjectError;

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
        }
    }

    /// Whether the outline is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Rect { .. } | Self::Ellipse { .. } => true,
            Self::Polyline { closed, .. } => *closed,
        }
    }

    /// The outline in local coordinates, within `tolerance` mm of the true
    /// curve.
    #[must_use]
    pub fn local_outline(&self, tolerance: f64) -> Vec<Point> {
        match self {
            Self::Rect { width, height } => vec![
                Point::new(0.0, 0.0),
                Point::new(*width, 0.0),
                Point::new(*width, *height),
                Point::new(0.0, *height),
            ],
            Self::Ellipse { rx, ry } => flatten::ellipse_points(*rx, *ry, tolerance),
            Self::Polyline { points, .. } => points.clone(),
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
        }
    }
}

/// A shape in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    id: u32,
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
        self.geometry.validate()?;
        validate_transform(&self.transform)
    }

    /// Identifier, unique in the project.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
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

    /// The outline in document coordinates within `tolerance` mm, and
    /// whether it is closed.
    #[must_use]
    pub fn outline(&self, tolerance: f64) -> (Vec<Point>, bool) {
        let local_tolerance = tolerance / self.transform.max_stretch().max(f64::EPSILON);
        let points = self
            .geometry
            .local_outline(local_tolerance)
            .into_iter()
            .map(|p| self.transform.apply(p))
            .collect();
        (points, self.geometry.is_closed())
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
            Geometry::Polyline { points, .. } => Rect::bounding(points.iter().map(|p| t.apply(*p)))
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
        let (points, closed) = self.outline(tolerance * 0.25);
        (closed && polyline::contains(&points, p))
            || polyline::distance_to_polyline(&points, closed, p) <= tolerance
    }

    /// Whether the outline crosses or lies inside `rect`, or a closed shape
    /// contains it.
    #[must_use]
    pub fn touches(&self, rect: &Rect, tolerance: f64) -> bool {
        let (points, closed) = self.outline(tolerance);
        polyline::touches_rect(&points, closed, rect)
    }
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
        let (pts, closed) = e.outline(0.01);
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
            r#"{"id":7,"geometry":{"kind":"polyline","points":[[1.0,2.0],[3.0,4.0]],"closed":false},"transform":[1.0,0.0,0.0,1.0,0.0,0.0]}"#
        );
        assert_eq!(serde_json::from_str::<Shape>(&json).unwrap(), line);
        let unknown = json.replace("\"closed\"", "\"extra\":1,\"closed\"");
        assert!(serde_json::from_str::<Shape>(&unknown).is_err());
    }
}
