//! Palette colors as egui colors.

use egui::Color32;
use lasercraft_project::{Color, LAYER_COUNT, layer::palette};

/// The egui color of a palette entry.
#[must_use]
pub fn layer_color(color: Color) -> Color32 {
    let [r, g, b] = palette()
        .get(usize::from(color))
        .copied()
        .unwrap_or([128, 128, 128]);
    Color32::from_rgb(r, g, b)
}

/// Every palette color with its index.
pub fn all() -> impl Iterator<Item = (Color, Color32)> {
    (0..LAYER_COUNT).filter_map(|i| {
        let c = Color::try_from(i).ok()?;
        Some((c, layer_color(c)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_palette_entry_maps() {
        assert_eq!(all().count(), LAYER_COUNT);
        assert_eq!(layer_color(0), Color32::from_rgb(0, 0, 0));
        assert_eq!(layer_color(200), Color32::from_rgb(128, 128, 128));
    }
}
