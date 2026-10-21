//! Geometry for LaserCraft: points, affine transforms and bounding boxes.
//!
//! All coordinates are millimetres in document space: x grows to the right and
//! y grows downwards, as in SVG (`docs/spec/geometry.md`). Everything here is
//! pure and deterministic.
//!
//! Paths, Béziers, flattening, booleans and offsets arrive with M1–M4.

mod affine;
mod point;
mod rect;

pub use affine::Affine;
pub use point::Point;
pub use rect::Rect;

/// Tolerance used when comparing lengths, in millimetres.
pub const EPSILON: f64 = 1e-9;
