//! Direct3D 12 import of an owned NT shared texture handle.
use super::{ExternalFrame, ExternalFrameMemory};
use crate::GpuCanvasError;
use std::os::windows::io::AsRawHandle;
use windows::Win32::{
    Foundation::HANDLE,
    Graphics::{
        Direct3D12::{D3D12_RESOURCE_DIMENSION_TEXTURE2D, ID3D12Resource},
        Dxgi::Common::*,
    },
};

/// Opens a same-adapter shared texture without mapping or copying pixels.
///
/// Returns an error for incompatible backends, handles, resource extents or formats.
pub(super) fn import(
    device: &wgpu::Device,
    frame: &ExternalFrame,
) -> Result<wgpu::Texture, GpuCanvasError> {
    let ExternalFrameMemory::SharedTexture(handle) = &frame.memory;
    let descriptor = frame.descriptor.texture_descriptor();
    // SAFETY: the guard borrows the actual DX12 backend; only a new resource is opened.
    let hal = unsafe { device.as_hal::<wgpu::hal::api::Dx12>() }.ok_or_else(|| {
        GpuCanvasError::new("shared texture import requires a Direct3D 12 device")
    })?;
    let mut resource = None;
    // SAFETY: handle is owned and live; the initialized out pointer requests ID3D12Resource.
    unsafe {
        hal.raw_device()
            .OpenSharedHandle::<ID3D12Resource>(HANDLE(handle.as_raw_handle()), &raw mut resource)
    }
    .map_err(|e| GpuCanvasError::new(format!("shared texture import failed: {e}")))?;
    let resource = resource.ok_or_else(|| GpuCanvasError::new("shared handle has no texture"))?;
    // SAFETY: the opened COM object is live for this query.
    let raw = unsafe { resource.GetDesc() };
    let expected = match descriptor.format {
        wgpu::TextureFormat::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
        wgpu::TextureFormat::Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        wgpu::TextureFormat::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
        wgpu::TextureFormat::Bgra8UnormSrgb => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
        _ => unreachable!("external frames use only packed RGBA/BGRA"),
    };
    if raw.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
        || raw.Width != u64::from(descriptor.size.width)
        || raw.Height != descriptor.size.height
        || raw.DepthOrArraySize != 1
        || raw.MipLevels != 1
        || raw.SampleDesc.Count != 1
        || raw.Format != expected
    {
        return Err(GpuCanvasError::new(
            "shared texture layout does not match frame",
        ));
    }
    // SAFETY: the checked shared resource matches this device and descriptor.
    let texture = unsafe {
        <wgpu::hal::api::Dx12 as wgpu::hal::Api>::Device::texture_from_raw(
            resource,
            descriptor.format,
            descriptor.dimension,
            descriptor.size,
            1,
            1,
        )
    };
    // SAFETY: PRESENT maps to COMMON in DX12, as required by ExternalFrame::new.
    Ok(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Dx12>(
            texture,
            &descriptor,
            wgpu::TextureUses::PRESENT,
        )
    })
}
