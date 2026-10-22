//! Finding the calibration sheet's dots in a picture
//! (`docs/spec/camera.md`, "Calibration", dot finding): a local threshold,
//! dark round regions, the three marker dots, then the grid grown outwards
//! from them.

use std::collections::BTreeMap;

use lasercraft_geom::Point;

use crate::CameraError;
use crate::homography::Homography;
use crate::picture::Picture;
use crate::sheet;

/// Found dots: grid position and pixel center.
pub type Dots = Vec<((i32, i32), Point)>;

/// Fewest dots a usable picture shows.
pub const MIN_DOTS: usize = 20;

/// A dark connected region.
#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    /// Pixel count.
    pub area: f64,
    /// Darkness-weighted center, pixels.
    pub center: Point,
    /// Bounding box: left, top, right, bottom (inclusive).
    pub bbox: [u32; 4],
    /// Major and minor axis lengths of the equivalent ellipse, pixels.
    pub axes: (f64, f64),
    /// Label in the label image.
    pub label: u32,
}

impl Blob {
    /// Area over the equivalent ellipse's area: about 1 for a filled
    /// ellipse, small for a ring or a line.
    #[must_use]
    pub fn fill(&self) -> f64 {
        let (a, b) = self.axes;
        if a * b <= 0.0 {
            return 0.0;
        }
        self.area / (std::f64::consts::PI * a / 2.0 * b / 2.0)
    }
}

/// Dark pixels: darker than their surroundings' mean by a margin.
#[must_use]
pub fn dark_mask(gray: &[u8], width: u32, height: u32) -> Vec<bool> {
    let (w, h) = (width as usize, height as usize);
    let win = (w.min(h) / 16).max(15) | 1;
    let r = win / 2;
    let mut integral = vec![0u64; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0u64;
        for x in 0..w {
            row += u64::from(gray[y * w + x]);
            integral[(y + 1) * (w + 1) + x + 1] = integral[y * (w + 1) + x + 1] + row;
        }
    }
    let mut out = vec![false; w * h];
    for y in 0..h {
        let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(h));
        for x in 0..w {
            let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(w));
            let sum = integral[y1 * (w + 1) + x1] + integral[y0 * (w + 1) + x0]
                - integral[y0 * (w + 1) + x1]
                - integral[y1 * (w + 1) + x0];
            let n = ((x1 - x0) * (y1 - y0)) as u64;
            let mean = sum as f64 / n as f64;
            out[y * w + x] = f64::from(gray[y * w + x]) + 12.0 < mean;
        }
    }
    out
}

/// Connected regions of `mask` (4-neighbours) and the label image (0 for
/// unlabeled pixels, region `i` labeled `i + 1`).
#[must_use]
pub fn blobs(mask: &[bool], gray: &[u8], width: u32, height: u32) -> (Vec<Blob>, Vec<u32>) {
    let (w, h) = (width as usize, height as usize);
    let mut labels = vec![0u32; w * h];
    let mut out = Vec::new();
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !mask[start] || labels[start] != 0 {
            continue;
        }
        let label = out.len() as u32 + 1;
        labels[start] = label;
        stack.push(start);
        let (mut n, mut sw, mut sx, mut sy) = (0.0, 0.0, 0.0, 0.0);
        let (mut mx, mut my, mut mxx, mut myy, mut mxy) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut bbox = [u32::MAX, u32::MAX, 0, 0];
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let weight = 255.0 - f64::from(gray[i]) + 1.0;
            n += 1.0;
            sw += weight;
            sx += weight * fx;
            sy += weight * fy;
            mx += fx;
            my += fy;
            mxx += fx * fx;
            myy += fy * fy;
            mxy += fx * fy;
            bbox = [
                bbox[0].min(x as u32),
                bbox[1].min(y as u32),
                bbox[2].max(x as u32),
                bbox[3].max(y as u32),
            ];
            let mut visit = |j: usize| {
                if mask[j] && labels[j] == 0 {
                    labels[j] = label;
                    stack.push(j);
                }
            };
            if x > 0 {
                visit(i - 1);
            }
            if x + 1 < w {
                visit(i + 1);
            }
            if y > 0 {
                visit(i - w);
            }
            if y + 1 < h {
                visit(i + w);
            }
        }
        let (cx, cy) = (mx / n, my / n);
        let (vxx, vyy, vxy) = (mxx / n - cx * cx, myy / n - cy * cy, mxy / n - cx * cy);
        let tr = vxx + vyy;
        let det = vxx * vyy - vxy * vxy;
        let disc = (tr * tr / 4.0 - det).max(0.0).sqrt();
        let (l1, l2) = (tr / 2.0 + disc, (tr / 2.0 - disc).max(0.0));
        out.push(Blob {
            area: n,
            center: Point::new(sx / sw, sy / sw),
            bbox,
            // A uniform ellipse with semi-axes a has variance a² / 4.
            axes: (4.0 * l1.sqrt(), 4.0 * l2.sqrt()),
            label,
        });
    }
    (out, labels)
}

/// The dark round regions of a picture: candidate dots.
#[must_use]
pub fn round_blobs(picture: &Picture) -> Vec<Blob> {
    let gray = picture.gray();
    let (w, h) = (picture.width(), picture.height());
    let mask = dark_mask(&gray, w, h);
    let (all, _) = blobs(&mask, &gray, w, h);
    let max_area = f64::from(w) * f64::from(h) / 40.0;
    all.into_iter()
        .filter(|b| {
            let (a, b2) = b.axes;
            b.area >= 8.0 && b.area <= max_area && b2 > 0.3 * a && (0.6..=1.4).contains(&b.fill())
        })
        .collect()
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// Indices of the `k` blobs nearest to blob `i`.
fn nearest(blobs: &[Blob], i: usize, k: usize) -> Vec<usize> {
    let mut d: Vec<(f64, usize)> = blobs
        .iter()
        .enumerate()
        .filter(|(j, _)| *j != i)
        .map(|(j, b)| (b.center.distance(blobs[i].center), j))
        .collect();
    d.sort_by(|a, b| a.0.total_cmp(&b.0));
    d.into_iter().take(k).map(|(_, j)| j).collect()
}

/// The three marker blobs: center, x direction, y direction.
fn markers(blobs: &[Blob]) -> Option<[usize; 3]> {
    let big: Vec<usize> = (0..blobs.len())
        .filter(|&i| {
            let around = median(
                nearest(blobs, i, 6)
                    .iter()
                    .map(|&j| blobs[j].area)
                    .collect(),
            );
            around > 0.0 && blobs[i].area > 1.6 * around
        })
        .collect();
    let mut best: Option<(f64, [usize; 3])> = None;
    for &o in &big {
        let spacing = median(
            nearest(blobs, o, 4)
                .iter()
                .map(|&j| blobs[j].center.distance(blobs[o].center))
                .collect(),
        );
        for &a in &big {
            for &b in &big {
                if a == o || b == o || a == b {
                    continue;
                }
                let (pa, pb) = (
                    blobs[a].center - blobs[o].center,
                    blobs[b].center - blobs[o].center,
                );
                let (la, lb) = (pa.length(), pb.length());
                let cross = pa.x * pb.y - pa.y * pb.x;
                if cross <= 0.0 || la < 1e-9 || lb < 1e-9 {
                    continue;
                }
                let cos = (pa.x * pb.x + pa.y * pb.y) / (la * lb);
                let ratio = la / lb;
                let near = la / spacing.max(1e-9);
                if cos.abs() > 0.4 || !(0.6..1.67).contains(&ratio) || !(0.6..1.8).contains(&near) {
                    continue;
                }
                let cost = cos.abs() + (ratio.ln()).abs() + (near.ln()).abs();
                if best.is_none_or(|(c, _)| cost < c) {
                    best = Some((cost, [o, a, b]));
                }
            }
        }
    }
    best.map(|(_, m)| m)
}

/// Fits the grid-to-pixel map from known dots near `(i, j)`.
fn local_map(known: &BTreeMap<(i32, i32), Point>, i: i32, j: i32) -> Option<Homography> {
    for reach in 2..=4 {
        let near: Vec<(Point, Point)> = known
            .iter()
            .filter(|((a, b), _)| (a - i).abs() <= reach && (b - j).abs() <= reach)
            .map(|((a, b), p)| (Point::new(f64::from(*a), f64::from(*b)), *p))
            .collect();
        if near.len() >= 4 {
            let (from, to): (Vec<Point>, Vec<Point>) = near.into_iter().unzip();
            let collinear_x = from.iter().all(|p| (p.x - from[0].x).abs() < 1e-9);
            let collinear_y = from.iter().all(|p| (p.y - from[0].y).abs() < 1e-9);
            if !collinear_x && !collinear_y {
                return Homography::fit(&from, &to);
            }
        }
    }
    None
}

/// The sheet's dots in `picture`: grid position and pixel center.
///
/// # Errors
///
/// The three marker dots are not found, or fewer than [`MIN_DOTS`] dots
/// are.
pub fn find_sheet(picture: &Picture) -> Result<Dots, CameraError> {
    let blobs = round_blobs(picture);
    let [o, x, y] = markers(&blobs).ok_or_else(|| {
        CameraError::new("the calibration sheet's three large dots are not in the picture")
    })?;
    let [(oi, oj), (xi, xj), (yi, yj)] = sheet::MARKERS;
    let mut known: BTreeMap<(i32, i32), Point> = BTreeMap::new();
    let mut used = vec![false; blobs.len()];
    for (pos, idx) in [((oi, oj), o), ((xi, xj), x), ((yi, yj), y)] {
        known.insert(pos, blobs[idx].center);
        used[idx] = true;
    }
    // The fourth corner of the first square, from the parallelogram rule.
    let guess = blobs[x].center + (blobs[y].center - blobs[o].center);
    if let Some(k) = closest(
        &blobs,
        &used,
        guess,
        0.4 * blobs[x].center.distance(blobs[o].center),
    ) {
        known.insert((xi, yj), blobs[k].center);
        used[k] = true;
    }
    loop {
        let mut added = false;
        for j in 0..sheet::ROWS {
            for i in 0..sheet::COLUMNS {
                if known.contains_key(&(i, j)) {
                    continue;
                }
                let touches = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(di, dj)| known.contains_key(&(i + di, j + dj)));
                if !touches {
                    continue;
                }
                let Some(map) = local_map(&known, i, j) else {
                    continue;
                };
                let (Some(p), Some(q)) = (
                    map.apply(Point::new(f64::from(i), f64::from(j))),
                    map.apply(Point::new(f64::from(i) + 1.0, f64::from(j))),
                ) else {
                    continue;
                };
                let pitch = p.distance(q);
                if let Some(k) = closest(&blobs, &used, p, 0.35 * pitch) {
                    known.insert((i, j), blobs[k].center);
                    used[k] = true;
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }
    if known.len() < MIN_DOTS {
        return Err(CameraError::new(format!(
            "only {} of the sheet's dots were found; at least {MIN_DOTS} are needed",
            known.len()
        )));
    }
    Ok(known.into_iter().collect())
}

fn closest(blobs: &[Blob], used: &[bool], p: Point, within: f64) -> Option<usize> {
    blobs
        .iter()
        .enumerate()
        .filter(|(k, _)| !used[*k])
        .map(|(k, b)| (b.center.distance(p), k))
        .filter(|(d, _)| *d < within)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, k)| k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::{Lens, Model};
    use crate::synth::{Pose, render};

    /// The sheet lying on the bed at `offset`, turned by `angle` radians.
    pub(crate) fn sheet_scene(offset: Point, angle: f64) -> impl Fn(Point) -> Option<[u8; 3]> {
        move |p: Point| {
            let d = p - offset;
            let (s, c) = angle.sin_cos();
            let local = Point::new(c * d.x + s * d.y, -s * d.x + c * d.y);
            sheet::dark_at(local).map(|dark| if dark { [20, 20, 20] } else { [235, 235, 230] })
        }
    }

    #[test]
    fn the_sheet_is_found_with_its_orientation() {
        let mut lens = Lens::ideal(Model::Standard, 640, 480, 520.0);
        lens.k = vec![-0.2, 0.05];
        let pose = Pose::overhead(150.0, 100.0, 420.0, [0.02, -0.01, 0.05]);
        let offset = Point::new(70.0, 40.0);
        let pic = render(&lens, &pose, [120, 110, 100], sheet_scene(offset, 0.3));
        let found = find_sheet(&pic).unwrap();
        assert!(found.len() >= 55, "{}", found.len());
        // Every found dot is where the truth puts it.
        for ((i, j), px) in &found {
            let local = sheet::point(*i, *j);
            let (s, c) = 0.3f64.sin_cos();
            let world = offset + Point::new(c * local.x - s * local.y, s * local.x + c * local.y);
            let truth = lens.project(pose.ray(world).unwrap());
            assert!(px.distance(truth) < 0.6, "({i}, {j}) {px:?} {truth:?}");
        }
        // A picture without the sheet says so.
        let empty = Picture::filled(64, 48, [200, 200, 200]);
        assert!(find_sheet(&empty).is_err());
    }
}
