//! Text as outlines from a font file (`docs/spec/text.md`).

use lasercraft_geom::{Path, Point, Segment, SubPath};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::ImportError;

/// Collects glyph contours in font units (y up), offset by the pen.
struct Collector {
    subpaths: Vec<SubPath>,
    current: Option<SubPath>,
    last: Point,
    /// Pen position in font units, y down (lines grow downwards).
    origin: Point,
}

impl Collector {
    fn map(&self, x: f32, y: f32) -> Point {
        Point::new(self.origin.x + f64::from(x), self.origin.y - f64::from(y))
    }

    fn finish(&mut self) {
        if let Some(sub) = self.current.take().filter(|s| !s.segments.is_empty()) {
            self.subpaths.push(sub);
        }
    }

    fn push(&mut self, seg: Segment) {
        self.last = seg.end();
        if let Some(c) = self.current.as_mut() {
            c.segments.push(seg);
        }
    }
}

impl OutlineBuilder for Collector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        self.last = self.map(x, y);
        self.current = Some(SubPath {
            start: self.last,
            segments: Vec::new(),
            closed: false,
        });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.map(x, y);
        self.push(Segment::Line(p));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.map(x1, y1), self.map(x, y));
        let c1 = self.last + (c - self.last) * (2.0 / 3.0);
        let c2 = p + (c - p) * (2.0 / 3.0);
        self.push(Segment::Cubic(c1, c2, p));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, b, p) = (self.map(x1, y1), self.map(x2, y2), self.map(x, y));
        self.push(Segment::Cubic(a, b, p));
    }

    fn close(&mut self) {
        if let Some(c) = self.current.as_mut() {
            c.closed = true;
        }
        self.finish();
    }
}

/// Font units per capital-letter height.
fn cap_height(face: &Face<'_>) -> f64 {
    if let Some(h) = face.capital_height().filter(|h| *h > 0) {
        return f64::from(h);
    }
    if let Some(r) = face
        .glyph_index('H')
        .and_then(|g| face.glyph_bounding_box(g))
        .filter(|r| r.y_max > r.y_min)
    {
        return f64::from(r.y_max) - f64::from(r.y_min);
    }
    0.7 * f64::from(face.units_per_em())
}

fn kerning(face: &Face<'_>, left: GlyphId, right: GlyphId) -> f64 {
    face.tables()
        .kern
        .and_then(|kern| {
            kern.subtables
                .into_iter()
                .filter(|s| s.horizontal && !s.variable && !s.has_cross_stream)
                .find_map(|s| s.glyphs_kerning(left, right))
        })
        .map_or(0.0, f64::from)
}

/// Outlines `text` with the face at `index` in `font_data`, scaled so
/// capital letters are `height` mm tall. The path's origin is the first
/// baseline's start; y grows downwards.
///
/// # Errors
///
/// Fails for unreadable font data, a bad height, or text with no outline.
pub fn outline(font_data: &[u8], index: u32, text: &str, height: f64) -> Result<Path, ImportError> {
    if !(height.is_finite() && (0.1..=1000.0).contains(&height)) {
        return Err(ImportError::new(format!(
            "text height must be 0.1–1000 mm, got {height}"
        )));
    }
    let face = Face::parse(font_data, index)
        .map_err(|e| ImportError::new(format!("not a usable font: {e}")))?;
    let line_height =
        f64::from(face.ascender()) - f64::from(face.descender()) + f64::from(face.line_gap());
    let mut c = Collector {
        subpaths: Vec::new(),
        current: None,
        last: Point::ZERO,
        origin: Point::ZERO,
    };
    let mut previous: Option<GlyphId> = None;
    for ch in text.chars() {
        match ch {
            '\n' => {
                c.origin = Point::new(0.0, c.origin.y + line_height);
                previous = None;
                continue;
            }
            '\r' => continue,
            _ => {}
        }
        let glyph = face.glyph_index(ch).unwrap_or(GlyphId(0));
        if let Some(prev) = previous {
            c.origin.x += kerning(&face, prev, glyph);
        }
        face.outline_glyph(glyph, &mut c);
        c.finish();
        c.origin.x += f64::from(face.glyph_hor_advance(glyph).unwrap_or(0));
        previous = Some(glyph);
    }
    let scale = height / cap_height(&face);
    let path = Path {
        subpaths: c.subpaths,
    }
    .transformed(&lasercraft_geom::Affine::scale(scale, scale));
    if path.subpaths.is_empty() || !path.is_valid() {
        return Err(ImportError::new("nothing to draw"));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> Vec<u8> {
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/fixtures/lctest.ttf"
        ))
        .unwrap()
    }

    #[test]
    fn capitals_are_the_requested_height_and_counters_are_holes() {
        let path = outline(&font(), 0, "HOLA", 10.0).unwrap();
        let b = path.bounds().unwrap();
        // Cap height 700 units → 10 mm; baseline at y = 0.
        assert!(
            (b.min.y + 10.0).abs() < 1e-9 && b.max.y.abs() < 1e-9,
            "{b:?}"
        );
        // H, O (2), L, A (2).
        assert_eq!(path.subpaths.len(), 6);
        assert!(path.is_closed());
        // O uses quadratics raised to cubics.
        assert!(
            path.subpaths[1]
                .segments
                .iter()
                .any(|s| matches!(s, Segment::Cubic(..)))
        );
        // Advances: H 600, O 600, L 500 → A starts at 1700 units.
        let a = path.subpaths[4].bounds();
        assert!((a.min.x - (1700.0 + 50.0) / 70.0).abs() < 1e-9, "{a:?}");
    }

    #[test]
    fn kerning_lines_and_determinism() {
        let lt = outline(&font(), 0, "LT", 7.0).unwrap();
        let t = lt.subpaths[1].bounds();
        // T starts at advance 500 minus the 100-unit kern pair, plus its 50
        // side bearing: 450 units at 0.01 mm per unit.
        assert!((t.min.x - 4.5).abs() < 1e-9, "{t:?}");
        let two = outline(&font(), 0, "I\nI", 7.0).unwrap();
        let second = two.subpaths[1].bounds();
        assert!((second.max.y - 10.0).abs() < 1e-9, "line height 1000 units");
        assert_eq!(
            outline(&font(), 0, "CAT", 5.0),
            outline(&font(), 0, "CAT", 5.0)
        );
        // Missing characters draw the missing-glyph box.
        assert_eq!(outline(&font(), 0, "z", 5.0).unwrap().subpaths.len(), 2);
    }

    #[test]
    fn bad_input_fails() {
        assert!(outline(&font(), 0, "   ", 5.0).is_err());
        assert!(outline(&font(), 0, "", 5.0).is_err());
        assert!(outline(&font(), 0, "H", 0.0).is_err());
        assert!(outline(b"not a font", 0, "H", 5.0).is_err());
    }
}
