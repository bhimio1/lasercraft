//! The pixel half of Trace to vectors (`docs/spec/trace.md`, "Method"
//! steps 1–3): which pixels are ink, clean-up, and the outlines along pixel
//! edges. Curves and placement are `lasercraft-geom`'s.

use crate::{Adjust, Gray};

/// How a pixel counts as ink.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InkRule {
    /// Darkest brightness that counts, after the adjustments.
    pub from: u8,
    /// Lightest brightness that counts, after the adjustments.
    pub to: u8,
    /// Every pixel that is not pure white counts (the brightness range is
    /// not used).
    pub transparency: bool,
    /// Pixels darker than their surroundings count (the brightness range is
    /// not used).
    pub edges: bool,
}

impl Default for InkRule {
    fn default() -> Self {
        Self {
            from: 0,
            to: 128,
            transparency: false,
            edges: false,
        }
    }
}

/// Levels a pixel must be below its neighbourhood's mean, in edge mode.
const EDGE_DROP: i64 = 12;

/// A pixel mask, row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `true` for ink.
    pub ink: Vec<bool>,
}

impl Mask {
    fn at(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && x < i64::from(self.width)
            && y < i64::from(self.height)
            && self.ink[y as usize * self.width as usize + x as usize]
    }

    /// Ink pixels.
    #[must_use]
    pub fn count(&self) -> usize {
        self.ink.iter().filter(|i| **i).count()
    }
}

/// Which pixels are ink, inside `area` (`[x0, y0, x1, y1]`, pixels, end
/// exclusive) or everywhere.
#[must_use]
pub fn ink_map(gray: &Gray, adjust: &Adjust, rule: &InkRule, area: Option<[u32; 4]>) -> Mask {
    let (w, h) = (gray.width(), gray.height());
    let table = adjust.table();
    let level = |i: usize| table[usize::from(gray.pixels()[i])];
    let [x0, y0, x1, y1] = area.map_or([0, 0, w, h], |[a, b, c, d]| {
        [a.min(w), b.min(h), c.min(w), d.min(h)]
    });
    let mean = rule.edges.then(|| local_means(gray, &table));
    let mut ink = vec![false; w as usize * h as usize];
    for y in y0..y1 {
        for x in x0..x1 {
            let i = y as usize * w as usize + x as usize;
            ink[i] = if rule.transparency {
                gray.pixels()[i] < 255
            } else if let Some(mean) = &mean {
                i64::from(level(i)) < mean[i] - EDGE_DROP
            } else {
                (rule.from..=rule.to).contains(&level(i))
            };
        }
    }
    Mask {
        width: w,
        height: h,
        ink,
    }
}

/// The mean adjusted level around each pixel: a square of side 1/20 of the
/// shorter side, at least 5, clipped at the edges.
fn local_means(gray: &Gray, table: &[u8; 256]) -> Vec<i64> {
    let (w, h) = (gray.width() as usize, gray.height() as usize);
    let side = (w.min(h) / 20).max(5);
    let r = side / 2;
    // Summed-area table with a zero row and column.
    let mut sum = vec![0i64; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0i64;
        for x in 0..w {
            row += i64::from(table[usize::from(gray.pixels()[y * w + x])]);
            sum[(y + 1) * (w + 1) + x + 1] = sum[y * (w + 1) + x + 1] + row;
        }
    }
    let mut out = vec![0i64; w * h];
    for y in 0..h {
        let (ya, yb) = (y.saturating_sub(r), (y + r + 1).min(h));
        for x in 0..w {
            let (xa, xb) = (x.saturating_sub(r), (x + r + 1).min(w));
            let total = sum[yb * (w + 1) + xb] - sum[ya * (w + 1) + xb] - sum[yb * (w + 1) + xa]
                + sum[ya * (w + 1) + xa];
            out[y * w + x] = total / ((xb - xa) * (yb - ya)) as i64;
        }
    }
    out
}

/// Labels the regions of pixels equal to `value`, joined at edges (and at
/// corners with `corners`); returns each region's pixels.
fn regions(mask: &Mask, value: bool, corners: bool) -> Vec<Vec<usize>> {
    let (w, h) = (mask.width as i64, mask.height as i64);
    let mut seen = vec![false; mask.ink.len()];
    let mut out = Vec::new();
    let mut stack = Vec::new();
    let steps: &[(i64, i64)] = if corners {
        &[
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ]
    } else {
        &[(1, 0), (-1, 0), (0, 1), (0, -1)]
    };
    for start in 0..mask.ink.len() {
        if seen[start] || mask.ink[start] != value {
            continue;
        }
        seen[start] = true;
        stack.push(start);
        let mut region = Vec::new();
        while let Some(i) = stack.pop() {
            region.push(i);
            let (x, y) = ((i as i64) % w, (i as i64) / w);
            for (dx, dy) in steps {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let n = (ny * w + nx) as usize;
                if !seen[n] && mask.ink[n] == value {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        out.push(region);
    }
    out
}

/// Drops ink regions smaller than `speck` pixels, then fills holes smaller
/// than `speck` that do not reach the image's border.
pub fn clean(mask: &mut Mask, speck: usize) {
    if speck <= 1 {
        return;
    }
    for region in regions(mask, true, true) {
        if region.len() < speck {
            for i in region {
                mask.ink[i] = false;
            }
        }
    }
    let (w, h) = (mask.width as usize, mask.height as usize);
    let on_border = |i: usize| {
        let (x, y) = (i % w, i / w);
        x == 0 || y == 0 || x + 1 == w || y + 1 == h
    };
    for region in regions(mask, false, false) {
        if region.len() < speck && !region.iter().any(|i| on_border(*i)) {
            for i in region {
                mask.ink[i] = true;
            }
        }
    }
}

/// Directions on the pixel-corner grid, clockwise on screen (y down).
const STEP: [(i64, i64); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// The outlines of the ink: closed loops of pixel corners (only the corners
/// where the outline turns). Ink lies on the right of the direction of
/// travel (y down), so outer outlines run clockwise on screen (positive
/// area by [`area`]) and hole outlines the other way. Ink pixels touching
/// only at a corner count as joined.
#[must_use]
pub fn outlines(mask: &Mask) -> Vec<Vec<(i64, i64)>> {
    let (w, h) = (i64::from(mask.width), i64::from(mask.height));
    let cw = (w + 1) as usize;
    // Outgoing edges per corner: bit d set for a step in direction d.
    let mut out = vec![0u8; cw * (h + 1) as usize];
    let corner = |x: i64, y: i64| y as usize * cw + x as usize;
    for y in 0..h {
        for x in 0..w {
            if !mask.at(x, y) {
                continue;
            }
            if !mask.at(x, y - 1) {
                out[corner(x, y)] |= 1 << 0;
            }
            if !mask.at(x + 1, y) {
                out[corner(x + 1, y)] |= 1 << 1;
            }
            if !mask.at(x, y + 1) {
                out[corner(x + 1, y + 1)] |= 1 << 2;
            }
            if !mask.at(x - 1, y) {
                out[corner(x, y + 1)] |= 1 << 3;
            }
        }
    }
    let mut loops = Vec::new();
    // Loops start at corners with one way out, so the first step at a
    // corner shared by two loops follows the same rule as later ones.
    let starts = (0..out.len())
        .filter(|i| out[*i].count_ones() == 1)
        .collect::<Vec<_>>();
    for start in starts.into_iter().chain(0..out.len()) {
        while out[start] != 0 {
            let (sx, sy) = ((start % cw) as i64, (start / cw) as i64);
            let first = out[start].trailing_zeros() as usize;
            let mut points = vec![(sx, sy)];
            let (mut x, mut y, mut d) = (sx, sy, first);
            out[start] &= !(1 << d);
            loop {
                x += STEP[d].0;
                y += STEP[d].1;
                let here = corner(x, y);
                // Left first joins ink touching at a corner.
                let next = [(d + 3) % 4, d, (d + 1) % 4]
                    .into_iter()
                    .find(|n| out[here] & (1 << n) != 0);
                let Some(n) = next else {
                    break;
                };
                if n != d {
                    points.push((x, y));
                }
                out[here] &= !(1 << n);
                d = n;
            }
            // The start is a corner only when the loop turns there.
            if d == first && points.len() > 1 {
                points.remove(0);
            }
            if points.len() >= 4 {
                loops.push(points);
            }
        }
    }
    loops
}

/// Twice the signed area of a loop (shoelace, y down): positive for outer
/// outlines.
#[must_use]
pub fn area(points: &[(i64, i64)]) -> i64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(rows: &[&str]) -> Mask {
        Mask {
            width: rows[0].len() as u32,
            height: rows.len() as u32,
            ink: rows
                .iter()
                .flat_map(|r| r.chars().map(|c| c == '#'))
                .collect(),
        }
    }

    #[test]
    fn ink_follows_the_range_transparency_and_edges() {
        let gray = Gray::new(4, 1, vec![0, 100, 200, 255]).unwrap();
        let rule = InkRule::default();
        let m = ink_map(&gray, &Adjust::default(), &rule, None);
        assert_eq!(m.ink, [true, true, false, false]);
        let t = InkRule {
            transparency: true,
            ..rule
        };
        assert_eq!(
            ink_map(&gray, &Adjust::default(), &t, None).ink,
            [true, true, true, false]
        );
        let area = ink_map(&gray, &Adjust::default(), &rule, Some([1, 0, 4, 1]));
        assert_eq!(area.ink, [false, true, false, false]);
        // A thin dark line on an unevenly lit page.
        let mut px = Vec::new();
        for y in 0..20u32 {
            for x in 0..20u32 {
                let page = 150 + (x * 5) as u8;
                px.push(if y == 10 { page - 60 } else { page });
            }
        }
        let page = Gray::new(20, 20, px).unwrap();
        let e = InkRule {
            edges: true,
            ..rule
        };
        let m = ink_map(&page, &Adjust::default(), &e, None);
        assert!((0..20).all(|x| m.ink[10 * 20 + x]));
        assert_eq!(m.count(), 20);
    }

    #[test]
    fn specks_go_and_small_holes_fill() {
        let mut m = mask(&[
            "#.......", //
            "..####..", "..#.##..", "..####..", "........",
        ]);
        clean(&mut m, 2);
        assert_eq!(
            m,
            mask(&[
                "........", //
                "..####..", "..####..", "..####..", "........",
            ])
        );
    }

    #[test]
    fn outlines_run_along_pixel_edges() {
        let m = mask(&[
            "......", //
            ".####.", ".#..#.", ".####.", "......",
        ]);
        let mut loops = outlines(&m);
        loops.sort_by_key(|l| -area(l));
        assert_eq!(loops.len(), 2);
        assert_eq!(loops[0], [(1, 1), (5, 1), (5, 4), (1, 4)]);
        assert_eq!(area(&loops[0]), 24);
        assert_eq!(area(&loops[1]), -4, "the hole runs the other way");
        // Pixels touching at a corner are one outline.
        let d = mask(&["#.", ".#"]);
        let loops = outlines(&d);
        assert_eq!(loops.len(), 1);
        assert_eq!(area(&loops[0]), 4);
    }
}
