use std::collections::HashMap;

use argui_core::Size;
use argui_paint::{ImageAsset, ImageId, VectorAsset, VectorId};
use argui_ui::{ElementKind, LayoutStyle};
use taffy::geometry::Size as TaffySize;

#[derive(Debug, Default)]
pub(crate) struct AssetMetrics {
    images: HashMap<ImageId, Size>,
    vectors: HashMap<VectorId, Size>,
}

impl AssetMetrics {
    pub(crate) fn update(&mut self, images: &[ImageAsset], vectors: &[VectorAsset]) -> bool {
        let next_images = images
            .iter()
            .map(|asset| (asset.id, Size::new(asset.width as f32, asset.height as f32)))
            .collect();
        let next_vectors = vectors.iter().map(|asset| (asset.id, asset.size)).collect();
        if self.images == next_images && self.vectors == next_vectors {
            return false;
        }
        self.images = next_images;
        self.vectors = next_vectors;
        true
    }

    pub(crate) fn intrinsic(&self, kind: &ElementKind) -> Option<Size> {
        match kind {
            ElementKind::Image { image, .. } => self.images.get(image).copied(),
            ElementKind::Vector { vector, .. } => self.vectors.get(vector).copied(),
            ElementKind::Custom(_)
            | ElementKind::GpuCanvas(_)
            | ElementKind::Container
            | ElementKind::Text { .. }
            | ElementKind::TextEditor { .. } => None,
        }
    }

    pub(crate) fn layout_style(&self, mut style: LayoutStyle, kind: &ElementKind) -> LayoutStyle {
        if style.aspect_ratio.is_none()
            && let Some(size) = self.intrinsic(kind)
            && size.height > 0.0
        {
            style.aspect_ratio = Some(size.width / size.height);
        }
        style
    }
}

pub(crate) fn resolve_intrinsic(known: TaffySize<Option<f32>>, intrinsic: Size) -> TaffySize<f32> {
    let ratio = intrinsic.width / intrinsic.height.max(f32::EPSILON);
    match (known.width, known.height) {
        (Some(width), Some(height)) => TaffySize { width, height },
        (Some(width), None) => TaffySize {
            width,
            height: width / ratio,
        },
        (None, Some(height)) => TaffySize {
            width: height * ratio,
            height,
        },
        (None, None) => TaffySize {
            width: intrinsic.width,
            height: intrinsic.height,
        },
    }
}
