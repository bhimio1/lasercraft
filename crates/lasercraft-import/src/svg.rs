//! SVG through `usvg` (`docs/spec/import.md`).

use lasercraft_geom::{Affine, Path, Point, Segment, SubPath};
use usvg::tiny_skia_path::PathSegment;

use crate::{ImportError, Imported};

/// Millimetres per SVG user unit (CSS pixel, 96 per inch).
pub const MM_PER_PX: f64 = 25.4 / 96.0;

/// Parses SVG data into outlines in millimetres.
///
/// # Errors
///
/// Returns an [`ImportError`] when the data is not valid SVG.
pub fn parse(data: &[u8]) -> Result<Vec<Imported>, ImportError> {
    let tree = usvg::Tree::from_data(data, &usvg::Options::default())
        .map_err(|e| ImportError::new(format!("not a valid SVG file: {e}")))?;
    let mut out = Vec::new();
    walk(tree.root(), &mut out);
    Ok(out)
}

/// One painted element of an SVG drawn for the UI (an icon), in SVG user
/// units, with what the painter needs to know about its style.
#[derive(Debug, Clone, PartialEq)]
pub struct Styled {
    /// The outline in user units (not millimetres).
    pub path: Path,
    /// The element's id, empty when it has none.
    pub id: String,
    /// The element is filled.
    pub filled: bool,
    /// The element is stroked.
    pub stroked: bool,
    /// The stroke's dash pattern in user units, if dashed.
    pub dashes: Option<Vec<f64>>,
}

/// Parses SVG data into styled outlines in user units, for icons drawn in
/// LaserCraft's own subset (`docs/spec/ui-shell.md`, "Icons").
///
/// # Errors
///
/// Returns an [`ImportError`] when the data is not valid SVG.
pub fn parse_styled(data: &[u8]) -> Result<Vec<Styled>, ImportError> {
    let tree = usvg::Tree::from_data(data, &usvg::Options::default())
        .map_err(|e| ImportError::new(format!("not a valid SVG file: {e}")))?;
    let mut out = Vec::new();
    walk_styled(tree.root(), &mut out);
    Ok(out)
}

fn walk_styled(group: &usvg::Group, out: &mut Vec<Styled>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => walk_styled(g, out),
            usvg::Node::Path(p) if p.is_visible() => {
                let t = p.abs_transform();
                let affine = Affine {
                    m: [t.sx, t.ky, t.kx, t.sy, t.tx, t.ty].map(f64::from),
                };
                let Some(path) = to_path(p.data()).map(|path| path.transformed(&affine)) else {
                    continue;
                };
                // The id may sit on the element or on a group around it.
                let id = if p.id().is_empty() {
                    group.id()
                } else {
                    p.id()
                };
                out.push(Styled {
                    path,
                    id: id.to_owned(),
                    filled: p.fill().is_some(),
                    stroked: p.stroke().is_some(),
                    dashes: p
                        .stroke()
                        .and_then(usvg::Stroke::dasharray)
                        .map(|d| d.iter().map(|v| f64::from(*v)).collect()),
                });
            }
            _ => {}
        }
    }
}

fn walk(group: &usvg::Group, out: &mut Vec<Imported>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => walk(g, out),
            usvg::Node::Path(p) => out.extend(convert(p)),
            usvg::Node::Image(_) | usvg::Node::Text(_) => {}
        }
    }
}

fn plain(paint: &usvg::Paint) -> Option<[u8; 3]> {
    match paint {
        usvg::Paint::Color(c) => Some([c.red, c.green, c.blue]),
        _ => None,
    }
}

fn convert(p: &usvg::Path) -> Option<Imported> {
    let painted = p.fill().is_some() || p.stroke().is_some();
    // usvg marks unpainted paths invisible; only `visibility` hides
    // painted ones, and those are skipped.
    if painted && !p.is_visible() {
        return None;
    }
    let t = p.abs_transform();
    let affine = Affine::scale(MM_PER_PX, MM_PER_PX)
        * Affine {
            m: [t.sx, t.ky, t.kx, t.sy, t.tx, t.ty].map(f64::from),
        };
    let path = to_path(p.data())?.transformed(&affine);
    let color = p
        .stroke()
        .and_then(|s| plain(s.paint()))
        .or_else(|| p.fill().and_then(|f| plain(f.paint())));
    path.is_valid().then_some(Imported { path, color })
}

/// Converts usvg path data, raising quadratics to cubics exactly.
fn to_path(data: &usvg::tiny_skia_path::Path) -> Option<Path> {
    let pt = |p: usvg::tiny_skia_path::Point| Point::new(f64::from(p.x), f64::from(p.y));
    let mut subpaths = Vec::new();
    let mut current: Option<SubPath> = None;
    let mut last = Point::ZERO;
    let finish = |sub: Option<SubPath>, subpaths: &mut Vec<SubPath>| {
        if let Some(sub) = sub.filter(|s| !s.segments.is_empty()) {
            subpaths.push(sub);
        }
    };
    for seg in data.segments() {
        if !matches!(seg, PathSegment::MoveTo(_) | PathSegment::Close) {
            // A segment after a close continues from that subpath's start.
            let restart = current.as_ref().is_none_or(|c| c.closed);
            if restart {
                let start = current.as_ref().map_or(last, |c| c.start);
                finish(current.take(), &mut subpaths);
                current = Some(SubPath {
                    start,
                    segments: Vec::new(),
                    closed: false,
                });
                last = start;
            }
        }
        match seg {
            PathSegment::MoveTo(p) => {
                finish(current.take(), &mut subpaths);
                last = pt(p);
                current = Some(SubPath {
                    start: last,
                    segments: Vec::new(),
                    closed: false,
                });
            }
            PathSegment::LineTo(p) => {
                last = pt(p);
                current.as_mut()?.segments.push(Segment::Line(last));
            }
            PathSegment::QuadTo(c, p) => {
                let (c, p) = (pt(c), pt(p));
                let c1 = last + (c - last) * (2.0 / 3.0);
                let c2 = p + (c - p) * (2.0 / 3.0);
                current.as_mut()?.segments.push(Segment::Cubic(c1, c2, p));
                last = p;
            }
            PathSegment::CubicTo(a, b, p) => {
                last = pt(p);
                current
                    .as_mut()?
                    .segments
                    .push(Segment::Cubic(pt(a), pt(b), last));
            }
            PathSegment::Close => {
                if let Some(c) = current.as_mut() {
                    c.closed = true;
                    last = c.start;
                }
            }
        }
    }
    finish(current, &mut subpaths);
    (!subpaths.is_empty()).then_some(Path { subpaths })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_parsing_keeps_ids_fills_and_dashes_in_user_units() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
            <rect x="3" y="6" width="18" height="12" fill="none" stroke="#000"/>
            <circle id="accent" cx="12" cy="12" r="2" fill="#000"/>
            <line x1="2" y1="2" x2="22" y2="2" stroke="#000" stroke-dasharray="2 2"/>
        </svg>"##;
        let parts = parse_styled(svg).unwrap();
        assert_eq!(parts.len(), 3);
        let b = parts[0].path.bounds().unwrap();
        assert!((b.min.x - 3.0).abs() < 1e-6 && (b.max.y - 18.0).abs() < 1e-6);
        assert!(parts[0].stroked && !parts[0].filled && parts[0].id.is_empty());
        assert!(parts[1].filled && !parts[1].stroked && parts[1].id == "accent");
        assert_eq!(parts[2].dashes.as_deref(), Some(&[2.0, 2.0][..]));
    }

    fn svg(body: &str) -> Vec<Imported> {
        let doc = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="50mm" viewBox="0 0 100 50">{body}</svg>"#
        );
        parse(doc.as_bytes()).unwrap()
    }

    #[test]
    fn units_follow_the_view_box_and_colors_prefer_the_stroke() {
        let shapes = svg(
            r##"<rect x="10" y="5" width="20" height="10" fill="red" stroke="#0000ff"/>
               <circle cx="50" cy="25" r="10" fill="#00ff00"/>
               <path d="M60 10 Q70 0 80 10" stroke="black" fill="none"/>"##,
        );
        assert_eq!(shapes.len(), 3);
        let b = shapes[0].path.bounds().unwrap();
        assert!(b.min.approx_eq(Point::new(10.0, 5.0), 1e-4), "{b:?}");
        assert!(b.max.approx_eq(Point::new(30.0, 15.0), 1e-4), "{b:?}");
        assert_eq!(shapes[0].color, Some([0, 0, 255]));
        assert!(shapes[0].path.is_closed());
        assert_eq!(shapes[1].color, Some([0, 255, 0]));
        let c = shapes[1].path.bounds().unwrap();
        assert!((c.width() - 20.0).abs() < 1e-3, "{c:?}");
        // The quadratic became one exact cubic.
        let q = &shapes[2].path.subpaths[0];
        assert!(!q.closed);
        assert!(matches!(q.segments[..], [Segment::Cubic(..)]));
        assert!((q.bounds().min.y - 5.0).abs() < 1e-4);
    }

    #[test]
    fn transforms_holes_and_hidden_elements() {
        let shapes = svg(r#"<g transform="translate(10 0) scale(2)">
                 <path d="M0 0 H10 V10 H0 Z M2 2 V8 H8 V2 Z" fill="black"/>
               </g>
               <rect width="5" height="5" fill="red" visibility="hidden"/>
               <rect width="5" height="5" fill="none" stroke="none"/>
               <text x="0" y="0">ignored</text>"#);
        assert_eq!(shapes.len(), 2, "hidden element and text skipped");
        assert_eq!(shapes[0].path.subpaths.len(), 2, "hole kept");
        let b = shapes[0].path.bounds().unwrap();
        assert!(b.max.approx_eq(Point::new(30.0, 20.0), 1e-4), "{b:?}");
        assert_eq!(shapes[1].color, None);
    }

    #[test]
    fn plain_user_units_are_css_pixels() {
        let doc = r#"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96"><rect width="96" height="48"/></svg>"#;
        let shapes = parse(doc.as_bytes()).unwrap();
        let b = shapes[0].path.bounds().unwrap();
        assert!((b.width() - 25.4).abs() < 1e-4);
        assert!(parse(b"<svg").is_err());
    }
}
