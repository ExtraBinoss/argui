use wgpu::{CompositeAlphaMode, TextureFormat};

use crate::{RendererError, SurfaceAlphaMode};

pub(super) const fn drawable_size(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

pub(super) fn srgb_target(format: TextureFormat) -> TextureFormat {
    format.add_srgb_suffix()
}

pub(super) fn surface_alpha_mode(
    requested: SurfaceAlphaMode,
    supported: &[CompositeAlphaMode],
) -> Result<CompositeAlphaMode, RendererError> {
    match requested {
        SurfaceAlphaMode::Opaque => supported
            .contains(&CompositeAlphaMode::Opaque)
            .then_some(CompositeAlphaMode::Opaque)
            .or_else(|| supported.first().copied())
            .ok_or(RendererError::UnsupportedSurface),
        SurfaceAlphaMode::Transparent => supported
            .contains(&CompositeAlphaMode::PreMultiplied)
            .then_some(CompositeAlphaMode::PreMultiplied)
            .ok_or(RendererError::UnsupportedSurfaceTransparency),
    }
}
