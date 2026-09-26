use wgpu::{CompositeAlphaMode, TextureFormat};

/// Describes failed backends in the warning shown after a compatibility fallback succeeds.
///
/// `failures` are earlier initialization errors, and `selected` names the backend in use.
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "android"))]
pub(super) fn fallback_message(
    failures: &[crate::RendererAttemptFailure],
    selected: &str,
) -> String {
    let failures = failures
        .iter()
        .map(|failure| format!("{} failed: {}", failure.renderer, failure.error))
        .collect::<Vec<_>>()
        .join("; ");
    #[cfg(target_os = "windows")]
    {
        format!(
            "Renderer fallback activated. {failures}. Continuing with {selected}. Desktop backdrop effects are disabled for this session."
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("Renderer fallback activated. {failures}. Continuing with {selected}.")
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WindowsRenderer {
    DirectX12DirectComposition,
    DirectX12Opaque,
    Vulkan,
}

#[cfg(target_os = "windows")]
impl WindowsRenderer {
    /// Returns the renderer configurations to try in preference order.
    ///
    /// `fallback_enabled` controls whether the opaque DirectX 12 and Vulkan
    /// compatibility configurations are included.
    pub(super) fn attempts(fallback_enabled: bool) -> &'static [Self] {
        const PREFERRED: &[WindowsRenderer] = &[WindowsRenderer::DirectX12DirectComposition];
        const WITH_FALLBACK: &[WindowsRenderer] = &[
            WindowsRenderer::DirectX12DirectComposition,
            WindowsRenderer::DirectX12Opaque,
            WindowsRenderer::Vulkan,
        ];
        if fallback_enabled {
            WITH_FALLBACK
        } else {
            PREFERRED
        }
    }

    /// Returns a concise English name for diagnostics shown to users.
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::DirectX12DirectComposition => "DirectX 12 with DirectComposition",
            Self::DirectX12Opaque => "DirectX 12 with an opaque window surface",
            Self::Vulkan => "Vulkan",
        }
    }
}

#[cfg(all(not(target_os = "windows"), not(target_arch = "wasm32")))]
/// Creates the preferred GPU instance for the current platform.
pub(super) fn instance() -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        descriptor.backends = wgpu::Backends::VULKAN;
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        descriptor.backends = wgpu::Backends::METAL;
    }
    wgpu::Instance::new(descriptor)
}

/// Creates a browser WebGPU instance for Argui's storage-buffer shaders.
#[cfg(target_arch = "wasm32")]
pub(super) fn browser_instance() -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::BROWSER_WEBGPU;
    wgpu::Instance::new(descriptor)
}

/// Creates the OpenGL compatibility instance after Vulkan initialization fails.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(super) fn fallback_instance() -> wgpu::Instance {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::GL;
    wgpu::Instance::new(descriptor)
}

/// Creates a Windows WGPU instance for one renderer initialization attempt.
#[cfg(target_os = "windows")]
pub(super) fn instance_for(renderer: WindowsRenderer) -> wgpu::Instance {
    match renderer {
        WindowsRenderer::DirectX12DirectComposition => {
            // Shared devices must also accept transparent windows created later.
            // DXGI HWND swapchains are opaque; WGPU owns the DirectComposition visual instead.
            let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
            descriptor.backends = wgpu::Backends::DX12;
            descriptor.backend_options.dx12.presentation_system =
                wgpu::Dx12SwapchainKind::DxgiFromVisual;
            wgpu::Instance::new(descriptor)
        }
        WindowsRenderer::DirectX12Opaque => {
            let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
            descriptor.backends = wgpu::Backends::DX12;
            descriptor.backend_options.dx12.presentation_system =
                wgpu::Dx12SwapchainKind::DxgiFromHwnd;
            wgpu::Instance::new(descriptor)
        }
        WindowsRenderer::Vulkan => {
            let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
            descriptor.backends = wgpu::Backends::VULKAN;
            wgpu::Instance::new(descriptor)
        }
    }
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
