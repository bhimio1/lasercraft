//! Image geometry (`docs/spec/images.md`): a grayscale bitmap spanning a
//! rectangle, with its own adjustments.

use std::sync::{Arc, OnceLock};

use lasercraft_raster::{Adjust, Gray, base64_decode, base64_encode, decode, encode_png};
use serde::{Deserialize, Serialize};

use crate::ProjectError;

/// An image: `width` × `height` mm locally, showing `bitmap`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "ImageFile", into = "ImageFile")]
pub struct Image {
    /// Width in mm.
    pub width: f64,
    /// Height in mm.
    pub height: f64,
    /// The pixels, shared between copies.
    pub bitmap: Arc<Gray>,
    /// Brightness, contrast and gamma.
    pub adjust: Adjust,
    /// The PNG text for the file, made once per bitmap.
    encoded: Arc<OnceLock<String>>,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.adjust == other.adjust
            && (Arc::ptr_eq(&self.bitmap, &other.bitmap) || self.bitmap == other.bitmap)
    }
}

impl Image {
    /// An image of `bitmap` spanning `width` × `height` mm.
    #[must_use]
    pub fn new(width: f64, height: f64, bitmap: Gray) -> Self {
        Self {
            width,
            height,
            bitmap: Arc::new(bitmap),
            adjust: Adjust::default(),
            encoded: Arc::default(),
        }
    }

    /// The same image with other adjustments (the bitmap is shared).
    #[must_use]
    pub fn with_adjust(&self, adjust: Adjust) -> Self {
        Self {
            adjust,
            ..self.clone()
        }
    }

    /// Checks the adjustments (sizes are checked with the geometry).
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidValue`] naming the first bad value.
    pub fn validate_adjust(&self) -> Result<(), ProjectError> {
        if self.adjust.is_valid() {
            return Ok(());
        }
        let a = self.adjust;
        let values = [a.brightness, a.contrast, a.gamma];
        let (name, lo, hi) = Adjust::RANGES
            .into_iter()
            .zip(values)
            .find(|((_, lo, hi), v)| !(v.is_finite() && (*lo..=*hi).contains(v)))
            .map_or(("adjustment", 0.0, 0.0), |(r, _)| r);
        Err(ProjectError::invalid(
            name,
            format!("must be between {lo} and {hi}"),
        ))
    }

    fn png_text(&self) -> String {
        self.encoded
            .get_or_init(|| base64_encode(&encode_png(&self.bitmap)))
            .clone()
    }
}

/// How an image is written in the project file.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ImageFile {
    width: f64,
    height: f64,
    pixels: [u32; 2],
    #[serde(default)]
    brightness: f64,
    #[serde(default)]
    contrast: f64,
    #[serde(default = "one")]
    gamma: f64,
    png: String,
}

fn one() -> f64 {
    1.0
}

impl From<Image> for ImageFile {
    fn from(image: Image) -> Self {
        Self {
            width: image.width,
            height: image.height,
            pixels: [image.bitmap.width(), image.bitmap.height()],
            brightness: image.adjust.brightness,
            contrast: image.adjust.contrast,
            gamma: image.adjust.gamma,
            png: image.png_text(),
        }
    }
}

impl TryFrom<ImageFile> for Image {
    type Error = String;

    fn try_from(file: ImageFile) -> Result<Self, String> {
        let bytes = base64_decode(&file.png).map_err(|e| format!("image data: {e}"))?;
        let decoded = decode(&bytes).map_err(|e| format!("image data: {e}"))?;
        let gray = decoded.gray;
        if [gray.width(), gray.height()] != file.pixels {
            return Err(format!(
                "image data is {} × {} pixels, the file says {} × {}",
                gray.width(),
                gray.height(),
                file.pixels[0],
                file.pixels[1]
            ));
        }
        let encoded = Arc::new(OnceLock::new());
        let _ = encoded.set(file.png);
        Ok(Self {
            width: file.width,
            height: file.height,
            bitmap: Arc::new(gray),
            adjust: Adjust {
                brightness: file.brightness,
                contrast: file.contrast,
                gamma: file.gamma,
            },
            encoded,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Image {
        Image::new(4.0, 2.0, Gray::new(2, 1, vec![0, 200]).unwrap())
    }

    #[test]
    fn round_trips_through_json_and_shares_pixels() {
        let img = image().with_adjust(Adjust {
            brightness: 10.0,
            contrast: -5.0,
            gamma: 1.5,
        });
        let json = serde_json::to_string(&img).unwrap();
        assert!(
            json.contains("\"pixels\":[2,1]") && json.contains("\"png\":\"iVBOR"),
            "{json}"
        );
        let back: Image = serde_json::from_str(&json).unwrap();
        assert_eq!(back, img);
        assert_eq!(serde_json::to_string(&back).unwrap(), json, "stable text");
        assert!(Arc::ptr_eq(&img.bitmap, &img.clone().bitmap));
    }

    #[test]
    fn rejects_bad_data_and_adjustments() {
        let json = serde_json::to_string(&image()).unwrap();
        let wrong = json.replace("[2,1]", "[3,1]");
        assert!(serde_json::from_str::<Image>(&wrong).is_err());
        let garbage = json.replace("\"png\":\"iVBOR", "\"png\":\"!!!!");
        assert!(serde_json::from_str::<Image>(&garbage).is_err());
        let bad = image().with_adjust(Adjust {
            gamma: 20.0,
            ..Adjust::default()
        });
        let err = bad.validate_adjust().unwrap_err().to_string();
        assert!(err.contains("gamma") && err.contains("0.1 and 10"), "{err}");
        assert!(image().validate_adjust().is_ok());
    }
}

#[cfg(test)]
mod shape_tests {
    use lasercraft_geom::{Affine, Point, Rect};

    use super::*;
    use crate::{Geometry, Project};

    #[test]
    fn image_shapes_save_load_and_select_by_their_rectangle() {
        let mut p = Project::default();
        let img = Image::new(10.0, 5.0, Gray::new(2, 2, vec![0, 50, 100, 255]).unwrap());
        let id = p
            .add_shape_on(Geometry::Image(img), Affine::translate(20.0, 30.0), 3)
            .unwrap();
        let back = Project::from_json(&p.to_json()).unwrap();
        assert_eq!(back, p);
        let shape = back.shape(id).unwrap();
        assert_eq!(shape.geometry().kind(), "image");
        assert!(shape.outlines(0.01).is_empty(), "no vector outline");
        assert!(shape.hit(Point::new(25.0, 32.0), 0.1), "inside the picture");
        assert!(!shape.hit(Point::new(35.0, 32.0), 0.1));
        assert_eq!(
            shape.bounds(),
            Rect::from_corners(Point::new(20.0, 30.0), Point::new(30.0, 35.0))
        );
        assert!(shape.touches(
            &Rect::from_corners(Point::new(29.0, 34.0), Point::new(40.0, 40.0)),
            0.01
        ));
        let bad = Image::new(0.0, 5.0, Gray::new(1, 1, vec![0]).unwrap());
        assert!(
            p.add_shape_on(Geometry::Image(bad), Affine::IDENTITY, 0)
                .is_err()
        );
    }
}
