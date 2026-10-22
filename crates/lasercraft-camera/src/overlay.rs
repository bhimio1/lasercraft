//! The overlay (`docs/spec/camera.md`, "Overlay"): a picture laid on the
//! bed through the lens correction and the alignment.

use lasercraft_geom::{Point, Rect};
use serde::{Deserialize, Serialize};

use crate::align::Alignment;
use crate::lens::Lens;
use crate::picture::{Picture, encode_png};

/// Overlay grid step, mm.
pub const STEP_MM: f64 = 0.5;
/// Longest overlay side, pixels.
pub const MAX_SIDE: u32 = 2000;

/// The overlay's fine adjustment (`docs/spec/camera.md`, `fit`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fit {
    /// Width, % (scaled about the bed center).
    pub width: f64,
    /// Height, %.
    pub height: f64,
    /// Shift right, mm.
    pub dx: f64,
    /// Shift down, mm.
    pub dy: f64,
}

impl Default for Fit {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            dx: 0.0,
            dy: 0.0,
        }
    }
}

/// An RGBA picture covering the bed `(0, 0)–(width_mm, height_mm)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Overlay {
    /// Pixels across.
    pub width: u32,
    /// Pixels down.
    pub height: u32,
    /// Bed millimetres per pixel.
    pub mm_per_px: f64,
    /// Packed RGBA, rows top to bottom; alpha 0 where no camera sees.
    pub rgba: Vec<u8>,
}

impl Overlay {
    /// A transparent overlay for a bed.
    #[must_use]
    pub fn empty(bed_width: f64, bed_height: f64) -> Self {
        let longer = bed_width.max(bed_height);
        let mm_per_px = STEP_MM.max(longer / f64::from(MAX_SIDE));
        let width = ((bed_width / mm_per_px).ceil() as u32).max(1);
        let height = ((bed_height / mm_per_px).ceil() as u32).max(1);
        Self {
            width,
            height,
            mm_per_px,
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    /// Paints one camera's picture onto the overlay, within `region` (bed
    /// mm) or everywhere.
    pub fn paint(
        &mut self,
        picture: &Picture,
        lens: &Lens,
        alignment: &Alignment,
        fit: &Fit,
        region: Option<Rect>,
    ) {
        let Some(inverse) = alignment.matrix.inverse() else {
            return;
        };
        // Pictures of another size are sampled at the lens's scale.
        let sx = f64::from(picture.width()) / f64::from(lens.width);
        let sy = f64::from(picture.height()) / f64::from(lens.height);
        let bed_w = f64::from(self.width) * self.mm_per_px;
        let bed_h = f64::from(self.height) * self.mm_per_px;
        let (cx, cy) = (bed_w / 2.0, bed_h / 2.0);
        let (fw, fh) = (fit.width / 100.0, fit.height / 100.0);
        let (x0, y0, x1, y1) = region.map_or((0, 0, self.width, self.height), |r| {
            let px = |v: f64| (v / self.mm_per_px).floor().max(0.0) as u32;
            (
                px(r.min.x),
                px(r.min.y),
                (px(r.max.x) + 1).min(self.width),
                (px(r.max.y) + 1).min(self.height),
            )
        });
        for y in y0..y1 {
            for x in x0..x1 {
                let shown = Point::new(
                    (f64::from(x) + 0.5) * self.mm_per_px,
                    (f64::from(y) + 0.5) * self.mm_per_px,
                );
                // Undo the fit: the bed point whose picture is shown here.
                let bed = Point::new(
                    cx + (shown.x - fit.dx - cx) / fw,
                    cy + (shown.y - fit.dy - cy) / fh,
                );
                let Some(corrected) = inverse.apply(bed) else {
                    continue;
                };
                let raw = lens.uncorrect(corrected);
                let Some(c) = picture.sample(raw.x * sx, raw.y * sy) else {
                    continue;
                };
                let i = (y as usize * self.width as usize + x as usize) * 4;
                self.rgba[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
    }

    /// The color at a bed point, `None` where transparent.
    #[must_use]
    pub fn at(&self, p: Point) -> Option<[u8; 3]> {
        let (x, y) = (
            (p.x / self.mm_per_px).floor(),
            (p.y / self.mm_per_px).floor(),
        );
        if x < 0.0 || y < 0.0 || x >= f64::from(self.width) || y >= f64::from(self.height) {
            return None;
        }
        let i = (y as usize * self.width as usize + x as usize) * 4;
        (self.rgba[i + 3] > 0).then(|| [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]])
    }

    /// An RGBA PNG of the overlay.
    #[must_use]
    pub fn encode_png(&self) -> Vec<u8> {
        encode_png(self.width, self.height, png::ColorType::Rgba, &self.rgba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::Model;
    use crate::marks::{Pattern, burned, find_marks};
    use crate::synth::{Pose, render};

    #[test]
    fn a_picture_lands_where_the_bed_is() {
        let (bw, bh) = (300.0, 200.0);
        let mut lens = Lens::ideal(Model::Standard, 640, 480, 600.0);
        lens.k = vec![-0.15, 0.02];
        let pose = Pose::overhead(150.0, 100.0, 380.0, [0.04, -0.03, 0.06]);
        let pattern = Pattern::default_for(bw, bh);
        let scene = |p: Point| {
            (p.x >= 0.0 && p.y >= 0.0 && p.x <= bw && p.y <= bh).then(|| {
                if burned(&pattern, bw, bh, p) {
                    [30, 20, 10]
                } else if p.x < 150.0 {
                    [220, 200, 160]
                } else {
                    [160, 200, 220]
                }
            })
        };
        let picture = render(&lens, &pose, [60, 60, 60], scene);
        let raw = find_marks(&picture).unwrap();
        let matrix = crate::align::fit(&lens, &raw, &pattern.centers(bw, bh)).unwrap();
        let alignment = Alignment {
            name: "bed".into(),
            height: 0.0,
            matrix,
            lens: String::new(),
            width: 640,
            height_px: 480,
            created: String::new(),
            head: None,
        };
        let mut overlay = Overlay::empty(bw, bh);
        overlay.paint(&picture, &lens, &alignment, &Fit::default(), None);
        // The color edge at x = 150 mm is within a millimetre.
        let left = overlay.at(Point::new(148.5, 60.0)).unwrap();
        let right = overlay.at(Point::new(151.5, 60.0)).unwrap();
        assert!(left[0] > 200 && right[2] > 200, "{left:?} {right:?}");
        // The pattern's center dot is where it was burned.
        let center = overlay.at(Point::new(150.0, 100.0)).unwrap();
        assert!(center[0] < 80, "{center:?}");
        // A fit shift moves what is shown.
        let mut shifted = Overlay::empty(bw, bh);
        let fit = Fit {
            dx: 10.0,
            ..Fit::default()
        };
        shifted.paint(&picture, &lens, &alignment, &fit, None);
        assert!(shifted.at(Point::new(160.0, 100.0)).unwrap()[0] < 80);
        assert!(!overlay.encode_png().is_empty());
    }
}
