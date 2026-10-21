//! Zoom and pan: the mapping between document millimetres and screen pixels.

use lasercraft_geom::{Point, Rect};

/// Fewest pixels per millimetre (most zoomed out).
pub const MIN_SCALE: f64 = 0.05;
/// Most pixels per millimetre (most zoomed in).
pub const MAX_SCALE: f64 = 200.0;
/// Zoom factor of one zoom-in step.
pub const ZOOM_STEP: f64 = 1.25;

/// `screen = origin + doc * scale`, with y down in both.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    scale: f64,
    origin: Point,
}

impl View {
    /// A view with `scale` pixels per millimetre and document (0, 0) at
    /// screen point `origin`.
    #[must_use]
    pub fn new(scale: f64, origin: Point) -> Self {
        Self {
            scale: scale.clamp(MIN_SCALE, MAX_SCALE),
            origin,
        }
    }

    /// The view that shows `content` centered in `screen` with `margin` pixels
    /// on each side.
    #[must_use]
    pub fn fit(screen: &Rect, content: &Rect, margin: f64) -> Self {
        let w = (screen.width() - 2.0 * margin).max(1.0);
        let h = (screen.height() - 2.0 * margin).max(1.0);
        let scale = (w / content.width().max(f64::EPSILON))
            .min(h / content.height().max(f64::EPSILON))
            .clamp(MIN_SCALE, MAX_SCALE);
        let origin = screen.center() - content.center() * scale;
        Self { scale, origin }
    }

    /// Pixels per millimetre.
    #[must_use]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Document point to screen point.
    #[must_use]
    pub fn to_screen(&self, p: Point) -> Point {
        self.origin + p * self.scale
    }

    /// Screen point to document point.
    #[must_use]
    pub fn to_doc(&self, s: Point) -> Point {
        (s - self.origin) * (1.0 / self.scale)
    }

    /// Zooms by `factor`, keeping the document point under `screen` fixed.
    pub fn zoom_about(&mut self, screen: Point, factor: f64) {
        if !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let fixed = self.to_doc(screen);
        self.scale = (self.scale * factor).clamp(MIN_SCALE, MAX_SCALE);
        self.origin = screen - fixed * self.scale;
    }

    /// Moves the view by `delta` pixels.
    pub fn pan(&mut self, delta: Point) {
        self.origin = self.origin + delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Rect {
        Rect::from_corners(Point::new(0.0, 0.0), Point::new(1000.0, 600.0))
    }

    #[test]
    fn screen_and_doc_round_trip() {
        let v = View::new(2.5, Point::new(100.0, 50.0));
        let p = Point::new(12.0, -3.5);
        assert!(v.to_doc(v.to_screen(p)).approx_eq(p, 1e-12));
        assert_eq!(v.to_screen(Point::ZERO), Point::new(100.0, 50.0));
    }

    #[test]
    fn fit_centers_content_with_margin() {
        let bed = Rect::from_corners(Point::ZERO, Point::new(400.0, 400.0));
        let v = View::fit(&screen(), &bed, 50.0);
        assert_eq!(v.scale(), 1.25);
        assert!(
            v.to_screen(bed.center())
                .approx_eq(Point::new(500.0, 300.0), 1e-9)
        );
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let mut v = View::new(1.0, Point::ZERO);
        let cursor = Point::new(300.0, 200.0);
        let under = v.to_doc(cursor);
        v.zoom_about(cursor, ZOOM_STEP);
        assert!(v.to_doc(cursor).approx_eq(under, 1e-9));
        assert_eq!(v.scale(), 1.25);
    }

    #[test]
    fn zoom_is_clamped_and_ignores_bad_factors() {
        let mut v = View::new(1.0, Point::ZERO);
        v.zoom_about(Point::ZERO, 1e9);
        assert_eq!(v.scale(), MAX_SCALE);
        v.zoom_about(Point::ZERO, 1e-12);
        assert_eq!(v.scale(), MIN_SCALE);
        v.zoom_about(Point::ZERO, f64::NAN);
        assert_eq!(v.scale(), MIN_SCALE);
    }

    #[test]
    fn pan_moves_the_origin() {
        let mut v = View::new(2.0, Point::ZERO);
        v.pan(Point::new(10.0, -4.0));
        assert_eq!(v.to_screen(Point::ZERO), Point::new(10.0, -4.0));
    }
}
