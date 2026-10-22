//! Cameras for LaserCraft (`docs/spec/camera.md`): pictures, the lens
//! model and its calibration from pictures of the calibration sheet, the
//! bed alignment from burned marks, and the overlay that puts a picture on
//! the bed. Everything here is pure and deterministic; capturing pictures
//! happens in the app.

pub mod align;
pub mod calibrate;
pub mod dots;
pub mod homography;
pub mod lens;
mod linalg;
pub mod marks;
pub mod overlay;
pub mod picture;
pub mod sheet;
pub mod synth;

pub use homography::Homography;
pub use lens::{Lens, Model};
pub use picture::Picture;

/// Why a camera step failed, in words for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraError(pub String);

impl CameraError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CameraError {}
