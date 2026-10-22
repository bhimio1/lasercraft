//! Images for LaserCraft (`docs/spec/images.md`): grayscale bitmaps, PNG and
//! JPEG decoding, adjustments, and turning an image into a grid of dot
//! powers with one of the dithering methods. Pure and deterministic; no
//! workspace dependencies.

mod adjust;
mod bitmap;
mod codec;
mod dither;
mod raster;
pub mod trace;

pub use adjust::Adjust;
pub use bitmap::{Gray, MAX_PIXELS, MAX_SIDE, base64_decode, base64_encode};
pub use codec::{Decoded, decode, encode_png};
pub use dither::{Method, dither};
pub use raster::{MAX_CELLS, Raster, rasterize};

use std::fmt;

/// Why an image could not be read or rasterized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterError(pub String);

impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RasterError {}

fn fail(reason: impl Into<String>) -> RasterError {
    RasterError(reason.into())
}
