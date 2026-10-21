//! Image layers: rows of dots (`docs/spec/images.md`, "Planning").

use lasercraft_geom::Point;
use lasercraft_project::raster::{Method, Raster, rasterize};
use lasercraft_project::{Dither, ImageSettings, Layer, Shape};

use crate::emit::{Emitter, Feed};

/// The dither method for a layer's image settings.
pub(crate) fn method(settings: &ImageSettings) -> Method {
    match settings.dither {
        Dither::Grayscale => Method::Grayscale,
        Dither::Threshold => Method::Threshold(0.5),
        Dither::FloydSteinberg => Method::FloydSteinberg,
        Dither::Jarvis => Method::Jarvis,
        Dither::Stucki => Method::Stucki,
        Dither::Atkinson => Method::Atkinson,
        Dither::Ordered => Method::Ordered,
        Dither::Halftone => Method::Halftone {
            cell: 25.4 / settings.halftone_cells / settings.interval,
            angle: settings.halftone_angle,
        },
    }
}

/// The image's dots on the layer's grid.
///
/// # Errors
///
/// Returns the reason when the image cannot be rasterized (too fine a grid).
pub(crate) fn raster_of(shape: &Shape, layer: &Layer) -> Result<Option<Raster>, String> {
    let Some(image) = shape.geometry().image() else {
        return Ok(None);
    };
    rasterize(
        &image.bitmap,
        &image.adjust,
        layer.image.negative,
        [image.width, image.height],
        shape.transform().m,
        layer.image.interval,
        method(&layer.image),
    )
    .map(Some)
    .map_err(|e| e.0)
}

/// How hard a dot of value `v` burns: dithered dots at full power,
/// grayscale between the layer's min and max (both already clamped).
fn dot_power(v: f64, grayscale: bool, min: f64, max: f64) -> f64 {
    if grayscale {
        min + v * (max - min)
    } else {
        max
    }
}

/// Runs of equal, non-zero value in a row: (first col, last col, value).
fn runs(raster: &Raster, row: usize) -> Vec<(usize, usize, f64)> {
    let mut out: Vec<(usize, usize, f64)> = Vec::new();
    for col in 0..raster.cols {
        let v = raster.get(col, row);
        if v <= 0.0 {
            continue;
        }
        match out.last_mut() {
            Some(run) if run.1 + 1 == col && run.2 == v => run.1 = col,
            _ => out.push((col, col, v)),
        }
    }
    out
}

/// Settings for one image's rows.
pub(crate) struct Rows {
    pub bidirectional: bool,
    pub overscan: f64,
    pub correction: f64,
    pub grayscale: bool,
    pub min_power: f64,
}

/// Emits one pass over `raster`, rows from the bottom of the screen up.
pub(crate) fn emit(raster: &Raster, shape: u32, rows: &Rows, feed: &Feed, out: &mut Emitter) {
    let step = raster.interval;
    let x = |col: i64| col as f64 * step;
    let mut emitted = 0usize;
    for row in (0..raster.rows).rev() {
        let row_runs = runs(raster, row);
        if row_runs.is_empty() {
            continue;
        }
        let y = (raster.row0 + row as i64) as f64 * step + step / 2.0;
        let forward = !rows.bidirectional || emitted.is_multiple_of(2);
        emitted += 1;
        let half = rows.correction / 2.0;
        // (start x, end x, power) in burn direction.
        let mut segments: Vec<(f64, f64, f64)> = row_runs
            .iter()
            .map(|&(c0, c1, v)| {
                let a = x(raster.col0 + c0 as i64) + half;
                let b = x(raster.col0 + c1 as i64 + 1) - half;
                (
                    a,
                    b,
                    dot_power(v, rows.grayscale, rows.min_power, feed.power),
                )
            })
            .collect();
        let dir = if forward { 1.0 } else { -1.0 };
        if !forward {
            segments.reverse();
            for s in &mut segments {
                std::mem::swap(&mut s.0, &mut s.1);
            }
        }
        let at = |u: f64| Point::new(u, y);
        let (first, last) = (segments[0], segments[segments.len() - 1]);
        out.travel(at(first.0 - dir * rows.overscan), feed, Some(shape));
        for (a, b, power) in segments {
            out.feed(false, at(a), feed, shape);
            out.burn(at(b), power, feed, shape);
        }
        out.feed(false, at(last.1 + dir * rows.overscan), feed, shape);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_join_equal_neighbours() {
        let r = Raster {
            col0: 0,
            row0: 0,
            interval: 1.0,
            cols: 6,
            rows: 1,
            values: vec![1.0, 1.0, 0.0, 0.5, 0.25, 0.25],
        };
        assert_eq!(runs(&r, 0), [(0, 1, 1.0), (3, 3, 0.5), (4, 5, 0.25)]);
        assert_eq!(dot_power(0.5, true, 10.0, 50.0), 30.0);
        assert_eq!(dot_power(1.0, false, 10.0, 50.0), 50.0);
    }
}
