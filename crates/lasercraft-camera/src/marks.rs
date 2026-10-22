//! The alignment pattern (own design, `docs/spec/camera.md`, "Bed
//! alignment"): four filled rings with a dot at their center on the
//! corners of a rectangle centered on the bed, plus a dot at the
//! rectangle's center; and finding the rings in a picture.

use lasercraft_geom::Point;
use serde::{Deserialize, Serialize};

use crate::CameraError;
use crate::dots::{Blob, blobs, dark_mask};
use crate::picture::Picture;

/// Pattern size and mark size, mm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    /// Distance between the left and right mark centers.
    pub width: f64,
    /// Distance between the top and bottom mark centers.
    pub height: f64,
    /// Outer ring diameter.
    pub mark: f64,
    /// Offset of the pattern's center from the bed's center, x.
    #[serde(default)]
    pub dx: f64,
    /// Offset of the pattern's center from the bed's center, y.
    #[serde(default)]
    pub dy: f64,
}

/// Inner ring diameter over the outer one.
pub const RING_INNER: f64 = 0.6;
/// Center dot diameter over the outer ring diameter.
pub const DOT: f64 = 0.25;

impl Pattern {
    /// 70 % of the bed, 12 mm marks.
    #[must_use]
    pub fn default_for(bed_width: f64, bed_height: f64) -> Self {
        Self {
            width: (bed_width * 0.7).round(),
            height: (bed_height * 0.7).round(),
            mark: 12.0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    /// For a camera on the head: 30 × 20 mm, 6 mm marks, centered; 36 ×
    /// 26 mm in all, inside a small view.
    #[must_use]
    pub fn for_head() -> Self {
        Self {
            width: 30.0,
            height: 20.0,
            mark: 6.0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    /// The pattern's center on a bed, document mm (the center dot).
    #[must_use]
    pub fn center(&self, bed_width: f64, bed_height: f64) -> Point {
        Point::new(bed_width / 2.0 + self.dx, bed_height / 2.0 + self.dy)
    }

    /// The pattern moved by `(dx, dy)` more, kept inside the bed with its
    /// marks.
    #[must_use]
    pub fn moved(&self, dx: f64, dy: f64, bed_width: f64, bed_height: f64) -> Self {
        let room = |bed: f64, size: f64| ((bed - size - self.mark) / 2.0).max(0.0);
        let (rx, ry) = (room(bed_width, self.width), room(bed_height, self.height));
        Self {
            dx: (self.dx + dx).clamp(-rx, rx),
            dy: (self.dy + dy).clamp(-ry, ry),
            ..*self
        }
    }

    /// The mark centers on a bed, document mm: top left, top right,
    /// bottom right, bottom left.
    #[must_use]
    pub fn centers(&self, bed_width: f64, bed_height: f64) -> [Point; 4] {
        let Point { x: cx, y: cy } = self.center(bed_width, bed_height);
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        [
            Point::new(cx - hw, cy - hh),
            Point::new(cx + hw, cy - hh),
            Point::new(cx + hw, cy + hh),
            Point::new(cx - hw, cy + hh),
        ]
    }

    /// Checks that the pattern fits the bed with room for the marks.
    ///
    /// # Errors
    ///
    /// Sizes out of range or a pattern larger than the bed.
    pub fn validate(&self, bed_width: f64, bed_height: f64) -> Result<(), CameraError> {
        if !(4.0..=50.0).contains(&self.mark) {
            return Err(CameraError::new("the mark size must be 4–50 mm"));
        }
        if !(self.width >= 3.0 * self.mark && self.height >= 3.0 * self.mark) {
            return Err(CameraError::new(
                "the pattern must be at least three marks wide and high",
            ));
        }
        if self.width + self.mark > bed_width || self.height + self.mark > bed_height {
            return Err(CameraError::new("the pattern is larger than the bed"));
        }
        let fits = |offset: f64, bed: f64, size: f64| {
            offset.is_finite() && offset.abs() <= (bed - size - self.mark) / 2.0 + 1e-9
        };
        if !fits(self.dx, bed_width, self.width) || !fits(self.dy, bed_height, self.height) {
            return Err(CameraError::new("the pattern reaches past the bed's edge"));
        }
        Ok(())
    }
}

/// Region indices inside `blob`'s box that are not part of it and do not
/// reach the box's edge: its holes.
fn holes(blob: &Blob, labels: &[u32], width: u32) -> Vec<Vec<(u32, u32)>> {
    let [x0, y0, x1, y1] = blob.bbox;
    let (bw, bh) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
    let inside = |x: usize, y: usize| {
        labels[(y0 as usize + y) * width as usize + x0 as usize + x] != blob.label
    };
    let mut seen = vec![false; bw * bh];
    let mut out = Vec::new();
    let mut stack = Vec::new();
    for start in 0..bw * bh {
        let (sx, sy) = (start % bw, start / bw);
        if seen[start] || !inside(sx, sy) {
            continue;
        }
        let mut region = Vec::new();
        let mut edge = false;
        seen[start] = true;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % bw, i / bw);
            region.push((x0 + x as u32, y0 + y as u32));
            edge |= x == 0 || y == 0 || x + 1 == bw || y + 1 == bh;
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx < bw && ny < bh && !seen[ny * bw + nx] && inside(nx, ny) {
                    seen[ny * bw + nx] = true;
                    stack.push(ny * bw + nx);
                }
            }
        }
        if !edge {
            out.push(region);
        }
    }
    out
}

/// Finds the four rings in a picture: raw pixel centers, clockwise from
/// the one nearest the picture's top-left corner.
///
/// # Errors
///
/// Fewer than four rings with a dot inside.
pub fn find_marks(picture: &Picture) -> Result<[Point; 4], CameraError> {
    let gray = picture.gray();
    let (w, h) = (picture.width(), picture.height());
    let mask = dark_mask(&gray, w, h);
    let (all, labels) = blobs(&mask, &gray, w, h);
    let mut rings: Vec<&Blob> = all
        .iter()
        .filter(|b| {
            b.area >= 20.0
                && b.area <= f64::from(w) * f64::from(h) / 50.0
                && b.bbox[2] - b.bbox[0] >= 6
                && b.bbox[3] - b.bbox[1] >= 6
                && b.bbox[0] > 0
                && b.bbox[1] > 0
                && b.bbox[2] + 1 < w
                && b.bbox[3] + 1 < h
        })
        .filter(|b| b.fill() < 0.9 && b.axes.1 > 0.5 * b.axes.0)
        .filter(|b| {
            let inner = holes(b, &labels, w);
            inner.len() == 1
                && all.iter().any(|d| {
                    d.label != b.label
                        && d.area >= 2.0
                        && inner[0].contains(&(d.center.x as u32, d.center.y as u32))
                })
        })
        .collect();
    if rings.len() < 4 {
        return Err(CameraError::new(format!(
            "{} of the 4 alignment marks were found; check the lighting, or click the marks instead",
            rings.len()
        )));
    }
    // The four largest of similar size.
    rings.sort_by(|a, b| b.area.total_cmp(&a.area));
    rings.truncate(4);
    if rings[3].area < 0.3 * rings[0].area {
        return Err(CameraError::new(
            "the marks found differ too much in size; click the marks instead",
        ));
    }
    let n = 4.0;
    let c = Point::new(
        rings.iter().map(|b| b.center.x).sum::<f64>() / n,
        rings.iter().map(|b| b.center.y).sum::<f64>() / n,
    );
    let mut ordered: Vec<Point> = rings.iter().map(|b| b.center).collect();
    // Clockwise on screen (y down) is increasing angle.
    ordered.sort_by(|a, b| {
        (a.y - c.y)
            .atan2(a.x - c.x)
            .total_cmp(&(b.y - c.y).atan2(b.x - c.x))
    });
    let first = (0..4)
        .min_by(|&i, &j| (ordered[i].x + ordered[i].y).total_cmp(&(ordered[j].x + ordered[j].y)))
        .unwrap_or(0);
    ordered.rotate_left(first);
    Ok([ordered[0], ordered[1], ordered[2], ordered[3]])
}

/// Whether document point `p` is burned by the pattern on a bed: for the
/// simulated camera and tests.
#[must_use]
pub fn burned(pattern: &Pattern, bed_width: f64, bed_height: f64, p: Point) -> bool {
    let r = pattern.mark / 2.0;
    let centers = pattern.centers(bed_width, bed_height);
    let center = pattern.center(bed_width, bed_height);
    centers.iter().any(|c| {
        let d = p.distance(*c);
        (d <= r && d >= r * RING_INNER) || d <= r * DOT
    }) || p.distance(center) <= r * DOT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::{Lens, Model};
    use crate::synth::{Pose, render};

    #[test]
    fn the_four_rings_are_found_in_order() {
        let (bw, bh) = (300.0, 200.0);
        let pattern = Pattern::default_for(bw, bh);
        pattern.validate(bw, bh).unwrap();
        let mut lens = Lens::ideal(Model::Standard, 640, 480, 600.0);
        lens.k = vec![-0.15, 0.02];
        let pose = Pose::overhead(150.0, 100.0, 380.0, [0.03, 0.02, 0.04]);
        let pic = render(&lens, &pose, [90, 90, 90], |p| {
            (p.x >= 0.0 && p.y >= 0.0 && p.x <= bw && p.y <= bh).then(|| {
                if burned(&pattern, bw, bh, p) {
                    [40, 30, 20]
                } else {
                    [215, 190, 150]
                }
            })
        });
        let found = find_marks(&pic).unwrap();
        for (px, bed) in found.iter().zip(pattern.centers(bw, bh)) {
            let truth = lens.project(pose.ray(bed).unwrap());
            assert!(px.distance(truth) < 0.5, "{px:?} {truth:?}");
        }
        assert!(find_marks(&Picture::filled(64, 48, [200, 200, 200])).is_err());
        assert!(
            Pattern {
                mark: 2.0,
                ..pattern
            }
            .validate(bw, bh)
            .is_err()
        );
        assert!(
            Pattern {
                width: 400.0,
                ..pattern
            }
            .validate(bw, bh)
            .is_err()
        );
    }

    #[test]
    fn a_moved_pattern_stays_on_the_bed() {
        let (bw, bh) = (300.0, 200.0);
        let pattern = Pattern::default_for(bw, bh);
        let moved = pattern.moved(10.0, -5.0, bw, bh);
        assert_eq!(moved.centers(bw, bh)[0], Point::new(55.0, 25.0));
        assert_eq!(moved.center(bw, bh), Point::new(160.0, 95.0));
        assert!(burned(&moved, bw, bh, Point::new(160.0, 95.0)));
        assert!(!burned(&moved, bw, bh, Point::new(150.0, 100.0)));
        // 210 + 12 wide on 300: 39 mm of room each side.
        let far = pattern.moved(500.0, -500.0, bw, bh);
        assert_eq!((far.dx, far.dy), (39.0, -24.0));
        far.validate(bw, bh).unwrap();
        assert!(Pattern { dx: 40.0, ..far }.validate(bw, bh).is_err());
        let json: Pattern =
            serde_json::from_str(r#"{"width": 210, "height": 140, "mark": 12}"#).unwrap();
        assert_eq!(json, pattern, "dx and dy are optional");
    }
}
