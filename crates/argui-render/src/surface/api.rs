use super::*;

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Returns whether the surface uses premultiplied transparency.
    #[must_use]
    pub fn is_transparent(&self) -> bool {
        self.surface_config.alpha_mode == wgpu::CompositeAlphaMode::PreMultiplied
    }

    /// Creates a renderer and requests a compatible GPU device for `target`.
    /// * `target` — native window or surface target; `width`, `height` — initial drawable dimensions.
    /// * `renderer_config` — renderer behavior and resource configuration.
    ///
    /// # Errors
    /// Returns an error if surface creation, adapter/device selection, or surface configuration fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        let instance = configure::instance();
        let surface = instance
            .create_surface(target)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: renderer_config.power_preference,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::AdapterRequest(error.to_string()))?;
        let required_features = if renderer_config.profiling
            && adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
        {
            wgpu::Features::TIMESTAMP_QUERY
        } else {
            wgpu::Features::empty()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("argui-device"),
                required_features,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;
        let device_handle = RendererDevice(Arc::new(RendererDeviceInner {
            instance: instance.clone(),
            adapter,
            device,
            queue,
        }));
        Self::from_existing_device(
            instance,
            surface,
            device_handle,
            width,
            height,
            renderer_config,
        )
    }

    /// Creates a renderer using an existing shared GPU device handle.
    ///
    /// # Errors
    /// Returns an error if the surface cannot be created or configured for the supplied device.
    /// * `target` — native window or surface target; `width`, `height` — initial drawable dimensions.
    /// * `renderer_config` — renderer behavior configuration; `device_handle` — existing GPU device and queue.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new_with_device(
        target: impl Into<SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
        device_handle: RendererDevice,
    ) -> Result<Self, RendererError> {
        let instance = device_handle.0.instance.clone();
        let surface = instance
            .create_surface(target)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        Self::from_existing_device(
            instance,
            surface,
            device_handle,
            width,
            height,
            renderer_config,
        )
    }

    /// Returns a cloneable handle to the GPU device and queue shared by this renderer.
    #[must_use]
    pub fn device_handle(&self) -> RendererDevice {
        self.device_handle.clone()
    }

    /// Resizes the surface; returns whether its drawable extent changed.
    /// * `width`, `height` — requested drawable dimensions in physical pixels.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        let Some((width, height)) = drawable_size(width, height) else {
            return false;
        };
        if self.surface_config.width == width && self.surface_config.height == height {
            return false;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        self.layer_cache.clear();
        true
    }

    /// Replaces the native surface target while retaining the existing GPU device.
    ///
    /// # Errors
    /// Returns an error if creating the replacement surface fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn recreate_surface(
        &mut self,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<(), RendererError> {
        self.surface = self
            .instance
            .create_surface(target)
            .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?;
        self.surface.configure(&self.device, &self.surface_config);
        self.layer_cache.clear();
        Ok(())
    }

    /// Renders an empty frame and presents it.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render(&mut self) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::None, || {})
    }

    /// Renders an empty frame and invokes `notify` when the frame is submitted.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    pub fn render_notified(
        &mut self,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::None, notify)
    }

    /// Renders prepared text and presents it.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    /// * `engine` — text engine containing GPU text resources; `text` — prepared text to draw.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_text(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Text { engine, text }, || {})
    }

    /// Renders prepared text and invokes `notify` when the frame is submitted.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    /// * `engine` — text engine containing GPU text resources; `text` — prepared text to draw.
    /// * `notify` — callback invoked after the frame is submitted.
    pub fn render_text_notified(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Text { engine, text }, notify)
    }

    /// Renders a display list with prepared text at the given scale factor.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    /// * `engine` — text engine containing GPU text resources; `text` — prepared text to draw.
    /// * `display_list` — UI draw commands; `scale_factor` — logical-to-physical scale.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_ui(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(
            FrameContent::Ui {
                engine,
                text,
                display_list,
                scale_factor,
            },
            || {},
        )
    }

    /// Renders a display list and invokes `notify` when the frame is submitted.
    ///
    /// # Errors
    /// Returns a renderer error if frame acquisition or rendering fails.
    /// * `engine` — text engine containing GPU text resources; `text` — prepared text to draw.
    /// * `display_list` — UI draw commands; `scale_factor` — logical-to-physical scale.
    /// * `notify` — callback invoked after the frame is submitted.
    pub fn render_ui_notified(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: &DisplayList,
        scale_factor: f32,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(
            FrameContent::Ui {
                engine,
                text,
                display_list,
                scale_factor,
            },
            notify,
        )
    }

    /// Returns current statistics for the offscreen texture pool.
    #[must_use]
    pub fn texture_pool_stats(&self) -> TexturePoolStats {
        self.offscreen.stats()
    }

    /// Returns profiling data from the most recently rendered frame.
    #[must_use]
    pub fn last_profile(&self) -> RenderProfile {
        self.last_profile.clone()
    }

    /// Enables profiling for subsequent frames when profiling is enabled in the configuration.
    /// * `active` — whether subsequent frame profiling is active.
    pub fn set_profiling_active(&mut self, active: bool) {
        self.profiling_active = self.renderer_config.profiling && active;
    }

    /// Registers an image asset for rendering.
    ///
    /// # Errors
    /// Returns a renderer error if the asset cannot be uploaded or registered.
    pub fn register_image(&mut self, asset: &ImageAsset) -> Result<(), RendererError> {
        self.image.register(&self.device, &self.queue, asset)
    }

    /// Registers an SVG vector asset for rendering.
    ///
    /// # Errors
    /// Returns a renderer error if the asset cannot be rasterized or registered.
    pub fn register_vector(&mut self, asset: &VectorAsset) -> Result<(), RendererError> {
        self.vector.register(asset)
    }
}
