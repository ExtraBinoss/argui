//! Owned native GPU allocations and their producer leases.
use crate::GpuCanvasError;
use std::{any::Any, sync::Arc};

/// Packed pixel format of an external single-plane image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalFrameFormat {
    Rgba8,
    Rgba8Srgb,
    Bgra8,
    Bgra8Srgb,
}
impl ExternalFrameFormat {
    /// Returns the WGPU sampling format; no conversion or pixel copy is performed.
    pub const fn texture_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
            Self::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Bgra8 => wgpu::TextureFormat::Bgra8Unorm,
            Self::Bgra8Srgb => wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }
}

/// Allocated extent and pixel interpretation, independent of the platform handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExternalFrameDescriptor {
    pub size: [u32; 2],
    pub format: ExternalFrameFormat,
}
impl ExternalFrameDescriptor {
    /// Checks the allocated extent against `max_dimension`.
    ///
    /// # Errors
    /// Returns an error for empty or over-limit allocations.
    pub fn validate(self, max_dimension: u32) -> Result<(), GpuCanvasError> {
        if self.size.contains(&0) || self.size.iter().any(|&n| n > max_dimension) {
            return Err(GpuCanvasError::new("invalid external GPU frame extent"));
        }
        Ok(())
    }
    /// Validates packed RGBA/BGRA row layout without reading the allocation.
    ///
    /// # Errors
    /// Rejects a row narrower than the image and overflowing native byte offsets.
    pub fn validate_row_layout(self, stride: u64, offset: u64) -> Result<(), GpuCanvasError> {
        self.validate(u32::MAX)?;
        if stride < u64::from(self.size[0]) * 4
            || stride
                .checked_mul(u64::from(self.size[1]))
                .and_then(|bytes| offset.checked_add(bytes))
                .is_none()
        {
            return Err(GpuCanvasError::new("invalid external GPU row layout"));
        }
        Ok(())
    }
    /// Describes the initialized native allocation as a sampled WGPU texture.
    pub(super) fn texture_descriptor(&self) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("argui-external-frame"),
            size: wgpu::Extent3d {
                width: self.size[0],
                height: self.size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format.texture_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }
    }
}

/// Native allocation owned by the frame. Handles never pass through the UI DSL.
pub enum ExternalFrameMemory {
    #[cfg(target_os = "linux")]
    DmaBuf {
        fd: std::os::fd::OwnedFd,
        modifier: u64,
        stride: u64,
        offset: u64,
    },
    #[cfg(target_os = "windows")]
    SharedTexture(std::os::windows::io::OwnedHandle),
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    IoSurface(RetainedIoSurface),
}

/// A completed GPU image with a lease preventing producer reuse.
///
/// Argui retains a frame for every command buffer which samples it, until that
/// submission completes. The producer owner must keep the allocation immutable
/// until its last lease is released.
pub struct ExternalFrame {
    pub(super) descriptor: ExternalFrameDescriptor,
    pub(super) memory: ExternalFrameMemory,
    pub(super) _producer: Arc<dyn Any + Send + Sync>,
}
impl ExternalFrame {
    /// Takes ownership of `memory` and the allocation's `producer` lease.
    ///
    /// # Safety
    /// The native allocation must match `descriptor`, belong to the rendering
    /// GPU, and be fully written (producer fences completed). Vulkan DMA-BUF
    /// must be released in GENERAL; a shared D3D texture must be in COMMON.
    /// `producer` must prevent writes/recycling until all frame leases drop.
    /// Only single-plane packed RGBA/BGRA allocations are accepted.
    ///
    /// # Errors
    /// Returns an error for invalid dimensions or DMA-BUF row layout.
    pub unsafe fn new(
        descriptor: ExternalFrameDescriptor,
        memory: ExternalFrameMemory,
        producer: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, GpuCanvasError> {
        descriptor.validate(u32::MAX)?;
        #[cfg(target_os = "linux")]
        match &memory {
            ExternalFrameMemory::DmaBuf {
                modifier,
                stride,
                offset,
                ..
            } => {
                if *modifier == u64::MAX || *modifier == 0x00ff_ffff_ffff_ffff {
                    return Err(GpuCanvasError::new("DMA-BUF requires an explicit modifier"));
                }
                descriptor.validate_row_layout(*stride, *offset)?;
            }
        }
        Ok(Self {
            descriptor,
            memory,
            _producer: producer,
        })
    }
    /// Returns the allocated dimensions and pixel interpretation.
    pub const fn descriptor(&self) -> ExternalFrameDescriptor {
        self.descriptor
    }
}

/// Retained immutable IOSurface, transferable between producer and render threads.
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub struct RetainedIoSurface(
    pub(super) objc2_core_foundation::CFRetained<objc2_io_surface::IOSurfaceRef>,
);
// SAFETY: IOSurface references support cross-thread retain/release. This wrapper
// exposes no mutation; ExternalFrame::new requires completed writes and no reuse.
#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe impl Send for RetainedIoSurface {}
#[cfg(any(target_os = "macos", target_os = "ios"))]
unsafe impl Sync for RetainedIoSurface {}
#[cfg(any(target_os = "macos", target_os = "ios"))]
impl RetainedIoSurface {
    /// Retains a producer IOSurface reference without mapping pixel memory.
    ///
    /// # Safety
    /// `surface` must point to a live IOSurface for the duration of this call.
    pub unsafe fn retain(surface: std::ptr::NonNull<std::ffi::c_void>) -> Self {
        // SAFETY: caller guarantees the native object is alive while retaining it.
        Self(unsafe { objc2_core_foundation::CFRetained::retain(surface.cast()) })
    }
}
