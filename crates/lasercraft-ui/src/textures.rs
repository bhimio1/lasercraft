//! GPU textures of the project's images, for the canvas
//! (`docs/spec/images.md`, "Canvas"): the adjusted grayscale bitmap, at most
//! [`MAX_SIDE`] pixels per side.

use std::collections::HashMap;
use std::sync::Arc;

use lasercraft_project::{Image, Project};

/// Largest texture side; bigger bitmaps are averaged down for display.
pub const MAX_SIDE: u32 = 2048;

/// Identifies what a texture shows: the bitmap (by identity) and the
/// adjustments.
type Key = (usize, [u64; 3]);

fn key(image: &Image) -> Key {
    let a = image.adjust;
    (
        Arc::as_ptr(&image.bitmap) as usize,
        [
            a.brightness.to_bits(),
            a.contrast.to_bits(),
            a.gamma.to_bits(),
        ],
    )
}

/// One texture per image shape, rebuilt when its bitmap or adjustments
/// change.
#[derive(Default)]
pub struct Textures {
    by_shape: HashMap<u32, (Key, egui::TextureHandle)>,
}

impl Textures {
    /// Makes textures for new or changed images and drops unused ones.
    pub fn update(&mut self, ctx: &egui::Context, project: &Project) {
        let mut live = Vec::new();
        for shape in project.shapes() {
            let Some(image) = shape.geometry().image() else {
                continue;
            };
            live.push(shape.id());
            let k = key(image);
            if self
                .by_shape
                .get(&shape.id())
                .is_some_and(|(old, _)| *old == k)
            {
                continue;
            }
            let bitmap = &image.bitmap;
            let shrink = bitmap.width().max(bitmap.height()).div_ceil(MAX_SIDE);
            let small = bitmap.shrink(shrink);
            let table = image.adjust.table();
            let pixels: Vec<u8> = small.pixels().iter().map(|p| table[*p as usize]).collect();
            let picture = egui::ColorImage::from_gray(
                [small.width() as usize, small.height() as usize],
                &pixels,
            );
            let handle = ctx.load_texture(
                format!("image-{}", shape.id()),
                picture,
                egui::TextureOptions::LINEAR,
            );
            self.by_shape.insert(shape.id(), (k, handle));
        }
        self.by_shape.retain(|id, _| live.contains(id));
    }

    /// The texture for an image shape.
    #[must_use]
    pub fn get(&self, shape: u32) -> Option<egui::TextureId> {
        self.by_shape.get(&shape).map(|(_, h)| h.id())
    }

    /// How many textures are held.
    #[must_use]
    pub fn count(&self) -> usize {
        self.by_shape.len()
    }
}
