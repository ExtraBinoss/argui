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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_sized_surfaces_are_not_configured() {
        assert_eq!(drawable_size(800, 600), Some((800, 600)));
        assert_eq!(drawable_size(0, 600), None);
        assert_eq!(drawable_size(800, 0), None);
    }

    #[test]
    fn presentation_uses_an_srgb_view_when_the_surface_has_one() {
        assert_eq!(
            srgb_target(TextureFormat::Bgra8Unorm),
            TextureFormat::Bgra8UnormSrgb
        );
        assert_eq!(
            srgb_target(TextureFormat::Rgba8UnormSrgb),
            TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            srgb_target(TextureFormat::Rgba16Float),
            TextureFormat::Rgba16Float
        );
    }

    #[test]
    fn transparent_surfaces_require_premultiplied_composition() {
        assert_eq!(
            surface_alpha_mode(
                SurfaceAlphaMode::Transparent,
                &[
                    CompositeAlphaMode::Opaque,
                    CompositeAlphaMode::PreMultiplied
                ]
            ),
            Ok(CompositeAlphaMode::PreMultiplied)
        );
        assert_eq!(
            surface_alpha_mode(SurfaceAlphaMode::Transparent, &[CompositeAlphaMode::Opaque]),
            Err(RendererError::UnsupportedSurfaceTransparency)
        );
    }

    #[test]
    fn opaque_surfaces_prefer_opaque_and_accept_the_available_mode() {
        assert_eq!(
            surface_alpha_mode(
                SurfaceAlphaMode::Opaque,
                &[
                    CompositeAlphaMode::PreMultiplied,
                    CompositeAlphaMode::Opaque
                ]
            ),
            Ok(CompositeAlphaMode::Opaque)
        );
        assert_eq!(
            surface_alpha_mode(SurfaceAlphaMode::Opaque, &[CompositeAlphaMode::Inherit]),
            Ok(CompositeAlphaMode::Inherit)
        );
        assert_eq!(
            surface_alpha_mode(SurfaceAlphaMode::Opaque, &[]),
            Err(RendererError::UnsupportedSurface)
        );
    }
}
