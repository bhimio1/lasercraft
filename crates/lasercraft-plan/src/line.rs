//! Line mode: kerf offset, tabs, cut order and tracing
//! (`docs/spec/planning.md`).

use lasercraft_geom::Point;
use lasercraft_geom::boolean::{Join, area, offset};
use lasercraft_project::Layer;

use crate::emit::{Emitter, Feed};
use crate::order::{Outline, containers, order};
use crate::{TOLERANCE, Warning};

/// An outline ready to trace: one flag per edge says whether the laser is on
/// (edge `i` runs from point `i` to `i + 1`, wrapping when closed).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cut {
    pub outline: Outline,
    pub lit: Vec<bool>,
}

fn edge_count(o: &Outline) -> usize {
    if o.closed {
        o.points.len()
    } else {
        o.points.len().saturating_sub(1)
    }
}

/// Offsets one closed outline by `distance` (positive grows it).
fn offset_outline(o: &Outline, distance: f64) -> Vec<Outline> {
    let mut points = o.points.clone();
    // Same winding for every input, so the sign means the same thing.
    if area(&points) < 0.0 {
        points.reverse();
    }
    offset(&[points], distance, Join::Round(TOLERANCE))
        .into_iter()
        .filter(|c| c.len() >= 3)
        .map(|points| Outline {
            shape: o.shape,
            points,
            closed: true,
        })
        .collect()
}

/// Applies the layer's kerf to closed outlines: outward for outer ones,
/// inward for holes. Returns the outlines with their nesting depth.
fn apply_kerf(
    outlines: Vec<Outline>,
    layer: &Layer,
    warnings: &mut Vec<Warning>,
) -> Vec<(Outline, usize)> {
    let depths: Vec<usize> = containers(&outlines).into_iter().map(|c| c.len()).collect();
    if layer.kerf == 0.0 {
        return outlines.into_iter().zip(depths).collect();
    }
    let open = outlines.iter().filter(|o| !o.closed).count();
    if open > 0 {
        warnings.push(Warning {
            layer: layer.color,
            shape: None,
            message: format!(
                "kerf applies to closed shapes only; {open} open outline(s) cut as drawn"
            ),
        });
    }
    let mut out = Vec::new();
    for (o, depth) in outlines.into_iter().zip(depths) {
        if !o.closed {
            out.push((o, depth));
            continue;
        }
        let sign = if depth % 2 == 0 { 1.0 } else { -1.0 };
        let offsets = offset_outline(&o, sign * layer.kerf);
        if offsets.is_empty() {
            warnings.push(Warning {
                layer: layer.color,
                shape: Some(o.shape),
                message: "the kerf offset leaves nothing of an outline; it is not cut".to_owned(),
            });
        }
        out.extend(offsets.into_iter().map(|c| (c, depth)));
    }
    out
}

/// Splits a closed outline at tab boundaries and marks the tab edges unlit.
/// Returns `None` when the tabs would take half the perimeter or more.
fn with_tabs(o: &Outline, tabs: u32, width: f64) -> Option<Cut> {
    let n = o.points.len();
    let edge = |i: usize| (o.points[i], o.points[(i + 1) % n]);
    let perimeter: f64 = (0..n).map(|i| edge(i).0.distance(edge(i).1)).sum();
    let count = f64::from(tabs);
    if count * width >= perimeter / 2.0 {
        return None;
    }
    let gaps: Vec<(f64, f64)> = (0..tabs)
        .map(|i| {
            let center = (f64::from(i) + 0.5) * perimeter / count;
            (center - width / 2.0, center + width / 2.0)
        })
        .collect();
    let in_gap = |s: f64| gaps.iter().any(|(a, b)| s > *a && s < *b);
    let mut breaks: Vec<f64> = gaps.iter().flat_map(|(a, b)| [*a, *b]).collect();
    breaks.sort_by(f64::total_cmp);
    let mut points = Vec::new();
    let mut lit = Vec::new();
    let mut s0 = 0.0;
    for i in 0..n {
        let (a, b) = edge(i);
        let len = a.distance(b);
        let s1 = s0 + len;
        let mut cuts = vec![s0];
        cuts.extend(
            breaks
                .iter()
                .copied()
                .filter(|s| *s > s0 + 1e-9 && *s < s1 - 1e-9),
        );
        cuts.push(s1);
        for w in cuts.windows(2) {
            let t = if len > 0.0 { (w[0] - s0) / len } else { 0.0 };
            points.push(a + (b - a) * t);
            lit.push(!in_gap((w[0] + w[1]) / 2.0));
        }
        s0 = s1;
    }
    Some(Cut {
        outline: Outline {
            shape: o.shape,
            points,
            closed: true,
        },
        lit,
    })
}

/// Turns outlines into cuts: kerf, then tabs on outer closed outlines.
pub(crate) fn cuts(outlines: Vec<Outline>, layer: &Layer, warnings: &mut Vec<Warning>) -> Vec<Cut> {
    apply_kerf(outlines, layer, warnings)
        .into_iter()
        .map(|(o, depth)| {
            if layer.tabs > 0 && o.closed && depth == 0 {
                if let Some(cut) = with_tabs(&o, layer.tabs, layer.tab_width) {
                    return cut;
                }
                warnings.push(Warning {
                    layer: layer.color,
                    shape: Some(o.shape),
                    message: "the tabs would take half the outline or more; cut without tabs"
                        .to_owned(),
                });
            }
            let lit = vec![true; edge_count(&o)];
            Cut { outline: o, lit }
        })
        .collect()
}

/// Plans every pass of a line layer.
pub(crate) fn plan_layer(cuts: &[Cut], passes: u32, feed: Feed, out: &mut Emitter) {
    let outlines: Vec<Outline> = cuts.iter().map(|c| c.outline.clone()).collect();
    let entries = order(&outlines, out.pos);
    for pass in 1..=passes {
        let feed = Feed { pass, ..feed };
        for e in &entries {
            let cut = &cuts[e.index];
            let o = &cut.outline;
            let n = o.points.len();
            let edges: Vec<(Point, Point, bool)> = if o.closed {
                (0..n)
                    .map(|k| (e.vertex + k) % n)
                    .map(|i| (o.points[i], o.points[(i + 1) % n], cut.lit[i]))
                    .collect()
            } else if e.reversed {
                (0..n - 1)
                    .rev()
                    .map(|i| (o.points[i + 1], o.points[i], cut.lit[i]))
                    .collect()
            } else {
                (0..n - 1)
                    .map(|i| (o.points[i], o.points[i + 1], cut.lit[i]))
                    .collect()
            };
            let Some(first) = edges.first() else { continue };
            out.travel(first.0, &feed, Some(o.shape));
            for (_, to, lit) in edges {
                out.feed(lit, to, &feed, o.shape);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(shape: u32, x: f64, y: f64, s: f64) -> Outline {
        Outline {
            shape,
            points: vec![
                Point::new(x, y),
                Point::new(x + s, y),
                Point::new(x + s, y + s),
                Point::new(x, y + s),
            ],
            closed: true,
        }
    }

    fn polygon_area(o: &Outline) -> f64 {
        area(&o.points).abs()
    }

    #[test]
    fn offset_sign_ignores_winding() {
        let cw = square(1, 0.0, 0.0, 10.0);
        let mut ccw = cw.clone();
        ccw.points.reverse();
        for o in [cw, ccw] {
            let grown = offset_outline(&o, 1.0);
            assert_eq!(grown.len(), 1);
            let a = polygon_area(&grown[0]);
            assert!(
                (a - (100.0 + 40.0 + std::f64::consts::PI)).abs() < 0.05,
                "{a}"
            );
            let shrunk = offset_outline(&o, -1.0);
            assert!((polygon_area(&shrunk[0]) - 64.0).abs() < 1e-6);
        }
    }

    #[test]
    fn kerf_grows_parts_and_shrinks_holes() {
        let mut layer = Layer::new(0);
        layer.kerf = 0.5;
        let mut warnings = Vec::new();
        let out = apply_kerf(
            vec![
                square(1, 0.0, 0.0, 20.0),
                square(2, 5.0, 5.0, 10.0),
                square(3, 8.0, 8.0, 0.5),
            ],
            &layer,
            &mut warnings,
        );
        // Outer grows past 20 × 20; the hole shrinks to 9 × 9; the tiny
        // square inside the hole is a part again (depth 2) and grows.
        assert_eq!(out.len(), 3);
        let outer = out.iter().find(|(o, _)| o.shape == 1).unwrap();
        assert!(outer.0.bounds().width() > 20.99);
        let hole = out.iter().find(|(o, _)| o.shape == 2).unwrap();
        assert!((hole.0.bounds().width() - 9.0).abs() < 1e-6);
        assert_eq!(hole.1, 1);
        let island = out.iter().find(|(o, _)| o.shape == 3).unwrap();
        assert!((island.0.bounds().width() - 1.5).abs() < 1e-6);
        assert!(warnings.is_empty());

        // A hole smaller than the kerf disappears with a warning.
        layer.kerf = 0.3;
        let out = apply_kerf(
            vec![square(1, 0.0, 0.0, 20.0), square(2, 5.0, 5.0, 0.5)],
            &layer,
            &mut warnings,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].shape, Some(2));
    }

    #[test]
    fn tabs_leave_evenly_spaced_unlit_gaps() {
        // 40 mm perimeter, 2 tabs of 2 mm centered at 10 and 30 mm.
        let cut = with_tabs(&square(1, 0.0, 0.0, 10.0), 2, 2.0).unwrap();
        let o = &cut.outline;
        let n = o.points.len();
        let unlit: f64 = (0..n)
            .filter(|i| !cut.lit[*i])
            .map(|i| o.points[i].distance(o.points[(i + 1) % n]))
            .sum();
        assert!((unlit - 4.0).abs() < 1e-9, "{unlit}");
        let first_gap = (0..n).find(|i| !cut.lit[*i]).unwrap();
        assert_eq!(o.points[first_gap], Point::new(9.0, 0.0));
        assert_eq!(cut.lit.len(), n);
        // Too many tabs for the perimeter.
        assert!(with_tabs(&square(1, 0.0, 0.0, 10.0), 10, 2.0).is_none());
    }

    #[test]
    fn tabs_skip_holes_and_open_outlines() {
        let mut layer = Layer::new(0);
        layer.tabs = 1;
        let mut warnings = Vec::new();
        let open = Outline {
            shape: 3,
            points: vec![Point::new(0.0, 30.0), Point::new(10.0, 30.0)],
            closed: false,
        };
        let cuts = cuts(
            vec![square(1, 0.0, 0.0, 20.0), square(2, 5.0, 5.0, 10.0), open],
            &layer,
            &mut warnings,
        );
        let unlit = |c: &Cut| c.lit.iter().filter(|l| !**l).count();
        // One tab centered on the corner at (20, 20): two unlit edges.
        assert_eq!(unlit(&cuts[0]), 2);
        assert_eq!(unlit(&cuts[1]), 0);
        assert_eq!(cuts[2].lit, [true]);
    }
}
