//! Darkness to dot power (`docs/spec/images.md`, "From bitmap to dots").

use std::f64::consts::PI;

/// How darkness becomes dots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
    /// Power follows darkness, quantized to 1/255.
    Grayscale,
    /// On at or above this darkness (0–1).
    Threshold(f64),
    /// Floyd–Steinberg error diffusion.
    FloydSteinberg,
    /// Jarvis–Judice–Ninke error diffusion.
    Jarvis,
    /// Stucki error diffusion.
    Stucki,
    /// Atkinson error diffusion.
    Atkinson,
    /// 8 × 8 Bayer ordered dither.
    Ordered,
    /// Clustered dots on a rotated screen.
    Halftone {
        /// Screen period, in dots.
        cell: f64,
        /// Screen angle, degrees.
        angle: f64,
    },
}

/// Error-diffusion kernel: (dx, dy, weight) and the divisor.
type Kernel = (&'static [(i64, i64, f64)], f64);

const FLOYD_STEINBERG: Kernel = (&[(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)], 16.0);

const JARVIS: Kernel = (
    &[
        (1, 0, 7.0),
        (2, 0, 5.0),
        (-2, 1, 3.0),
        (-1, 1, 5.0),
        (0, 1, 7.0),
        (1, 1, 5.0),
        (2, 1, 3.0),
        (-2, 2, 1.0),
        (-1, 2, 3.0),
        (0, 2, 5.0),
        (1, 2, 3.0),
        (2, 2, 1.0),
    ],
    48.0,
);

const STUCKI: Kernel = (
    &[
        (1, 0, 8.0),
        (2, 0, 4.0),
        (-2, 1, 2.0),
        (-1, 1, 4.0),
        (0, 1, 8.0),
        (1, 1, 4.0),
        (2, 1, 2.0),
        (-2, 2, 1.0),
        (-1, 2, 2.0),
        (0, 2, 4.0),
        (1, 2, 2.0),
        (2, 2, 1.0),
    ],
    42.0,
);

/// Atkinson spreads 6/8 of the error; the rest is dropped.
const ATKINSON: Kernel = (
    &[
        (1, 0, 1.0),
        (2, 0, 1.0),
        (-1, 1, 1.0),
        (0, 1, 1.0),
        (1, 1, 1.0),
        (0, 2, 1.0),
    ],
    8.0,
);

/// The 8 × 8 Bayer index matrix, built recursively from the 2 × 2 one.
fn bayer8() -> [[u8; 8]; 8] {
    let mut m = [[0u8; 8]; 8];
    let mut size = 1;
    while size < 8 {
        for y in 0..size {
            for x in 0..size {
                let v = 4 * m[y][x];
                m[y][x] = v;
                m[y][x + size] = v + 2;
                m[y + size][x] = v + 3;
                m[y + size][x + size] = v + 1;
            }
        }
        size *= 2;
    }
    m
}

fn diffuse(values: &mut [f64], mask: &[bool], cols: usize, (kernel, divisor): Kernel) {
    let rows = values.len() / cols.max(1);
    for y in 0..rows {
        for x in 0..cols {
            let i = y * cols + x;
            if !mask[i] {
                continue;
            }
            let old = values[i];
            let new = if old >= 0.5 { 1.0 } else { 0.0 };
            values[i] = new;
            let error = old - new;
            for &(dx, dy, w) in kernel {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < 0 || nx >= cols as i64 || ny >= rows as i64 {
                    continue;
                }
                let j = ny as usize * cols + nx as usize;
                if mask[j] {
                    values[j] += error * w / divisor;
                }
            }
        }
    }
}

/// The cosine spot function on a cell: 0 at the center, 1 at the corners;
/// `fx`, `fy` in −1–1.
fn cosine_spot(fx: f64, fy: f64) -> f64 {
    1.0 - ((PI * fx).cos() + (PI * fy).cos() + 2.0) / 4.0
}

/// Samples per cell side used to make dot area follow darkness.
const SPOT_SAMPLES: usize = 64;

/// Sorted spot values over a cell, so a spot value's rank is the share of
/// the cell a dot covers when it reaches that value.
fn spot_ranks() -> Vec<f64> {
    let n = SPOT_SAMPLES;
    let mut v: Vec<f64> = (0..n * n)
        .map(|i| {
            let fx = 2.0 * ((i % n) as f64 + 0.5) / n as f64 - 1.0;
            let fy = 2.0 * ((i / n) as f64 + 0.5) / n as f64 - 1.0;
            cosine_spot(fx, fy)
        })
        .collect();
    v.sort_by(f64::total_cmp);
    v
}

/// The halftone threshold at dot (`x`, `y`): the share of a cell covered by
/// the time the dot grows to this point, so coverage tracks darkness.
fn spot(ranks: &[f64], x: f64, y: f64, cell: f64, angle: f64) -> f64 {
    let (s, c) = angle.to_radians().sin_cos();
    let u = (x * c + y * s) / cell;
    let v = (-x * s + y * c) / cell;
    let value = cosine_spot(2.0 * (u - u.floor()) - 1.0, 2.0 * (v - v.floor()) - 1.0);
    let below = ranks.partition_point(|r| *r < value);
    (below as f64 + 0.5) / ranks.len() as f64
}

/// Turns darkness (0 white – 1 black) into power fractions in place. `mask`
/// marks dots inside the image; others become 0 and take no error. Dot
/// (col, row) of the grid has global indices (`col0` + col, `row0` + row),
/// which ordered and halftone screens use so neighbouring images line up.
pub fn dither(values: &mut [f64], mask: &[bool], cols: usize, origin: (i64, i64), method: Method) {
    let on = |b: bool| if b { 1.0 } else { 0.0 };
    match method {
        Method::Grayscale => {
            for v in values.iter_mut() {
                *v = (v.clamp(0.0, 1.0) * 255.0).round() / 255.0;
            }
        }
        Method::Threshold(t) => values.iter_mut().for_each(|v| *v = on(*v >= t)),
        Method::FloydSteinberg => diffuse(values, mask, cols, FLOYD_STEINBERG),
        Method::Jarvis => diffuse(values, mask, cols, JARVIS),
        Method::Stucki => diffuse(values, mask, cols, STUCKI),
        Method::Atkinson => diffuse(values, mask, cols, ATKINSON),
        Method::Ordered => {
            let m = bayer8();
            for (i, v) in values.iter_mut().enumerate() {
                let x = (origin.0 + (i % cols) as i64).rem_euclid(8) as usize;
                let y = (origin.1 + (i / cols) as i64).rem_euclid(8) as usize;
                *v = on(*v > (f64::from(m[y][x]) + 0.5) / 64.0);
            }
        }
        Method::Halftone { cell, angle } => {
            let ranks = spot_ranks();
            for (i, v) in values.iter_mut().enumerate() {
                let x = (origin.0 + (i % cols) as i64) as f64 + 0.5;
                let y = (origin.1 + (i / cols) as i64) as f64 + 0.5;
                *v = on(*v > 0.0 && *v >= spot(&ranks, x, y, cell.max(1.0), angle));
            }
        }
    }
    for (v, inside) in values.iter_mut().zip(mask) {
        if !inside {
            *v = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(method: Method, level: f64, n: usize) -> f64 {
        let mut v = vec![level; n * n];
        dither(&mut v, &vec![true; n * n], n, (0, 0), method);
        assert!(v.iter().all(|x| *x == 0.0 || *x == 1.0), "{method:?}");
        v.iter().sum::<f64>() / (n * n) as f64
    }

    #[test]
    fn bayer_matrix_is_a_permutation_with_the_classic_corner() {
        let m = bayer8();
        let mut seen: Vec<u8> = m.iter().flatten().copied().collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..64).collect::<Vec<u8>>());
        assert_eq!(m[0][..4], [0, 32, 8, 40]);
        assert_eq!(m[1][..4], [48, 16, 56, 24]);
    }

    #[test]
    fn binary_methods_keep_the_average_darkness() {
        for method in [
            Method::FloydSteinberg,
            Method::Jarvis,
            Method::Stucki,
            Method::Ordered,
        ] {
            for level in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let got = run(method, level, 64);
                assert!((got - level).abs() < 0.03, "{method:?} {level}: {got}");
            }
        }
        // Clustered dots grow in steps, so coverage is only near the level.
        for angle in [0.0, 45.0] {
            for level in [0.1, 0.25, 0.5, 0.75, 0.9] {
                let got = run(Method::Halftone { cell: 16.0, angle }, level, 128);
                assert!(
                    (got - level).abs() < 0.07,
                    "halftone {angle}° {level}: {got}"
                );
            }
        }
        // Atkinson drops a quarter of the error, so mid-tones lose a little.
        let got = run(Method::Atkinson, 0.5, 64);
        assert!((got - 0.5).abs() < 0.1, "{got}");
        assert_eq!(run(Method::Atkinson, 1.0, 16), 1.0);
        assert_eq!(run(Method::Atkinson, 0.0, 16), 0.0);
    }

    #[test]
    fn floyd_steinberg_textbook_case() {
        // One row: 0.5 rounds up; −0.5 error sent right (7/16) leaves 0.28…
        let mut v = vec![0.5, 0.5, 0.5, 0.5];
        dither(&mut v, &[true; 4], 4, (0, 0), Method::FloydSteinberg);
        assert_eq!(v, [1.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn threshold_grayscale_and_mask() {
        let mut v = vec![0.2, 0.6, 0.6, 0.9];
        dither(
            &mut v,
            &[true, true, false, true],
            4,
            (0, 0),
            Method::Threshold(0.5),
        );
        assert_eq!(v, [0.0, 1.0, 0.0, 1.0]);
        let mut g = vec![0.5, 1.3, -0.1];
        dither(&mut g, &[true; 3], 3, (0, 0), Method::Grayscale);
        assert_eq!(g, [128.0 / 255.0, 1.0, 0.0]);
    }

    #[test]
    fn masked_dots_take_no_error() {
        let mut v = vec![0.4, 0.4, 0.4];
        dither(
            &mut v,
            &[true, false, true],
            3,
            (0, 0),
            Method::FloydSteinberg,
        );
        assert_eq!(v, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn screens_follow_global_dot_indices() {
        let level = 0.4;
        let mut a = vec![level; 16];
        let mut b = vec![level; 4];
        dither(&mut a, &[true; 16], 16, (0, 0), Method::Ordered);
        dither(&mut b, &[true; 4], 4, (8, 0), Method::Ordered);
        assert_eq!(a[8..12], b[..]);
    }
}
