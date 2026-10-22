//! An image on the dot grid (`docs/spec/images.md`, "From bitmap to dots").

use crate::{Adjust, Gray, Method, RasterError, dither, fail};

/// Most dots in one image's grid.
pub const MAX_CELLS: u64 = 60_000_000;

/// An image's dots: power fractions on the document grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster {
    /// Grid index of the first column: its dots are centered on
    /// x = (col0 + i + 0.5) × interval.
    pub col0: i64,
    /// Grid index of the first row (rows grow with document y).
    pub row0: i64,
    /// Dot spacing, mm.
    pub interval: f64,
    /// Columns.
    pub cols: usize,
    /// Rows.
    pub rows: usize,
    /// Power fraction 0–1 per dot, row-major; 0 is off.
    pub values: Vec<f64>,
}

impl Raster {
    /// The value at (`col`, `row`).
    #[must_use]
    pub fn get(&self, col: usize, row: usize) -> f64 {
        self.values[row * self.cols + col]
    }

    /// Number of dots that burn.
    #[must_use]
    pub fn lit(&self) -> usize {
        self.values.iter().filter(|v| **v > 0.0).count()
    }
}

/// Inverse of the SVG matrix `[a, b, c, d, e, f]`.
fn invert(m: [f64; 6]) -> Option<[f64; 6]> {
    let [a, b, c, d, e, f] = m;
    let det = a * d - b * c;
    if det.abs() < 1e-12 || !det.is_finite() {
        return None;
    }
    let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
    Some([ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)])
}

fn apply(m: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

/// Bilinear sample of `g` at pixel coordinates (x, y), pixel centers at
/// half-integers; edges clamp.
fn bilinear(g: &Gray, x: f64, y: f64) -> f64 {
    let max_x = f64::from(g.width() - 1);
    let max_y = f64::from(g.height() - 1);
    let fx = (x - 0.5).clamp(0.0, max_x);
    let fy = (y - 0.5).clamp(0.0, max_y);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    // Clamped to the bitmap, so the casts stay in range.
    let (x0, y0) = (x0 as u32, y0 as u32);
    let (x1, y1) = ((x0 + 1).min(g.width() - 1), (y0 + 1).min(g.height() - 1));
    let p = |x, y| f64::from(g.get(x, y));
    let top = p(x0, y0) * (1.0 - tx) + p(x1, y0) * tx;
    let bottom = p(x0, y1) * (1.0 - tx) + p(x1, y1) * tx;
    (top * (1.0 - ty) + bottom * ty) / 255.0
}

/// Places `gray` (spanning `size` mm locally, mapped to the document by
/// `matrix`) on the grid of `interval` mm and turns it into dot powers.
///
/// # Errors
///
/// A bad interval, size or transform, or a grid over [`MAX_CELLS`].
#[allow(clippy::too_many_arguments)]
pub fn rasterize(
    gray: &Gray,
    adjust: &Adjust,
    negative: bool,
    size: [f64; 2],
    matrix: [f64; 6],
    interval: f64,
    method: Method,
) -> Result<Raster, RasterError> {
    if !(interval.is_finite() && interval > 0.0) {
        return Err(fail("the line interval must be positive"));
    }
    let [w, h] = size;
    let inverse = invert(matrix).ok_or_else(|| fail("the image is flattened to a line"))?;
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(x, y)| apply(matrix, x, y));
    let min_x = corners.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
    let max_x = corners
        .iter()
        .map(|c| c.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = corners.iter().map(|c| c.1).fold(f64::INFINITY, f64::min);
    let max_y = corners
        .iter()
        .map(|c| c.1)
        .fold(f64::NEG_INFINITY, f64::max);
    // Dot centers strictly inside the box's span of grid cells.
    let col0 = (min_x / interval).floor() as i64;
    let row0 = (min_y / interval).floor() as i64;
    let cols = ((max_x / interval).ceil() as i64 - col0).max(1) as usize;
    let rows = ((max_y / interval).ceil() as i64 - row0).max(1) as usize;
    if cols as u64 * rows as u64 > MAX_CELLS {
        return Err(fail(format!(
            "the image needs {cols} × {rows} dots at {interval} mm; at most {MAX_CELLS} (raise the line interval)"
        )));
    }
    // Shrink so there is about one bitmap pixel per dot.
    let det = (matrix[0] * matrix[3] - matrix[1] * matrix[2]).abs().sqrt();
    let px_per_mm = (f64::from(gray.width()) / w).min(f64::from(gray.height()) / h);
    let k = (px_per_mm * interval / det).floor().max(1.0) as u32;
    let small = gray.shrink(k);
    let sx = f64::from(small.width()) / w;
    let sy = f64::from(small.height()) / h;

    let table: Vec<f64> = (0..=255)
        .map(|i| adjust.apply(f64::from(i) / 255.0))
        .collect();
    let mut values = vec![0.0; cols * rows];
    let mut mask = vec![false; cols * rows];
    for r in 0..rows {
        let y = (row0 + r as i64) as f64 * interval + interval / 2.0;
        for c in 0..cols {
            let x = (col0 + c as i64) as f64 * interval + interval / 2.0;
            let (lx, ly) = apply(inverse, x, y);
            if !(0.0..w).contains(&lx) || !(0.0..h).contains(&ly) {
                continue;
            }
            let level = bilinear(&small, lx * sx, ly * sy);
            // Interpolate the adjustment table between bytes.
            let pos = level * 255.0;
            let i = (pos.floor() as usize).min(254);
            let t = pos - i as f64;
            let adjusted = table[i] * (1.0 - t) + table[i + 1] * t;
            let i = r * cols + c;
            values[i] = if negative { adjusted } else { 1.0 - adjusted };
            mask[i] = true;
        }
    }
    dither(&mut values, &mask, cols, (col0, row0), method);
    Ok(Raster {
        col0,
        row0,
        interval,
        cols,
        rows,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

    fn ramp() -> Gray {
        // 4 × 1: black, dark, light, white.
        Gray::new(4, 1, vec![0, 85, 170, 255]).unwrap()
    }

    #[test]
    fn grid_is_aligned_to_the_document_and_covers_the_box() {
        let g = Gray::new(1, 1, vec![0]).unwrap();
        let moved = [1.0, 0.0, 0.0, 1.0, 1.03, 2.0];
        let r = rasterize(
            &g,
            &Adjust::default(),
            false,
            [1.0, 0.5],
            moved,
            0.1,
            Method::Threshold(0.5),
        )
        .unwrap();
        assert_eq!((r.col0, r.row0, r.cols, r.rows), (10, 20, 11, 5));
        assert_eq!(r.get(0, 0), 1.0, "center 1.05 is inside");
        assert_eq!(r.get(10, 0), 0.0, "center 2.05 is past the right edge");
        assert_eq!(r.lit(), 50);
    }

    #[test]
    fn grayscale_follows_levels_and_negative_inverts() {
        let r = rasterize(
            &ramp(),
            &Adjust::default(),
            false,
            [4.0, 1.0],
            ID,
            1.0,
            Method::Grayscale,
        )
        .unwrap();
        let v: Vec<f64> = (0..4).map(|c| (r.get(c, 0) * 255.0).round()).collect();
        assert_eq!(v, [255.0, 170.0, 85.0, 0.0]);
        let n = rasterize(
            &ramp(),
            &Adjust::default(),
            true,
            [4.0, 1.0],
            ID,
            1.0,
            Method::Grayscale,
        )
        .unwrap();
        assert_eq!((n.get(0, 0), n.get(3, 0)), (0.0, 1.0));
        let bright = Adjust {
            brightness: 100.0,
            ..Adjust::default()
        };
        let b = rasterize(
            &ramp(),
            &bright,
            false,
            [4.0, 1.0],
            ID,
            1.0,
            Method::Grayscale,
        )
        .unwrap();
        assert_eq!(b.lit(), 0, "everything white");
    }

    #[test]
    fn rotation_and_mirroring_follow_the_transform() {
        // Rotated 90° clockwise (y down): the black left column ends on top.
        let rot = [0.0, 1.0, -1.0, 0.0, 1.0, 0.0];
        let r = rasterize(
            &ramp(),
            &Adjust::default(),
            false,
            [4.0, 1.0],
            rot,
            1.0,
            Method::Threshold(0.5),
        )
        .unwrap();
        assert_eq!((r.cols, r.rows), (1, 4));
        assert_eq!((r.get(0, 0), r.get(0, 3)), (1.0, 0.0));
        let mirror = [-1.0, 0.0, 0.0, 1.0, 4.0, 0.0];
        let m = rasterize(
            &ramp(),
            &Adjust::default(),
            false,
            [4.0, 1.0],
            mirror,
            1.0,
            Method::Threshold(0.5),
        )
        .unwrap();
        assert_eq!((m.get(0, 0), m.get(3, 0)), (0.0, 1.0));
    }

    #[test]
    fn big_bitmaps_are_averaged_down_first() {
        // A fine checkerboard averages to mid-gray instead of aliasing.
        let px: Vec<u8> = (0..64 * 64)
            .map(|i| if (i % 64 + i / 64) % 2 == 0 { 0 } else { 255 })
            .collect();
        let g = Gray::new(64, 64, px).unwrap();
        let r = rasterize(
            &g,
            &Adjust::default(),
            false,
            [8.0, 8.0],
            ID,
            1.0,
            Method::Grayscale,
        )
        .unwrap();
        assert!(
            r.values.iter().all(|v| (v - 0.5).abs() < 0.01),
            "{:?}",
            &r.values[..4]
        );
    }

    #[test]
    fn refusals() {
        let g = ramp();
        let a = Adjust::default();
        assert!(rasterize(&g, &a, false, [4.0, 1.0], ID, 0.0, Method::Grayscale).is_err());
        assert!(rasterize(&g, &a, false, [4.0, 1.0], [0.0; 6], 1.0, Method::Grayscale).is_err());
        let err = rasterize(
            &g,
            &a,
            false,
            [4000.0, 4000.0],
            ID,
            0.0001,
            Method::Grayscale,
        )
        .unwrap_err();
        assert!(err.0.contains("raise the line interval"), "{err}");
    }
}
