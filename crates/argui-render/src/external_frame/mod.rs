//! Native GPU frames imported into Argui's selected device without CPU pixel copies.
//!
//! This reviewed FFI boundary owns handles and validates descriptions. Unsafe
//! constructors require completed producer fences and an immutable allocation
//! lease. Every sample is retained through Argui's actual queue submission.
#![allow(unsafe_code)]
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(target_os = "linux")]
mod linux;
mod types;
#[cfg(target_os = "windows")]
mod windows;

use crate::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasRenderContext, GpuCanvasRequirements,
};
use std::sync::Arc;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use types::RetainedIoSurface;
pub use types::{ExternalFrame, ExternalFrameDescriptor, ExternalFrameFormat, ExternalFrameMemory};

/// Selected native-memory transport. Unsupported devices retain ordinary canvas support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalFrameTransport {
    Unavailable,
    DmaBuf,
    IoSurface,
    SharedTexture,
}

/// An imported texture and its retained immutable producer allocation.
pub struct ImportedExternalFrame {
    texture: wgpu::Texture,
    frame: Arc<ExternalFrame>,
}
impl ImportedExternalFrame {
    /// Borrows the texture and retains its producer through this canvas submission.
    ///
    /// Use this each time commands sample a previously imported texture.
    pub fn texture<'a>(&'a self, context: &mut GpuCanvasRenderContext<'_>) -> &'a wgpu::Texture {
        self.retain(context);
        &self.texture
    }
    /// Retains the producer for commands using an already cached bind group.
    pub fn retain(&self, context: &mut GpuCanvasRenderContext<'_>) {
        if !context
            .external_frames
            .iter()
            .any(|frame| Arc::ptr_eq(frame, &self.frame))
        {
            context.external_frames.push(Arc::clone(&self.frame));
        }
    }
    /// Returns the allocated frame description without accessing pixel memory.
    pub fn descriptor(&self) -> ExternalFrameDescriptor {
        self.frame.descriptor()
    }
}
impl ExternalFrame {
    /// Returns optional device features needed by the native import path.
    ///
    /// Pass this contract from a canvas factory's `requirements` method.
    pub fn requirements() -> GpuCanvasRequirements {
        #[cfg(target_os = "linux")]
        return GpuCanvasRequirements::default()
            .optional_features(wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF)
            .reason("Sample external GPU allocations without CPU pixel transfers");
        #[cfg(not(target_os = "linux"))]
        GpuCanvasRequirements::default()
    }
}
impl GpuCanvasDeviceContext<'_> {
    /// Reports the actual import transport enabled on the selected device.
    pub fn external_frame_transport(&self) -> ExternalFrameTransport {
        transport(self.device())
    }
}
impl GpuCanvasRenderContext<'_> {
    /// Imports `frame` on Argui's device and retains its allocation for this submission.
    ///
    /// # Errors
    /// Returns an error for unsupported backends, oversized allocations, or an
    /// incompatible native handle/format. No CPU fallback is performed by this API.
    pub fn import_external_frame(
        &mut self,
        frame: Arc<ExternalFrame>,
    ) -> Result<ImportedExternalFrame, GpuCanvasError> {
        frame
            .descriptor
            .validate(self.device().limits().max_texture_dimension_2d)?;
        #[cfg(target_os = "linux")]
        let texture = linux::import(self.device(), &frame)?;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        let texture = apple::import(self.device(), &frame)?;
        #[cfg(target_os = "windows")]
        let texture = windows::import(self.device(), &frame)?;
        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "ios",
            target_os = "windows"
        )))]
        return Err(GpuCanvasError::new(
            "native external frame import is unavailable on this platform",
        ));
        #[cfg(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "ios",
            target_os = "windows"
        ))]
        {
            let imported = ImportedExternalFrame { texture, frame };
            imported.retain(self);
            Ok(imported)
        }
    }
}
/// Detects a validated native import backend without changing device requirements.
fn transport(device: &wgpu::Device) -> ExternalFrameTransport {
    #[cfg(target_os = "linux")]
    if device
        .features()
        .contains(wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF)
        && unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() }.is_some()
    {
        return ExternalFrameTransport::DmaBuf;
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    if unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.is_some() {
        return ExternalFrameTransport::IoSurface;
    }
    #[cfg(target_os = "windows")]
    if unsafe { device.as_hal::<wgpu::hal::api::Dx12>() }.is_some() {
        return ExternalFrameTransport::SharedTexture;
    }
    let _ = device;
    ExternalFrameTransport::Unavailable
}
