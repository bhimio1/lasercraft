//! LaserCraft's calibration sheet (own design, `docs/spec/camera.md`,
//! "Calibration"): 9 × 7 black dots on white with three larger dots that
//! mark the center and the x and y directions.

use std::fmt::Write as _;

use lasercraft_geom::Point;

/// Dot columns.
pub const COLUMNS: i32 = 9;
/// Dot rows.
pub const ROWS: i32 = 7;
/// Distance between dot centers at 100 % print size, mm.
pub const PITCH: f64 = 20.0;
/// Dot diameter, mm.
pub const DOT: f64 = 8.0;
/// Marker dot diameter, mm.
pub const BIG: f64 = 12.0;
/// The marker dots: the center, then the x and y directions.
pub const MARKERS: [(i32, i32); 3] = [(4, 3), (5, 3), (4, 4)];
/// White border around the outer dots, mm.
pub const MARGIN: f64 = 20.0;

/// A dot's center on the sheet, mm from the first dot.
#[must_use]
pub fn point(i: i32, j: i32) -> Point {
    Point::new(f64::from(i) * PITCH, f64::from(j) * PITCH)
}

/// Whether grid position `(i, j)` is on the sheet.
#[must_use]
pub fn on_sheet(i: i32, j: i32) -> bool {
    (0..COLUMNS).contains(&i) && (0..ROWS).contains(&j)
}

/// Whether `p` (sheet mm) is inside a dot; `None` off the sheet.
#[must_use]
pub fn dark_at(p: Point) -> Option<bool> {
    let (w, h) = (f64::from(COLUMNS - 1) * PITCH, f64::from(ROWS - 1) * PITCH);
    if p.x < -MARGIN || p.y < -MARGIN || p.x > w + MARGIN || p.y > h + MARGIN {
        return None;
    }
    let i = (p.x / PITCH).round() as i32;
    let j = (p.y / PITCH).round() as i32;
    if !on_sheet(i, j) {
        return Some(false);
    }
    let r = if MARKERS.contains(&(i, j)) { BIG } else { DOT } / 2.0;
    Some(p.distance(point(i, j)) <= r)
}

/// The sheet as SVG at 100 % size, millimetres.
#[must_use]
pub fn svg() -> String {
    let w = f64::from(COLUMNS - 1) * PITCH + 2.0 * MARGIN;
    let h = f64::from(ROWS - 1) * PITCH + 2.0 * MARGIN + 10.0;
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}mm\" height=\"{h}mm\" viewBox=\"0 0 {w} {h}\">\n<rect width=\"{w}\" height=\"{h}\" fill=\"#fff\"/>\n"
    );
    for j in 0..ROWS {
        for i in 0..COLUMNS {
            let c = point(i, j);
            let r = if MARKERS.contains(&(i, j)) { BIG } else { DOT } / 2.0;
            let _ = writeln!(
                out,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{r}\" fill=\"#000\"/>",
                c.x + MARGIN,
                c.y + MARGIN
            );
        }
    }
    let _ = writeln!(
        out,
        "<text x=\"{MARGIN}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"4\" fill=\"#000\">LaserCraft calibration sheet. Print at any size, keep it flat.</text>",
        h - 8.0
    );
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_are_larger_and_the_sheet_has_its_dots() {
        assert_eq!(dark_at(point(0, 0)), Some(true));
        assert_eq!(dark_at(point(0, 0) + Point::new(4.5, 0.0)), Some(false));
        assert_eq!(dark_at(point(4, 3) + Point::new(5.5, 0.0)), Some(true));
        assert_eq!(dark_at(Point::new(-30.0, 0.0)), None);
        let svg = svg();
        assert_eq!(svg.matches("<circle").count(), 63);
        assert!(svg.contains("r=\"6\""));
    }
}
