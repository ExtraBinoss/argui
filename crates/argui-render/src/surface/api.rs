use super::*;

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
                let fallback_message = (!failures.is_empty())
                    .then(|| configure::fallback_message(&failures, renderer.label()));
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

        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let primary = Self::new_with_instance(
                target.clone(),
                width,
                height,
                renderer_config.clone(),
                configure::instance(),
                None,
            )
            .await;
            match primary {
                Ok(renderer) => Ok(renderer),
                Err(error)
                    if renderer_config.renderer_fallback
                        && matches!(
                            &error,
                            RendererError::SurfaceCreation(_)
                                | RendererError::AdapterRequest(_)
                                | RendererError::DeviceRequest(_)
                                | RendererError::UnsupportedSurface
                                | RendererError::UnsupportedSurfaceTransparency
                        ) =>
                {
                    let failures = [crate::RendererAttemptFailure {
                        renderer: "Vulkan".into(),
                        error: error.to_string(),
                    }];
                    Self::new_with_instance(
                        target,
                        width,
                        height,
                        renderer_config,
                        configure::fallback_instance(),
                        Some(configure::fallback_message(&failures, "OpenGL")),
                    )
                    .await
                    .map_err(|error| RendererError::Initialization {
                        attempts: vec![
                            failures[0].clone(),
                            crate::RendererAttemptFailure {
                                renderer: "OpenGL".into(),
                                error: error.to_string(),
                            },
                        ],
                        fallback_enabled: true,
                    })
                }
                Err(error) => Err(error),
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android")))]
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

    /// Adds or transactionally replaces a live custom-effect definition.
    ///
    /// Every WGSL pass and the complete parameter layout are prepared before the
    /// active registry and GPU pipelines are swapped. A rejected edit therefore
    /// leaves the prior working revision active. Existing definitions require a
    /// strictly greater revision; new identifiers begin at any non-zero revision.
    ///
    /// * `definition` — owned, revisioned effect schema and WGSL passes.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or parameter storage exceeds the
    /// active adapter limit.
    pub fn update_effect_definition(
        &mut self,
        definition: crate::EffectDefinition,
    ) -> Result<(), RendererError> {
        let registry = if self.renderer_config.effects.get(&definition.id).is_some() {
            self.renderer_config
                .effects
                .clone()
                .with_replacement(definition)?
        } else {
            self.renderer_config
                .effects
                .clone()
                .with_definition(definition)?
        };
        self.install_effect_registry(registry)
    }

    /// Removes a live effect definition and releases its pipelines.
    ///
    /// Parameter storage retains its high-water capacity so repeated schema edits
    /// never cause shrink/grow churn.
    ///
    /// * `id` — effect identifier to remove.
    ///
    /// # Errors
    ///
    /// Returns an error if rebuilding the remaining GPU effect set fails.
    pub fn remove_effect_definition(
        &mut self,
        id: &argui_paint::EffectId,
    ) -> Result<(), RendererError> {
        let registry = self.renderer_config.effects.clone().without_definition(id);
        self.install_effect_registry(registry)
    }

    /// Replaces the complete GPU effect registry after preparing every shader.
    ///
    /// `registry` contains the definitions to use for subsequent frames. The
    /// current pipelines remain active if any definition cannot be prepared.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry exceeds adapter limits or a WGSL pass
    /// fails to compile.
    pub fn replace_effect_registry(
        &mut self,
        registry: crate::EffectRegistry,
    ) -> Result<(), RendererError> {
        self.install_effect_registry(registry)
    }

    /// Prepares a complete GPU effect generation and commits it atomically.
    ///
    /// * `registry` — already validated next registry generation.
    ///
    /// # Errors
    ///
    /// Returns an error if storage limits or a shader pass are invalid.
    fn install_effect_registry(
        &mut self,
        registry: crate::EffectRegistry,
    ) -> Result<(), RendererError> {
        let parameter_words = registry
            .maximum_parameter_words()
            .max(self.effect.parameter_word_capacity());
        let maximum = self.device.limits().max_storage_buffer_binding_size as usize;
        let provided = parameter_words.saturating_mul(std::mem::size_of::<u32>());
        if provided > maximum {
            return Err(RendererError::EffectParametersTooLarge { provided, maximum });
        }
        let mut prepared =
            crate::effect::EffectGpu::new(&self.device, self.target_format, parameter_words);
        for definition in registry.definitions() {
            for (pass_index, pass) in definition.passes.iter().enumerate() {
                prepared.register(
                    &self.device,
                    definition.id.clone(),
                    definition.revision,
                    pass_index,
                    &pass.wgsl,
                )?;
            }
        }
        self.renderer_config.effects = registry;
        self.effect = prepared;
        self.layer_cache.clear();
        self.scene_snapshot = None;
        self.effect_root = None;
        self.damage.invalidate();
        Ok(())
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
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.surface_config);
        } else {
            self.offscreen_target = Some(offscreen_texture(&self.device, width, height));
        }
        self.layer_cache.clear();
        self.damage.invalidate();
        self.scene_snapshot = None;
        self.effect_root = None;
        true
    }

    /// Replaces the native surface target while retaining the existing GPU device.
    ///
    /// # Errors
    /// Returns an error for a windowless renderer or if surface creation fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn recreate_surface(
        &mut self,
        target: impl Into<SurfaceTarget<'static>>,
    ) -> Result<(), RendererError> {
        self.surface
            .as_ref()
            .ok_or(RendererError::UnsupportedSurface)?;
        self.surface = Some(
            self.instance
                .create_surface(target)
                .map_err(|error| RendererError::SurfaceCreation(error.to_string()))?,
        );
        self.offscreen_target = None;
        self.surface
            .as_ref()
            .expect("replacement surface exists")
            .configure(&self.device, &self.surface_config);
        self.layer_cache.clear();
        self.damage.invalidate();
        self.effect_root = None;
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

    /// Presents composition-only changes while reusing prepared primitive and text buffers.
    ///
    /// This method is valid after at least one successful [`Self::render_ui`] or
    /// [`Self::render_ui_notified`] call for the same retained display-list content.
    ///
    /// # Errors
    /// Returns a renderer error if the display list is invalid, a required effect
    /// is unavailable, or frame acquisition or rendering fails.
    ///
    /// * `display_list` — retained commands with updated compositor layers.
    /// * `scale_factor` — logical-to-physical scale used by the prepared content.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_composite(
        &mut self,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(
            FrameContent::Composite {
                display_list,
                scale_factor,
            },
            || {},
        )
    }

    /// Presents composition-only changes and invokes `notify` after submission.
    ///
    /// This method is valid after at least one successful [`Self::render_ui`] or
    /// [`Self::render_ui_notified`] call for the same retained display-list content.
    ///
    /// # Errors
    /// Returns a renderer error if the display list is invalid, a required effect
    /// is unavailable, or frame acquisition or rendering fails.
    ///
    /// * `display_list` — retained commands with updated compositor layers.
    /// * `scale_factor` — logical-to-physical scale used by the prepared content.
    /// * `notify` — callback invoked after the frame is submitted.
    pub fn render_composite_notified(
        &mut self,
        display_list: &DisplayList,
        scale_factor: f32,
        notify: impl FnOnce(),
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(
            FrameContent::Composite {
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

    /// Reconfigures adaptive damage tracking for subsequent frames.
    ///
    /// Changing `tracking` invalidates the retained surface so the next frame
    /// establishes a correct baseline before partial rendering resumes.
    pub fn set_damage_tracking(&mut self, tracking: crate::DamageTracking) {
        if self.renderer_config.damage_tracking == tracking {
            return;
        }
        self.renderer_config.damage_tracking = tracking;
        self.scene_snapshot = None;
        self.damage.invalidate();
        self.effect_root = None;
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
        self.image.register(&self.device, &self.queue, asset)?;
        self.layer_cache.clear();
        self.damage.invalidate();
        self.effect_root = None;
        Ok(())
    }

    /// Registers an SVG vector asset for rendering.
    ///
    /// # Errors
    /// Returns a renderer error if the asset cannot be rasterized or registered.
    pub fn register_vector(&mut self, asset: &VectorAsset) -> Result<(), RendererError> {
        self.vector.register(asset)?;
        self.layer_cache.clear();
        self.damage.invalidate();
        self.effect_root = None;
        Ok(())
    }
}
