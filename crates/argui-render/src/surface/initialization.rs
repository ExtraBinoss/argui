use super::*;
use crate::SurfaceAlphaMode;

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Reuses `device_handle` and `instance` for a visible `surface` of the
    /// requested pixel size, applying `renderer_config` to its render resources.
    /// Returns a renderer or a surface/resource configuration error.
    pub(super) fn from_existing_device(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        device_handle: RendererDevice,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        Self::finish_new(
            instance,
            Some(surface),
            device_handle.0.adapter.clone(),
            device_handle.0.device.clone(),
            device_handle.0.queue.clone(),
            device_handle,
            width,
            height,
            renderer_config,
        )
    }

    /// Creates shared render resources from `instance`, `adapter`, `device`,
    /// `queue`, and `device_handle`. `surface` selects visible or offscreen
    /// output; `width`, `height`, and `renderer_config` set target properties.
    /// Returns a renderer or a device, surface, or resource configuration error.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_new(
        instance: wgpu::Instance,
        surface: Option<wgpu::Surface<'static>>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        device_handle: RendererDevice,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        renderer_config
            .gpu_canvases
            .validate_device(device.features(), &device.limits())?;
        let mut surface_config = if let Some(surface) = &surface {
            let mut config = surface
                .get_default_config(&adapter, width.max(1), height.max(1))
                .ok_or(RendererError::UnsupportedSurface)?;
            config.present_mode = renderer_config.present_mode;
            config.desired_maximum_frame_latency = renderer_config.maximum_frame_latency;
            config.alpha_mode = surface_alpha_mode(
                renderer_config.surface_alpha,
                &surface.get_capabilities(&adapter).alpha_modes,
            )?;
            config
        } else {
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rgba8Unorm,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: width.max(1),
                height: height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: if renderer_config.surface_alpha == SurfaceAlphaMode::Transparent {
                    wgpu::CompositeAlphaMode::PreMultiplied
                } else {
                    wgpu::CompositeAlphaMode::Opaque
                },
                view_formats: Vec::new(),
            }
        };
        let target_format = srgb_target(surface_config.format);
        if target_format != surface_config.format {
            surface_config.view_formats.push(target_format);
        }
        if let Some(surface) = &surface {
            surface.configure(&device, &surface_config);
        }
        let offscreen_target = surface
            .is_none()
            .then(|| offscreen_texture(&device, surface_config.width, surface_config.height));
        renderer_config.gpu_canvases.device_ready(
            &device,
            &queue,
            target_format,
            device_handle.0.generation,
        );
        let quad = QuadGpu::new(
            &device,
            target_format,
            renderer_config.gradient_stop_capacity,
        );
        let text = TextGpu::new(&device, target_format);
        let image = ImageGpu::new(&device, target_format, renderer_config.image_cache_bytes);
        let vector = VectorGpu::new(&device, target_format);
        let gpu_canvas = CanvasGpu::new(
            &device,
            &queue,
            target_format,
            device_handle.0.generation,
            renderer_config.gpu_canvases.clone(),
            renderer_config.gpu_canvas_cache_bytes,
        );
        let maximum_parameter_words = renderer_config
            .effects
            .definitions()
            .iter()
            .map(crate::EffectDefinition::parameter_words)
            .max()
            .unwrap_or(1);
        let maximum_storage_bytes = device.limits().max_storage_buffer_binding_size as usize;
        let parameter_bytes = maximum_parameter_words * size_of::<u32>();
        if parameter_bytes > maximum_storage_bytes {
            return Err(RendererError::EffectParametersTooLarge {
                provided: parameter_bytes,
                maximum: maximum_storage_bytes,
            });
        }
        let effect = EffectGpu::new(&device, target_format, maximum_parameter_words);
        let damage = DamageGpu::new(&device, target_format);
        let offscreen = TexturePool::new(target_format, 32 * 1024 * 1024);
        let gpu_profiler = GpuProfiler::new(&adapter, &device, &queue, renderer_config.profiling);

        let profiling_active = renderer_config.profiling;
        Ok(Self {
            device_handle,
            instance,
            surface,
            offscreen_target,
            alpha_target: None,
            device,
            queue,
            surface_config,
            target_format,
            renderer_config,
            batches: Vec::new(),
            text_ranges: Vec::new(),
            text_bounds: Vec::new(),
            quad,
            text,
            image,
            vector,
            gpu_canvas,
            effect,
            offscreen,
            gpu_profiler,
            last_profile: RenderProfile::default(),
            profiling_active,
            layer_cache: HashMap::new(),
            content_revision: 0,
            damage,
            scene_snapshot: None,
            effect_root: None,
            deferred_reconfigure: false,
        })
    }
}
