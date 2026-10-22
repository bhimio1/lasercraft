//! LaserCraft's icons (`docs/spec/style/icons.md`): our own SVG drawings
//! in `icons/`, read once through our SVG importer and painted as strokes
//! with round caps and joins in the colors the caller passes.

use std::collections::HashMap;
use std::sync::OnceLock;

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke};
use lasercraft_import::svg::{Styled, parse_styled};

/// Side of the drawing grid, in SVG user units.
const GRID: f32 = 24.0;
/// Stroke weight on the grid.
const STROKE: f32 = 1.5;
/// Flattening tolerance on the grid.
const TOLERANCE: f64 = 0.05;

/// Every icon: id and its embedded source.
const SOURCES: [(&str, &[u8]); 6] = [
    ("select", include_bytes!("../icons/select.svg")),
    ("line", include_bytes!("../icons/line.svg")),
    ("rect", include_bytes!("../icons/rect.svg")),
    ("ellipse", include_bytes!("../icons/ellipse.svg")),
    ("edit-nodes", include_bytes!("../icons/edit-nodes.svg")),
    ("text-tool", include_bytes!("../icons/text-tool.svg")),
];

fn parsed() -> &'static HashMap<&'static str, Vec<Styled>> {
    static CACHE: OnceLock<HashMap<&'static str, Vec<Styled>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        SOURCES
            .iter()
            .map(|(id, data)| (*id, parse_styled(data).unwrap_or_default()))
            .collect()
    })
}

/// Whether an icon with this id exists.
#[cfg(test)]
#[must_use]
pub fn exists(id: &str) -> bool {
    SOURCES.iter().any(|(i, _)| *i == id)
}

/// Paints icon `id` filling the square `rect`: base parts in `base`, the
/// part with the id `accent` in `accent`. Unknown ids paint nothing.
pub fn paint(painter: &Painter, id: &str, rect: Rect, base: Color32, accent: Color32) {
    let Some(parts) = parsed().get(id) else {
        return;
    };
    let scale = rect.width().min(rect.height()) / GRID;
    let origin = rect.center() - egui::vec2(GRID, GRID) * scale / 2.0;
    let to =
        |p: lasercraft_geom::Point| Pos2::new(p.x as f32, p.y as f32) * scale + origin.to_vec2();
    let width = STROKE * scale;
    for part in parts {
        let color = if part.id == "accent" { accent } else { base };
        for (points, closed) in part.path.flatten(TOLERANCE) {
            let mut points: Vec<Pos2> = points.into_iter().map(to).collect();
            if part.filled {
                painter.add(Shape::convex_polygon(points.clone(), color, Stroke::NONE));
            }
            if !part.stroked {
                continue;
            }
            if closed && let Some(first) = points.first().copied() {
                points.push(first);
            }
            let stroke = Stroke::new(width, color);
            match part.dashes.as_deref() {
                Some([dash, gap, ..]) => painter.extend(Shape::dashed_line(
                    &points,
                    stroke,
                    *dash as f32 * scale,
                    *gap as f32 * scale,
                )),
                _ => {
                    painter.add(Shape::line(points.clone(), stroke));
                    // Round joins and caps.
                    for p in &points {
                        painter.circle_filled(*p, width / 2.0, color);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Elements the icon subset allows (`docs/spec/ui-shell.md`, "Icons").
    const ALLOWED: [&str; 9] = [
        "svg", "g", "path", "line", "polyline", "polygon", "rect", "circle", "ellipse",
    ];

    fn text(data: &[u8]) -> &str {
        std::str::from_utf8(data).unwrap()
    }

    #[test]
    fn every_source_stays_inside_the_subset() {
        for (id, data) in SOURCES {
            let svg = text(data);
            assert!(
                svg.contains(r#"viewBox="0 0 24 24""#),
                "{id}: square 24-unit view box"
            );
            for tag in svg.split('<').skip(1) {
                if tag.starts_with('/') || tag.starts_with("!--") || tag.starts_with('?') {
                    continue;
                }
                let name: String = tag
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .collect();
                assert!(
                    ALLOWED.contains(&name.as_str()),
                    "{id}: <{name}> is not allowed"
                );
            }
            for banned in [
                "url(",
                "transform=",
                "<text",
                "<image",
                "filter",
                "mask",
                "opacity",
            ] {
                assert!(!svg.contains(banned), "{id}: uses {banned}");
            }
            assert!(
                svg.matches(r#"id="accent""#).count() <= 1,
                "{id}: one accent at most"
            );
        }
    }

    #[test]
    fn every_icon_parses_and_keeps_its_safe_margin() {
        for (id, _) in SOURCES {
            let parts = &parsed()[id];
            assert!(!parts.is_empty(), "{id}: nothing drawn");
            for part in parts {
                let b = part.path.bounds().unwrap();
                let margin = f64::from(STROKE) / 2.0;
                assert!(
                    b.min.x >= margin && b.min.y >= margin,
                    "{id}: {b:?} crosses the margin"
                );
                assert!(
                    b.max.x <= 24.0 - margin && b.max.y <= 24.0 - margin,
                    "{id}: {b:?} crosses the margin"
                );
            }
            let fills = parts.iter().filter(|p| p.filled).count();
            assert!(fills <= 1, "{id}: fills are for small marks only");
        }
    }

    #[test]
    fn every_icon_has_a_provenance_line() {
        let provenance = include_str!("../icons/PROVENANCE.md");
        for (id, _) in SOURCES {
            assert!(provenance.contains(&format!("| `{id}` |")), "{id}");
        }
    }

    #[test]
    fn painting_emits_shapes_in_the_given_colors() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(24.0, 24.0));
            paint(ui.painter(), "line", rect, Color32::BLACK, Color32::BLUE);
            assert!(!exists("nope"));
        });
        output.textures_delta.clear();
        let shapes: Vec<Shape> = output.shapes.into_iter().map(|c| c.shape).collect();
        let colored = |c: Color32| {
            shapes.iter().any(|s| match s {
                Shape::Path(p) => p.stroke.color == egui::epaint::ColorMode::Solid(c),
                _ => false,
            })
        };
        assert!(colored(Color32::BLACK) && colored(Color32::BLUE));
    }
}
