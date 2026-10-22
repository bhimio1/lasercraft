//! Geometry for LaserCraft: points, affine transforms and bounding boxes.
//!
//! All coordinates are millimetres in document space: x grows to the right and
//! y grows downwards, as in SVG (`docs/spec/geometry.md`). Everything here is
//! pure and deterministic.
//!
//! Polylines, ellipses and Bézier paths; booleans and offsets through
//! `i_overlay`.

mod affine;
pub mod boolean;
pub mod flatten;
pub mod manipulate;
pub mod nodes;
pub mod path;
mod point;
pub mod polyline;
mod rect;
pub mod snap;
pub mod taper;
pub mod trace;

pub use affine::Affine;
pub use path::{Path, Segment, SubPath};
pub use point::Point;
pub use rect::Rect;

/// Tolerance used when comparing lengths, in millimetres.
pub const EPSILON: f64 = 1e-9;
