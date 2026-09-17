use super::*;

/// Builds the warning emitted after Windows selects a compatibility renderer.
///
/// `failures` contains earlier initialization errors and `selected` names the
/// renderer configuration that succeeded.
#[cfg(target_os = "windows")]
fn fallback_message(failures: &[crate::RendererAttemptFailure], selected: &str) -> String {
    let failures = failures
        .iter()
        .map(|failure| format!("{} failed: {}", failure.renderer, failure.error))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "Renderer fallback activated. {failures}. Continuing with {selected}. Desktop backdrop effects are disabled for this session."
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Returns whether the surface uses premultiplied transparency.
    #[must_use]
    pub fn is_transparent(&self) -> bool {
        self.surface_config.alpha_mode == wgpu::CompositeAlphaMode::PreMultiplied
    }

    /// Returns the user-facing warning produced when renderer initialization used a fallback.
    ///
    /// The returned message includes each failed configuration and the renderer that was
    /// ultimately selected. `None` means the preferred renderer initialized successfully.
    #[must_use]
    pub fn initialization_fallback(&self) -> Option<&str> {
        self.device_handle.0.initialization_fallback.as_deref()
    }

    /// Creates a renderer and requests a compatible GPU device for `target`.
    /// * `target` — native window or surface target; `width`, `height` — initial drawable dimensions.
    /// * `renderer_config` — renderer behavior and resource configuration.
    ///
    /// # Errors
    /// Returns an error if surface creation, adapter/device selection, or surface configuration fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new<T>(
        target: T,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError>
    where
        T: Into<SurfaceTarget<'static>> + Clone,
    {
        #[cfg(target_os = "windows")]
        {
            let mut failures = Vec::new();
            for renderer in configure::WindowsRenderer::attempts(renderer_config.renderer_fallback)
            {
                let fallback_message =
                    (!failures.is_empty()).then(|| fallback_message(&failures, renderer.label()));
                let result = Self::new_with_instance(
                    target.clone(),
                    width,
                    height,
                    renderer_config.clone(),
                    configure::instance_for(*renderer),
                    fallback_message,
                )
                .await;
                match result {
                    Ok(renderer) => return Ok(renderer),
                    Err(error) => failures.push(crate::RendererAttemptFailure {
                        renderer: renderer.label().into(),
                        error: error.to_string(),
                    }),
                }
            }
            Err(RendererError::Initialization {
                attempts: failures,
                fallback_enabled: renderer_config.renderer_fallback,
            })
        }

        #[cfg(not(target_os = "windows"))]
        Self::new_with_instance(
            target,
            width,
            height,
            renderer_config,
            configure::instance(),
            None,
        )
        .await
    }

    /// Creates a renderer using one already-selected WGPU instance.
    ///
    /// `target` identifies the window surface, `width` and `height` specify its initial
    /// drawable size, `renderer_config` controls resources and presentation,
    /// `instance` selects the GPU API, and `initialization_fallback` records any
    /// compatibility fallback exposed to the runtime.
    async fn new_with_instance(
        target: impl Into<SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
        instance: wgpu::Instance,
        initialization_fallback: Option<String>,
    ) -> Result<Self, RendererError> {
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
        let requirements = renderer_config.gpu_canvases.device_requirements(
            adapter.features(),
            &adapter.limits(),
            renderer_config.profiling,
        )?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("argui-device"),
                required_features: requirements.features,
                required_limits: requirements.limits,
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
            initialization_fallback,
            generation: next_device_generation(),
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

    /// Returns retained and per-frame GPU-canvas cache statistics.
    #[must_use]
    pub fn gpu_canvas_stats(&self) -> crate::GpuCanvasStats {
        self.gpu_canvas.stats()
    }

    /// Drains recoverable GPU-canvas failure and recovery diagnostics.
    pub fn take_gpu_canvas_diagnostics(&mut self) -> Vec<crate::GpuCanvasDiagnostic> {
        self.gpu_canvas.take_diagnostics()
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
