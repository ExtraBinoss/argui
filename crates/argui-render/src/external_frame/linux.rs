//! Vulkan DMA-BUF import, using WGPU's owned-descriptor HAL API.
use super::{ExternalFrame, ExternalFrameMemory, ExternalFrameTransport, transport};
use crate::GpuCanvasError;

/// Imports a validated packed allocation on the same Vulkan GPU.
///
/// Returns an error for unsupported devices or layouts rejected by Vulkan.
pub(super) fn import(
    device: &wgpu::Device,
    frame: &ExternalFrame,
) -> Result<wgpu::Texture, GpuCanvasError> {
    if transport(device) != ExternalFrameTransport::DmaBuf {
        return Err(GpuCanvasError::new(
            "Vulkan DMA-BUF import is unavailable on this device",
        ));
    }
    let ExternalFrameMemory::DmaBuf {
        fd,
        modifier,
        stride,
        offset,
    } = &frame.memory;
    let fd = fd
        .try_clone()
        .map_err(|e| GpuCanvasError::new(e.to_string()))?;
    let descriptor = frame.descriptor.texture_descriptor();
    // SAFETY: the constructor validates the layout and requires an initialized,
    // immutable same-GPU allocation. HAL takes ownership of the duplicated FD.
    let texture = unsafe {
        let hal = device
            .as_hal::<wgpu::hal::api::Vulkan>()
            .ok_or_else(|| GpuCanvasError::new("external frame requires a Vulkan device"))?;
        hal.texture_from_dmabuf_fd(
            fd,
            &wgpu::hal::TextureDescriptor {
                label: descriptor.label,
                size: descriptor.size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: descriptor.dimension,
                format: descriptor.format,
                usage: wgpu::TextureUses::RESOURCE,
                memory_flags: wgpu::hal::MemoryFlags::empty(),
                view_formats: vec![],
            },
            *modifier,
            *stride,
            *offset,
        )
        .map_err(|error| GpuCanvasError::new(format!("DMA-BUF import failed: {error}")))?
    };
    // SAFETY: GENERAL is the completed producer layout required by the constructor.
    Ok(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Vulkan>(
            texture,
            &descriptor,
            wgpu::TextureUses::STORAGE_READ_WRITE,
        )
    })
}
