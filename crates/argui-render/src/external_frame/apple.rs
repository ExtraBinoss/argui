//! Metal IOSurface import. Native objects are retained; pixels remain on the GPU.
use super::{ExternalFrame, ExternalFrameFormat, ExternalFrameMemory};
use crate::GpuCanvasError;
use objc2_metal::{
    MTLDevice as _, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};

/// Imports a retained IOSurface into the actual Metal device.
///
/// Returns an error when the device or IOSurface cannot supply the described texture.
pub(super) fn import(
    device: &wgpu::Device,
    frame: &ExternalFrame,
) -> Result<wgpu::Texture, GpuCanvasError> {
    let ExternalFrameMemory::IoSurface(surface) = &frame.memory;
    let surface = &surface.0;
    let descriptor = frame.descriptor.texture_descriptor();
    let format = match frame.descriptor.format {
        ExternalFrameFormat::Rgba8 => MTLPixelFormat::RGBA8Unorm,
        ExternalFrameFormat::Rgba8Srgb => MTLPixelFormat::RGBA8Unorm_sRGB,
        ExternalFrameFormat::Bgra8 => MTLPixelFormat::BGRA8Unorm,
        ExternalFrameFormat::Bgra8Srgb => MTLPixelFormat::BGRA8Unorm_sRGB,
    };
    let texture = objc2::rc::autoreleasepool(|_| {
        // SAFETY: borrow the actual backend, creating only this independent texture.
        let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }
            .ok_or_else(|| GpuCanvasError::new("IOSurface import requires a Metal device"))?;
        let raw = hal.raw_device();
        if surface.width() != frame.descriptor.size[0] as usize
            || surface.height() != frame.descriptor.size[1] as usize
        {
            return Err(GpuCanvasError::new("IOSurface extent does not match frame"));
        }
        let desc = MTLTextureDescriptor::new();
        // SAFETY: validated nonzero dimensions fit NSUInteger on supported Apple targets.
        unsafe {
            desc.setWidth(frame.descriptor.size[0] as usize);
            desc.setHeight(frame.descriptor.size[1] as usize);
        }
        desc.setTextureType(MTLTextureType::Type2D);
        desc.setPixelFormat(format);
        desc.setUsage(MTLTextureUsage::ShaderRead);
        desc.setStorageMode(if raw.hasUnifiedMemory() {
            MTLStorageMode::Shared
        } else {
            MTLStorageMode::Managed
        });
        let raw_texture = raw
            .newTextureWithDescriptor_iosurface_plane(&desc, surface, 0)
            .ok_or_else(|| GpuCanvasError::new("Metal rejected the IOSurface frame"))?;
        // SAFETY: the freshly created texture exactly matches the checked descriptor.
        Ok(unsafe {
            <wgpu::hal::api::Metal as wgpu::hal::Api>::Device::texture_from_raw(
                raw_texture,
                descriptor.format,
                MTLTextureType::Type2D,
                1,
                1,
                wgpu::hal::CopyExtent {
                    width: descriptor.size.width,
                    height: descriptor.size.height,
                    depth: 1,
                },
                None,
            )
        })
    })?;
    // SAFETY: Metal does not track Vulkan-style layouts; completed ShaderRead texture.
    Ok(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            texture,
            &descriptor,
            wgpu::TextureUses::RESOURCE,
        )
    })
}
