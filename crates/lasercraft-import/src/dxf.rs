//! ASCII DXF, read with our own group-code parser (`docs/spec/import.md`).
//!
//! Geometry is built in DXF space (y up, file units), then scaled to
//! millimetres and mirrored so DXF (x, y) lands on document (x, H − y).

use std::collections::HashMap;

use lasercraft_geom::nodes::join;
use lasercraft_geom::path::arc;
use lasercraft_geom::{Affine, Path, Point, Segment, SubPath};

use crate::{ImportError, Imported};

/// Ends closer than this (mm) are joined.
pub const JOIN_TOLERANCE_MM: f64 = 0.01;
/// Spline evaluation points per knot span.
const SPLINE_SAMPLES: u32 = 16;
/// Deepest block nesting followed.
const MAX_INSERT_DEPTH: u32 = 8;
/// Default color: ACI 7 (black or white, shown black).
const ACI_DEFAULT: i32 = 7;

/// One entity: its type, its group codes, and (for `POLYLINE`) its vertices.
#[derive(Debug, Clone, Default)]
struct Entity {
    kind: String,
    codes: Vec<(i32, String)>,
    vertices: Vec<Entity>,
}

impl Entity {
    fn text(&self, code: i32) -> Option<&str> {
        self.codes
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, v)| v.as_str())
    }

    fn num(&self, code: i32) -> Option<f64> {
        self.text(code)?
            .trim()
            .parse()
            .ok()
            .filter(|v: &f64| v.is_finite())
    }

    fn num_or(&self, code: i32, default: f64) -> f64 {
        self.num(code).unwrap_or(default)
    }

    fn int(&self, code: i32) -> Option<i32> {
        self.text(code)?.trim().parse().ok()
    }

    fn point(&self, x: i32, y: i32) -> Point {
        Point::new(self.num_or(x, 0.0), self.num_or(y, 0.0))
    }

    fn all(&self, code: i32) -> Vec<f64> {
        self.codes
            .iter()
            .filter(|(c, _)| *c == code)
            .filter_map(|(_, v)| v.trim().parse().ok())
            .collect()
    }

    /// Whether the object coordinate system is mirrored (extrusion z < 0).
    fn mirrored(&self) -> bool {
        self.num(230).is_some_and(|z| z < 0.0)
    }
}

/// A drawing: units, layer colors, blocks and model-space entities.
#[derive(Default)]
struct Drawing {
    units: Option<i32>,
    layer_colors: HashMap<String, i32>,
    blocks: HashMap<String, (Point, Vec<Entity>)>,
    entities: Vec<Entity>,
}

fn pairs(text: &str) -> Result<Vec<(i32, String)>, ImportError> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::with_capacity(lines.len() / 2);
    for (i, chunk) in lines.chunks(2).enumerate() {
        let [code, value] = chunk else {
            break;
        };
        let code: i32 = code.trim().parse().map_err(|_| {
            ImportError::new(format!(
                "not a valid DXF file: line {} should be a group code",
                i * 2 + 1
            ))
        })?;
        out.push((code, value.trim().to_owned()));
    }
    Ok(out)
}

/// Splits `pairs` at code-0 markers into entities, attaching `VERTEX`es to
/// the `POLYLINE` before them.
fn entities(pairs: &[(i32, String)]) -> Vec<Entity> {
    let mut out: Vec<Entity> = Vec::new();
    let mut in_polyline = false;
    for (code, value) in pairs {
        if *code == 0 {
            let e = Entity {
                kind: value.clone(),
                ..Entity::default()
            };
            match value.as_str() {
                "VERTEX" if in_polyline => {
                    if let Some(p) = out.last_mut() {
                        p.vertices.push(e);
                    }
                    continue;
                }
                "SEQEND" => {
                    in_polyline = false;
                    continue;
                }
                "POLYLINE" => in_polyline = true,
                _ => in_polyline = false,
            }
            out.push(e);
        } else if let Some(e) = out.last_mut() {
            match e.vertices.last_mut() {
                Some(v) if in_polyline => v.codes.push((*code, value.clone())),
                _ => e.codes.push((*code, value.clone())),
            }
        }
    }
    out
}

fn read_drawing(text: &str) -> Result<Drawing, ImportError> {
    let pairs = pairs(text)?;
    let mut drawing = Drawing::default();
    let mut i = 0;
    while i < pairs.len() {
        if pairs[i] != (0, "SECTION".to_owned()) {
            i += 1;
            continue;
        }
        let name = pairs.get(i + 1).map(|(_, v)| v.clone()).unwrap_or_default();
        let end = pairs[i..]
            .iter()
            .position(|p| p.0 == 0 && p.1 == "ENDSEC")
            .map_or(pairs.len(), |n| i + n);
        let body = &pairs[(i + 2).min(end)..end];
        match name.as_str() {
            "HEADER" => {
                if let Some(at) = body.iter().position(|p| *p == (9, "$INSUNITS".to_owned())) {
                    drawing.units = body
                        .get(at + 1)
                        .filter(|p| p.0 == 70)
                        .and_then(|p| p.1.parse().ok());
                }
            }
            "TABLES" => {
                for e in entities(body).iter().filter(|e| e.kind == "LAYER") {
                    if let Some(name) = e.text(2) {
                        let color = e.int(62).map_or(ACI_DEFAULT, i32::abs);
                        drawing.layer_colors.insert(name.to_owned(), color);
                    }
                }
            }
            "BLOCKS" => {
                let mut current: Option<(String, Point, Vec<Entity>)> = None;
                for e in entities(body) {
                    match e.kind.as_str() {
                        "BLOCK" => {
                            let name = e.text(2).unwrap_or_default().to_owned();
                            current = Some((name, e.point(10, 20), Vec::new()));
                        }
                        "ENDBLK" => {
                            if let Some((name, base, list)) = current.take() {
                                drawing.blocks.insert(name, (base, list));
                            }
                        }
                        _ => {
                            if let Some((_, _, list)) = current.as_mut() {
                                list.push(e);
                            }
                        }
                    }
                }
            }
            "ENTITIES" => drawing.entities = entities(body),
            _ => {}
        }
        i = end + 1;
    }
    Ok(drawing)
}

/// Millimetres per drawing unit for `$INSUNITS`.
fn unit_scale(units: Option<i32>) -> Result<f64, ImportError> {
    Ok(match units.unwrap_or(0) {
        0 | 4 => 1.0,
        1 => 25.4,
        2 => 304.8,
        5 => 10.0,
        6 => 1000.0,
        8 => 25.4e-6,
        9 => 0.0254,
        10 => 914.4,
        14 => 100.0,
        other => {
            return Err(ImportError::new(format!(
                "unsupported DXF units ($INSUNITS {other})"
            )));
        }
    })
}

/// Approximate sRGB of an AutoCAD Color Index.
#[must_use]
pub fn aci_rgb(aci: i32) -> [u8; 3] {
    match aci {
        1 => [255, 0, 0],
        2 => [255, 255, 0],
        3 => [0, 255, 0],
        4 => [0, 255, 255],
        5 => [0, 0, 255],
        6 => [255, 0, 255],
        8 => [128, 128, 128],
        9 => [192, 192, 192],
        10..=249 => {
            let hue = f64::from(aci / 10 - 1) * 15.0;
            let value = [1.0, 0.65, 0.5, 0.3, 0.15][((aci % 10) / 2) as usize];
            let saturation = if aci % 2 == 0 { 1.0 } else { 0.5 };
            hsv(hue, saturation, value)
        }
        // 250–255 run from dark gray to white.
        250..=255 => {
            let v = u8::try_from(51 + (aci - 250) * 41).unwrap_or(255);
            [v, v, v]
        }
        _ => [0, 0, 0],
    }
}

fn hsv(hue: f64, saturation: f64, value: f64) -> [u8; 3] {
    let c = value * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = value - c;
    // Each channel is in 0..=1, so the cast cannot overflow.
    let byte = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    [byte(r), byte(g), byte(b)]
}

/// An outline piece in DXF space with its resolved ACI color.
struct Piece {
    sub: SubPath,
    aci: i32,
}

struct Builder<'a> {
    drawing: &'a Drawing,
    pieces: Vec<Piece>,
}

impl Builder<'_> {
    fn color(&self, e: &Entity, inherited: i32) -> i32 {
        match e.int(62) {
            Some(0) => inherited,
            Some(c) if c != 256 => c.abs(),
            _ => e
                .text(8)
                .and_then(|l| self.drawing.layer_colors.get(l))
                .copied()
                .unwrap_or(ACI_DEFAULT),
        }
    }

    fn add(&mut self, e: &Entity, t: &Affine, inherited: i32, depth: u32) {
        let aci = self.color(e, inherited);
        let ocs = if e.mirrored() {
            *t * Affine::scale(-1.0, 1.0)
        } else {
            *t
        };
        let sub = match e.kind.as_str() {
            "LINE" => SubPath::from_points(&[e.point(10, 20), e.point(11, 21)], false),
            "CIRCLE" => Some(circle_arc(e.point(10, 20), e.num_or(40, 0.0), 0.0, 360.0)),
            "ARC" => {
                let (a, b) = (e.num_or(50, 0.0), e.num_or(51, 360.0));
                Some(circle_arc(
                    e.point(10, 20),
                    e.num_or(40, 0.0),
                    a,
                    sweep(a, b),
                ))
            }
            "ELLIPSE" => Some(ellipse(e)),
            "LWPOLYLINE" => lwpolyline(e),
            "POLYLINE" => polyline(e),
            "SPLINE" => spline(e),
            "INSERT" => {
                self.insert(e, &ocs, aci, depth);
                None
            }
            _ => None,
        };
        let world = match e.kind.as_str() {
            "CIRCLE" | "ARC" | "LWPOLYLINE" | "POLYLINE" => ocs,
            _ => *t,
        };
        if let Some(sub) = sub.filter(|s| !s.segments.is_empty()) {
            let sub = Path::single(sub).transformed(&world).subpaths.remove(0);
            self.pieces.push(Piece { sub, aci });
        }
    }

    fn insert(&mut self, e: &Entity, t: &Affine, aci: i32, depth: u32) {
        if depth >= MAX_INSERT_DEPTH {
            return;
        }
        let Some((base, list)) = e.text(2).and_then(|n| self.drawing.blocks.get(n)) else {
            return;
        };
        let at = e.point(10, 20);
        let local = Affine::translate(at.x, at.y)
            * Affine::rotate_degrees(e.num_or(50, 0.0))
            * Affine::scale(e.num_or(41, 1.0), e.num_or(42, 1.0))
            * Affine::translate(-base.x, -base.y);
        let t = *t * local;
        for child in list {
            self.add(child, &t, aci, depth + 1);
        }
    }
}

/// Counter-clockwise sweep from `a` to `b` degrees, in (0, 360].
fn sweep(a: f64, b: f64) -> f64 {
    let s = (b - a).rem_euclid(360.0);
    if s == 0.0 { 360.0 } else { s }
}

/// A circular arc as a subpath; a full turn is closed and ends exactly at
/// its start.
fn circle_arc(center: Point, radius: f64, start: f64, sweep: f64) -> SubPath {
    let (first, mut segments) = arc(center, radius, start, sweep);
    let full = sweep >= 360.0;
    if full && let Some(Segment::Cubic(_, _, end)) = segments.last_mut() {
        *end = first;
    }
    SubPath {
        start: first,
        segments,
        closed: full,
    }
}

fn ellipse(e: &Entity) -> SubPath {
    let c = e.point(10, 20);
    let m = e.point(11, 21);
    let ratio = e.num_or(40, 1.0);
    let n = Point::new(-m.y, m.x) * ratio;
    let a = e.num_or(41, 0.0).to_degrees();
    let b = e.num_or(42, std::f64::consts::TAU).to_degrees();
    let s = sweep(a, b);
    let unit = circle_arc(Point::ZERO, 1.0, a, s);
    let t = Affine {
        m: [m.x, m.y, n.x, n.y, c.x, c.y],
    };
    Path::single(unit).transformed(&t).subpaths.remove(0)
}

/// Appends the segment from `from` to `to` with bulge `b`.
fn bulge_to(segments: &mut Vec<Segment>, from: Point, to: Point, b: f64) {
    let chord = to - from;
    let len = chord.length();
    if b.abs() < 1e-12 || len < 1e-12 {
        segments.push(Segment::Line(to));
        return;
    }
    let theta = 4.0 * b.atan();
    let normal = Point::new(-chord.y, chord.x) * (1.0 / len);
    let center = (from + to) * 0.5 + normal * (len / 2.0 / (theta / 2.0).tan());
    let radius = center.distance(from);
    let d = from - center;
    let start = d.y.atan2(d.x).to_degrees();
    let (_, mut arc_segments) = arc(center, radius, start, theta.to_degrees());
    if let Some(Segment::Cubic(_, _, end)) = arc_segments.last_mut() {
        *end = to;
    }
    segments.extend(arc_segments);
}

/// A polyline through `(point, bulge)` vertices.
fn bulged(vertices: &[(Point, f64)], closed: bool) -> Option<SubPath> {
    let (first, _) = *vertices.first()?;
    let mut segments = Vec::new();
    for pair in vertices.windows(2) {
        bulge_to(&mut segments, pair[0].0, pair[1].0, pair[0].1);
    }
    if closed && vertices.len() > 1 {
        let (last, b) = vertices[vertices.len() - 1];
        if b.abs() >= 1e-12 {
            bulge_to(&mut segments, last, first, b);
        }
    }
    Some(SubPath {
        start: first,
        segments,
        closed,
    })
}

fn lwpolyline(e: &Entity) -> Option<SubPath> {
    let mut vertices: Vec<(Point, f64)> = Vec::new();
    for (code, value) in &e.codes {
        let v: f64 = value.trim().parse().unwrap_or(0.0);
        match code {
            10 => vertices.push((Point::new(v, 0.0), 0.0)),
            20 => {
                if let Some(last) = vertices.last_mut() {
                    last.0.y = v;
                }
            }
            42 => {
                if let Some(last) = vertices.last_mut() {
                    last.1 = v;
                }
            }
            _ => {}
        }
    }
    bulged(&vertices, e.int(70).unwrap_or(0) & 1 == 1)
}

fn polyline(e: &Entity) -> Option<SubPath> {
    let flags = e.int(70).unwrap_or(0);
    if flags & (16 | 64) != 0 {
        return None;
    }
    let vertices: Vec<(Point, f64)> = e
        .vertices
        .iter()
        .filter(|v| v.int(70).unwrap_or(0) & 16 == 0)
        .map(|v| (v.point(10, 20), v.num_or(42, 0.0)))
        .collect();
    bulged(&vertices, flags & 1 == 1)
}

fn spline(e: &Entity) -> Option<SubPath> {
    let closed = e.int(70).unwrap_or(0) & 1 == 1;
    let degree = usize::try_from(e.int(71).unwrap_or(3)).ok()?;
    let knots = e.all(40);
    let xs = e.all(10);
    let ys = e.all(20);
    let control: Vec<Point> = xs
        .iter()
        .zip(&ys)
        .map(|(x, y)| Point::new(*x, *y))
        .collect();
    let mut weights = e.all(41);
    let n = control.len();
    if n == 0 || degree == 0 || knots.len() != n + degree + 1 {
        let fit: Vec<Point> = e
            .all(11)
            .iter()
            .zip(&e.all(21))
            .map(|(x, y)| Point::new(*x, *y))
            .collect();
        return SubPath::from_points(&fit, closed && fit.len() > 2);
    }
    if weights.len() != n {
        weights = vec![1.0; n];
    }
    let mut points = Vec::new();
    for span in degree..n {
        let (k0, k1) = (knots[span], knots[span + 1]);
        if k1 <= k0 {
            continue;
        }
        let first = if points.is_empty() { 0 } else { 1 };
        for i in first..=SPLINE_SAMPLES {
            let t = k0 + (k1 - k0) * f64::from(i) / f64::from(SPLINE_SAMPLES);
            points.push(de_boor(span, t, degree, &knots, &control, &weights));
        }
    }
    if closed && points.len() > 2 && points.first() == points.last() {
        points.pop();
    }
    SubPath::from_points(&points, closed && points.len() > 2)
}

/// Point of a (rational) B-spline at `t` in knot span `k`.
fn de_boor(k: usize, t: f64, p: usize, knots: &[f64], c: &[Point], w: &[f64]) -> Point {
    let mut d: Vec<(f64, f64, f64)> = (0..=p)
        .map(|j| {
            let (pt, wt) = (c[j + k - p], w[j + k - p]);
            (pt.x * wt, pt.y * wt, wt)
        })
        .collect();
    for r in 1..=p {
        for j in (r..=p).rev() {
            let lo = knots[j + k - p];
            let hi = knots[j + 1 + k - r];
            let alpha = if hi > lo { (t - lo) / (hi - lo) } else { 0.0 };
            let (a, b) = (d[j - 1], d[j]);
            d[j] = (
                (1.0 - alpha) * a.0 + alpha * b.0,
                (1.0 - alpha) * a.1 + alpha * b.1,
                (1.0 - alpha) * a.2 + alpha * b.2,
            );
        }
    }
    let (x, y, wt) = d[p];
    if wt.abs() > 1e-300 {
        Point::new(x / wt, y / wt)
    } else {
        Point::new(x, y)
    }
}

/// Parses ASCII DXF data into outlines in document millimetres.
///
/// # Errors
///
/// Returns an [`ImportError`] for binary or malformed files and unsupported
/// units.
pub fn parse(data: &[u8], bed_height_mm: f64) -> Result<Vec<Imported>, ImportError> {
    if data.starts_with(b"AutoCAD Binary DXF") {
        return Err(ImportError::new("binary DXF is not supported"));
    }
    let text = String::from_utf8_lossy(data);
    let drawing = read_drawing(&text)?;
    let scale = unit_scale(drawing.units)?;
    let mut builder = Builder {
        drawing: &drawing,
        pieces: Vec::new(),
    };
    for e in &drawing.entities {
        builder.add(e, &Affine::IDENTITY, ACI_DEFAULT, 0);
    }
    let to_doc = Affine::translate(0.0, bed_height_mm) * Affine::scale(scale, -scale);
    let mut colors: Vec<i32> = Vec::new();
    for piece in &builder.pieces {
        if !colors.contains(&piece.aci) {
            colors.push(piece.aci);
        }
    }
    let mut out = Vec::new();
    for aci in colors {
        let subs: Vec<SubPath> = builder
            .pieces
            .iter()
            .filter(|p| p.aci == aci)
            .map(|p| {
                Path::single(p.sub.clone())
                    .transformed(&to_doc)
                    .subpaths
                    .remove(0)
            })
            .collect();
        for sub in join(&subs, JOIN_TOLERANCE_MM) {
            let path = Path::single(sub);
            if path.is_valid() {
                out.push(Imported {
                    path,
                    color: Some(aci_rgb(aci)),
                });
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DXF document from (code, value) pairs inside an ENTITIES section.
    fn doc(header: &str, entities: &[&str]) -> String {
        let mut text = String::from("0\nSECTION\n2\nHEADER\n");
        text.push_str(header);
        text.push_str("0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");
        for e in entities {
            text.push_str(e);
        }
        text.push_str("0\nENDSEC\n0\nEOF\n");
        text
    }

    fn line(x1: f64, y1: f64, x2: f64, y2: f64) -> String {
        format!("0\nLINE\n8\n0\n10\n{x1}\n20\n{y1}\n11\n{x2}\n21\n{y2}\n")
    }

    #[test]
    fn lines_join_into_a_closed_outline_and_flip_onto_the_bed() {
        let entities = [
            line(0.0, 0.0, 10.0, 0.0),
            line(10.0, 0.0, 10.0, 5.0),
            line(10.0, 5.0, 0.0, 5.0),
            line(0.0, 5.0, 0.0, 0.0),
        ];
        let refs: Vec<&str> = entities.iter().map(String::as_str).collect();
        let shapes = parse(doc("", &refs).as_bytes(), 100.0).unwrap();
        assert_eq!(shapes.len(), 1);
        assert!(shapes[0].path.is_closed());
        let b = shapes[0].path.bounds().unwrap();
        assert_eq!(
            (b.min, b.max),
            (Point::new(0.0, 95.0), Point::new(10.0, 100.0))
        );
        assert_eq!(shapes[0].color, Some([0, 0, 0]));
    }

    #[test]
    fn units_circles_arcs_and_colors() {
        let header = "9\n$INSUNITS\n70\n1\n";
        let circle = "0\nCIRCLE\n8\n0\n62\n1\n10\n1\n20\n1\n40\n0.5\n";
        let arc = "0\nARC\n8\n0\n62\n5\n10\n0\n20\n0\n40\n1\n50\n0\n51\n90\n";
        let shapes = parse(doc(header, &[circle, arc]).as_bytes(), 0.0).unwrap();
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].color, Some([255, 0, 0]));
        let b = shapes[0].path.bounds().unwrap();
        assert!((b.width() - 25.4).abs() < 1e-9, "{b:?}");
        assert!(shapes[0].path.is_closed());
        let a = shapes[1].path.bounds().unwrap();
        assert!(a.max.approx_eq(Point::new(25.4, 0.0), 1e-9), "{a:?}");
        assert!(a.min.approx_eq(Point::new(0.0, -25.4), 1e-9), "{a:?}");
        let bad = doc("9\n$INSUNITS\n70\n3\n", &[circle]);
        assert!(parse(bad.as_bytes(), 0.0).unwrap_err().0.contains("units"));
    }

    #[test]
    fn bulged_polylines_ellipses_and_splines() {
        // A slot: two straight sides and two half-circle ends.
        let lw = "0\nLWPOLYLINE\n8\n0\n90\n4\n70\n1\n10\n0\n20\n0\n10\n10\n20\n0\n42\n1\n10\n10\n20\n4\n10\n0\n20\n4\n42\n1\n";
        let ellipse = "0\nELLIPSE\n8\n0\n10\n50\n20\n0\n11\n10\n21\n0\n40\n0.5\n";
        let spline = "0\nSPLINE\n8\n0\n71\n1\n72\n4\n73\n2\n40\n0\n40\n0\n40\n1\n40\n1\n10\n0\n20\n20\n10\n10\n20\n20\n";
        let shapes = parse(doc("", &[lw, ellipse, spline]).as_bytes(), 0.0).unwrap();
        assert_eq!(shapes.len(), 3);
        let slot = shapes[0].path.bounds().unwrap();
        assert!(slot.min.approx_eq(Point::new(-2.0, -4.0), 1e-9), "{slot:?}");
        assert!(slot.max.approx_eq(Point::new(12.0, 0.0), 1e-9), "{slot:?}");
        let e = shapes[1].path.bounds().unwrap();
        assert!((e.width() - 20.0).abs() < 1e-9 && (e.height() - 10.0).abs() < 1e-9);
        let s = &shapes[2].path.subpaths[0];
        assert_eq!(s.segments.len(), 16);
        assert_eq!(s.segments.last().unwrap().end(), Point::new(10.0, -20.0));
    }

    #[test]
    fn blocks_are_inserted_with_their_transform_and_layer_colors() {
        let text = "0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n0\nLAYER\n2\nCUT\n62\n-3\n0\nENDTAB\n0\nENDSEC\n\
0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB\n10\n1\n20\n0\n0\nLINE\n8\nCUT\n62\n0\n10\n1\n20\n0\n11\n2\n21\n0\n0\nENDBLK\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n0\nINSERT\n8\nCUT\n2\nB\n10\n5\n20\n5\n41\n2\n50\n90\n0\nENDSEC\n0\nEOF\n";
        let shapes = parse(text.as_bytes(), 0.0).unwrap();
        assert_eq!(shapes.len(), 1);
        assert_eq!(
            shapes[0].color,
            Some([0, 255, 0]),
            "by block → insert's layer"
        );
        let b = shapes[0].path.bounds().unwrap();
        assert!(b.min.approx_eq(Point::new(5.0, -7.0), 1e-9), "{b:?}");
        assert!(b.max.approx_eq(Point::new(5.0, -5.0), 1e-9), "{b:?}");
    }

    #[test]
    fn malformed_and_binary_files_fail() {
        assert!(parse(b"AutoCAD Binary DXF\r\n", 0.0).is_err());
        assert!(parse(b"zero\nSECTION\n", 0.0).is_err());
        assert!(parse(doc("", &[]).as_bytes(), 0.0).unwrap().is_empty());
    }

    #[test]
    fn aci_colors() {
        assert_eq!(aci_rgb(1), [255, 0, 0]);
        assert_eq!(aci_rgb(10), [255, 0, 0]);
        assert_eq!(aci_rgb(255), [255, 255, 255]);
        assert_eq!(aci_rgb(7), [0, 0, 0]);
    }
}
