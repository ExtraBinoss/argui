//! Bounded reuse of popup GPU resources and scene-local asset registration.

use std::collections::HashMap;

use argui_paint::{DisplayCommand, DisplayList, ImageAsset, ImageId, VectorAsset, VectorId};
use argui_render::{RendererError, SurfaceRenderer};

pub(super) struct PopupRenderer {
    pub(super) surface: SurfaceRenderer,
    images: HashMap<ImageId, ImageAsset>,
    vectors: HashMap<VectorId, VectorAsset>,
}

impl PopupRenderer {
    /// Wraps `surface` with the asset versions already installed on this renderer.
    #[allow(unused_mut)]
    pub(super) fn new(mut surface: SurfaceRenderer) -> Self {
        #[cfg(target_os = "linux")]
        surface.prefer_mailbox_presentation();
        Self {
            surface,
            images: HashMap::new(),
            vectors: HashMap::new(),
        }
    }

    /// Uploads `image` only when its exact asset version is not already registered.
    pub(super) fn register_image(&mut self, image: &ImageAsset) -> Result<(), RendererError> {
        if self.images.get(&image.id) != Some(image) {
            self.surface.register_image(image)?;
            self.images.insert(image.id, image.clone());
        }
        Ok(())
    }

    /// Parses `vector` only when its exact asset version is not already registered.
    pub(super) fn register_vector(&mut self, vector: &VectorAsset) -> Result<(), RendererError> {
        if self.vectors.get(&vector.id) != Some(vector) {
            self.surface.register_vector(vector)?;
            self.vectors.insert(vector.id, vector.clone());
        }
        Ok(())
    }

    /// Registers only assets painted by `scene`, including versions changed while closed.
    pub(super) fn register_scene(
        &mut self,
        scene: &DisplayList,
        images: &[ImageAsset],
        vectors: &[VectorAsset],
    ) -> Result<(), RendererError> {
        for command in scene.commands() {
            match command {
                DisplayCommand::Image(item) => {
                    if let Some(asset) = images.iter().find(|asset| asset.id == item.image) {
                        self.register_image(asset)?;
                    }
                }
                DisplayCommand::Vector(item) => {
                    if let Some(asset) = vectors.iter().find(|asset| asset.id == item.vector) {
                        self.register_vector(asset)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/app/popups/renderer.rs"]
mod tests;
