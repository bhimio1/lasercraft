//! Scanning the bed with a camera on the head (`docs/spec/camera.md`,
//! "Cameras on the laser head"): where the head stops so the pictures
//! cover the bed.

use lasercraft_geom::{Point, Rect};

use crate::align::Alignment;
use crate::lens::Lens;

/// Share of a picture's footprint each step advances.
pub const STEP: f64 = 0.8;

/// Most stops in one scan.
pub const MAX_STOPS: usize = 400;

/// The bed area a head camera's picture covers, relative to the head
/// (document mm), or `None` without a head position or with a picture
/// corner at the horizon.
#[must_use]
pub fn footprint(alignment: &Alignment, lens: &Lens) -> Option<Rect> {
    let head = alignment.head?;
    let (w, h) = (f64::from(lens.width), f64::from(lens.height));
    let corners = [
        Point::new(0.0, 0.0),
        Point::new(w, 0.0),
        Point::new(w, h),
        Point::new(0.0, h),
    ]
    .map(|p| alignment.to_bed(lens, p).map(|b| b - head));
    let corners: Option<Vec<Point>> = corners.into_iter().collect();
    // The inner rectangle: every point of it is seen.
    let c = corners?;
    let (x0, x1) = (c[0].x.max(c[3].x), c[1].x.min(c[2].x));
    let (y0, y1) = (c[0].y.max(c[1].y), c[2].y.min(c[3].y));
    (x1 > x0 && y1 > y0).then(|| Rect::from_corners(Point::new(x0, y0), Point::new(x1, y1)))
}

/// Evenly spaced positions along one axis so the footprint `[f0, f1]`
/// (relative to the head) covers `[0, bed]`, each inside the bed.
fn axis(f0: f64, f1: f64, bed: f64) -> Vec<f64> {
    let (first, last) = (-f0, bed - f1);
    let step = (f1 - f0) * STEP;
    let n = if last > first && step > 0.0 {
        ((last - first) / step).ceil() as usize + 1
    } else {
        1
    };
    (0..n)
        .map(|i| {
            let t = if n > 1 {
                i as f64 / (n - 1) as f64
            } else {
                0.5
            };
            (first + (last - first) * t).clamp(0.0, bed)
        })
        .collect()
}

/// Head positions for a scan of the bed, row by row in alternating
/// directions; `None` when more than [`MAX_STOPS`] would be needed.
#[must_use]
pub fn stops(footprint: Rect, bed_width: f64, bed_height: f64) -> Option<Vec<Point>> {
    let xs = axis(footprint.min.x, footprint.max.x, bed_width);
    let ys = axis(footprint.min.y, footprint.max.y, bed_height);
    if xs.len() * ys.len() > MAX_STOPS {
        return None;
    }
    let mut out = Vec::with_capacity(xs.len() * ys.len());
    for (row, y) in ys.iter().enumerate() {
        let mut line: Vec<Point> = xs.iter().map(|x| Point::new(*x, *y)).collect();
        if row % 2 == 1 {
            line.reverse();
        }
        out.extend(line);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_cover_the_bed_with_overlap_and_stay_on_it() {
        // A 50 × 40 mm view, 15 mm right of the head.
        let fp = Rect::from_corners(Point::new(-10.0, -20.0), Point::new(40.0, 20.0));
        let s = stops(fp, 300.0, 200.0).unwrap();
        assert!(
            s.iter()
                .all(|p| (0.0..=300.0).contains(&p.x) && (0.0..=200.0).contains(&p.y))
        );
        // Every bed point is seen from some stop.
        for (x, y) in [(0.0, 0.0), (299.0, 199.0), (150.0, 100.0), (5.0, 190.0)] {
            assert!(
                s.iter().any(|h| {
                    let r = Rect::from_corners(*h + fp.min, *h + fp.max);
                    r.contains(Point::new(x, y))
                }),
                "{x}, {y}"
            );
        }
        // Rows alternate direction.
        assert!(s[0].x < s[1].x);
        let row = s.iter().filter(|p| p.y == s[0].y).count();
        assert!(s[row].x > s[row + 1].x);
        // A tiny view is refused rather than taking forever.
        let tiny = Rect::from_corners(Point::new(-1.0, -1.0), Point::new(1.0, 1.0));
        assert!(stops(tiny, 300.0, 200.0).is_none());
    }
}
