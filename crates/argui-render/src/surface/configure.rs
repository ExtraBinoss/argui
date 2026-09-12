use wgpu::{CompositeAlphaMode, TextureFormat};

pub(super) fn instance() -> wgpu::Instance {
    #[cfg(target_os = "windows")]
    {
        // Shared devices must also accept transparent windows created later.
        // DXGI HWND swapchains are opaque; WGPU owns the DirectComposition visual instead.
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        descriptor.backend_options.dx12.presentation_system =
            wgpu::Dx12SwapchainKind::DxgiFromVisual;
        wgpu::Instance::new(descriptor)
    }
    #[cfg(not(target_os = "windows"))]
    wgpu::Instance::default()
}

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
    // WebGPU guarantees premultiplied canvas composition. Its WGPU capability
    // list only reports Opaque, although configure accepts PreMultiplied.
    #[cfg(target_arch = "wasm32")]
    if requested != SurfaceAlphaMode::Opaque {
        return Ok(CompositeAlphaMode::PreMultiplied);
    }
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
        SurfaceAlphaMode::PreferTransparent => supported
            .contains(&CompositeAlphaMode::PreMultiplied)
            .then_some(CompositeAlphaMode::PreMultiplied)
            .or_else(|| supported.first().copied())
            .ok_or(RendererError::UnsupportedSurface),
    }
}
