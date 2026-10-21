//! The project's layers: settings for all 30 colors in output order
//! (`docs/spec/layers.md`).

use std::collections::HashSet;

use crate::layer::{Color, LAYER_COUNT, Layer};
use crate::{Project, ProjectError};

impl Project {
    /// All 30 layers in output order.
    #[must_use]
    pub fn layers(&self) -> &[Layer] {
        &self.layers.0
    }

    /// The layer for a color.
    ///
    /// # Errors
    ///
    /// Fails for a color outside 0–29.
    pub fn layer(&self, color: Color) -> Result<&Layer, ProjectError> {
        self.layers
            .0
            .iter()
            .find(|l| l.color == color)
            .ok_or_else(|| ProjectError::invalid("color", format!("{color} is not 0–29")))
    }

    fn layer_mut(&mut self, color: Color) -> Result<&mut Layer, ProjectError> {
        self.layers
            .0
            .iter_mut()
            .find(|l| l.color == color)
            .ok_or_else(|| ProjectError::invalid("color", format!("{color} is not 0–29")))
    }

    /// Changes one setting of a layer.
    ///
    /// # Errors
    ///
    /// Fails for a bad color, setting or value; the layer is then unchanged.
    pub fn set_layer_setting(
        &mut self,
        color: Color,
        key: &str,
        value: &str,
    ) -> Result<(), ProjectError> {
        self.layer_mut(color)?.set(key, value)
    }

    /// Replaces a layer's settings, keeping its color and place in the order.
    ///
    /// # Errors
    ///
    /// Fails for a bad color or invalid settings.
    pub fn replace_layer(&mut self, color: Color, mut settings: Layer) -> Result<(), ProjectError> {
        settings.color = color;
        settings.validate()?;
        *self.layer_mut(color)? = settings;
        Ok(())
    }

    /// Output order as colors.
    #[must_use]
    pub fn layer_order(&self) -> Vec<Color> {
        self.layers.0.iter().map(|l| l.color).collect()
    }

    /// Sets the output order.
    ///
    /// # Errors
    ///
    /// Fails unless `order` lists all 30 colors exactly once.
    pub fn set_layer_order(&mut self, order: &[Color]) -> Result<(), ProjectError> {
        let unique: HashSet<Color> = order.iter().copied().collect();
        if order.len() != LAYER_COUNT || unique.len() != LAYER_COUNT {
            return Err(ProjectError::invalid(
                "layer order",
                "must list all 30 colors exactly once",
            ));
        }
        let mut layers = Vec::with_capacity(LAYER_COUNT);
        for color in order {
            layers.push(self.layer(*color)?.clone());
        }
        self.layers.0 = layers;
        Ok(())
    }

    /// Colors that have shapes, in output order: what the layer list shows.
    #[must_use]
    pub fn used_layers(&self) -> Vec<Color> {
        let used: HashSet<Color> = self.shapes.iter().map(crate::Shape::layer).collect();
        self.layers
            .0
            .iter()
            .map(|l| l.color)
            .filter(|c| used.contains(c))
            .collect()
    }

    /// Ids of the shapes on a layer, in drawing order.
    #[must_use]
    pub fn shapes_on(&self, color: Color) -> Vec<u32> {
        self.shapes
            .iter()
            .filter(|s| s.layer() == color)
            .map(crate::Shape::id)
            .collect()
    }

    pub(crate) fn validate_layers(&self) -> Result<(), ProjectError> {
        for layer in &self.layers.0 {
            layer.validate()?;
        }
        let order = self.layer_order();
        let unique: HashSet<Color> = order.iter().copied().collect();
        if order.len() != LAYER_COUNT || unique.len() != LAYER_COUNT {
            return Err(ProjectError::invalid(
                "layers",
                "must hold all 30 colors exactly once",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use lasercraft_geom::Affine;

    use super::*;
    use crate::Geometry;

    fn rect() -> Geometry {
        Geometry::Rect {
            width: 1.0,
            height: 1.0,
        }
    }

    #[test]
    fn used_layers_follow_output_order() {
        let mut p = Project::default();
        p.add_shape_on(rect(), Affine::IDENTITY, 5).unwrap();
        p.add_shape_on(rect(), Affine::IDENTITY, 2).unwrap();
        assert_eq!(p.used_layers(), [2, 5]);
        let mut order = p.layer_order();
        order.swap(2, 5);
        p.set_layer_order(&order).unwrap();
        assert_eq!(p.used_layers(), [5, 2]);
        assert_eq!(p.shapes_on(2), [2]);
    }

    #[test]
    fn order_must_be_a_permutation_of_all_colors() {
        let mut p = Project::default();
        assert!(p.set_layer_order(&[0, 1]).is_err());
        let mut dup = p.layer_order();
        dup[1] = 0;
        assert!(p.set_layer_order(&dup).is_err());
        let mut bad = p.layer_order();
        bad[0] = 99;
        assert!(p.set_layer_order(&bad).is_err());
    }

    #[test]
    fn replace_keeps_color_and_validates() {
        let mut p = Project::default();
        let mut settings = p.layer(3).unwrap().clone();
        settings.color = 9;
        settings.name = "Engrave".to_owned();
        p.replace_layer(3, settings.clone()).unwrap();
        assert_eq!(p.layer(3).unwrap().name, "Engrave");
        assert_eq!(p.layer(9).unwrap().name, "Layer 09");
        settings.speed = -1.0;
        assert!(p.replace_layer(3, settings).is_err());
        assert!(p.set_layer_setting(3, "speed", "0").is_err());
        assert!(p.layer(30).is_err());
    }

    #[test]
    fn shapes_reject_unknown_colors() {
        let mut p = Project::default();
        assert!(p.add_shape_on(rect(), Affine::IDENTITY, 30).is_err());
        let id = p.add_shape(rect(), Affine::IDENTITY).unwrap();
        assert!(p.set_shape_layer(id, 30).is_err());
        p.set_shape_layer(id, 29).unwrap();
        assert_eq!(p.shape(id).unwrap().layer(), 29);
    }
}
