use super::*;

#[cfg(not(target_arch = "wasm32"))]
impl SurfaceRenderer {
    /// Creates the normal scene renderer with a windowless RGBA target.
    /// `width` and `height` are physical pixels and `renderer_config` selects
    /// the same paint resources used by visible surfaces.
    ///
    /// # Errors
    /// Returns an adapter, device, or resource error when the GPU cannot render.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_offscreen(
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: renderer_config.power_preference,
                compatible_surface: None,
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::AdapterRequest(error.to_string()))?;
        let requirements = renderer_config.gpu_canvases.device_requirements(
            adapter.features(),
            &adapter.limits(),
            renderer_config.profiling,
        )?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("argui-offscreen-device"),
                required_features: requirements.features,
                required_limits: requirements.limits,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;
        let handle = RendererDevice(Arc::new(RendererDeviceInner {
            instance: instance.clone(),
            adapter: adapter.clone(),
            device: device.clone(),
            queue: queue.clone(),
            initialization_fallback: None,
            generation: next_device_generation(),
        }));
        Self::finish_new(
            instance,
            None,
            adapter,
            device,
            queue,
            handle,
            width,
            height,
            renderer_config,
        )
    }

    /// Reads the last rendered windowless scene as packed RGBA8 pixels.
    /// The caller supplies no buffer; the result has `width * height * 4` bytes.
    /// Completed GPU timestamp queries update [`Self::last_profile`] before return.
    ///
    /// # Errors
    /// Returns an error for a visible surface, mapping failure, or GPU poll failure.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_offscreen_rgba(&mut self) -> Result<Vec<u8>, RendererError> {
        let texture = self
            .offscreen_target
            .as_ref()
            .ok_or_else(|| RendererError::Readback("renderer has a visible surface".into()))?;
        let width = self.surface_config.width;
        let height = self.surface_config.height;
        let stride = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("argui-scene-readback"),
            size: u64::from(stride) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| RendererError::Readback(error.to_string()))?;
        if let Some(gpu) = self.gpu_profiler.take_latest() {
            self.last_profile.gpu = Some(gpu);
        }
        receiver
            .recv()
            .map_err(|error| RendererError::Readback(error.to_string()))?
            .map_err(|error| RendererError::Readback(error.to_string()))?;
        let mapped = readback
            .slice(..)
            .get_mapped_range()
            .map_err(|error| RendererError::Readback(error.to_string()))?;
        Ok(mapped
            .chunks_exact(stride as usize)
            .flat_map(|row| row[..width as usize * 4].iter().copied())
            .collect())
    }
}

/// Creates an RGBA texture with an sRGB render view and copyable pixels.
/// `device` allocates the texture, and `width` and `height` are nonzero pixel dimensions.
pub(super) fn offscreen_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("argui-offscreen-scene"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
    })
}
