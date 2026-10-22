//! Small dense linear algebra for calibration: solving normal equations
//! and the smallest eigenvector of a symmetric matrix. Index loops read
//! like the textbook formulas here.
#![allow(clippy::needless_range_loop)]

/// A row-major `n × n` matrix.
pub(crate) type Square = Vec<Vec<f64>>;

/// Solves `a · x = b` by Gaussian elimination with partial pivoting.
/// `None` when `a` is singular.
pub(crate) fn solve(mut a: Square, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-300 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            if f == 0.0 {
                continue;
            }
            for k in col..n {
                a[row][k] -= f * a[col][k];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let s: f64 = (row + 1..n).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - s) / a[row][row];
    }
    x.iter().all(|v| v.is_finite()).then_some(x)
}

/// The unit eigenvector of the smallest eigenvalue of symmetric `a`
/// (cyclic Jacobi rotations).
pub(crate) fn smallest_eigenvector(mut a: Square) -> Vec<f64> {
    let n = a.len();
    let mut v: Square = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    for _ in 0..100 {
        let off: f64 = (0..n)
            .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
            .map(|(i, j)| a[i][j] * a[i][j])
            .sum();
        if off < 1e-24 {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let (vkp, vkq) = (v[k][p], v[k][q]);
                    v[k][p] = c * vkp - s * vkq;
                    v[k][q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let i = (0..n)
        .min_by(|&i, &j| a[i][i].total_cmp(&a[j][j]))
        .unwrap_or(0);
    let col: Vec<f64> = (0..n).map(|k| v[k][i]).collect();
    let norm = col.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-300);
    col.into_iter().map(|x| x / norm).collect()
}

/// `aᵀa` for rows `a`.
pub(crate) fn gram(rows: &[Vec<f64>], n: usize) -> Square {
    let mut m = vec![vec![0.0; n]; n];
    for r in rows {
        for i in 0..n {
            for j in 0..n {
                m[i][j] += r[i] * r[j];
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_and_finds_the_null_direction() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let x = solve(a, vec![3.0, 5.0]).unwrap();
        assert!((x[0] - 0.8).abs() < 1e-12 && (x[1] - 1.4).abs() < 1e-12);
        assert!(solve(vec![vec![1.0, 2.0], vec![2.0, 4.0]], vec![1.0, 2.0]).is_none());
        // Rows orthogonal to (1, 1, 0) / √2.
        let rows = vec![
            vec![1.0, -1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![2.0, -2.0, 3.0],
        ];
        let e = smallest_eigenvector(gram(&rows, 3));
        assert!(
            (e[0].abs() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9,
            "{e:?}"
        );
        assert!((e[0] - e[1]).abs() < 1e-9 && e[2].abs() < 1e-9);
    }
}
