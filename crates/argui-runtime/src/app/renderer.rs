use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(feature = "inspect")]
use argui_inspect::{AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord, Invalidation};
#[cfg(feature = "inspect")]
use argui_render::{AdapterProfile, DamageMode, GpuFrameProfile};
use argui_render::{
    EffectDefinition, EffectRegistry, GpuCanvasDiagnosticKind, RenderStatus, SurfaceAlphaMode,
    SurfaceRenderer,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{RuntimeEvent, app::Application};

use super::RendererState;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    /// Schedules a retained paint frame when `id` belongs to this window's registry.
    pub(crate) fn gpu_canvas_ready(&mut self, id: argui_paint::GpuCanvasId) {
        if self.renderer_config.gpu_canvases.get(id).is_none() {
            return;
        }
        self.pending_ui_frame.request_paint();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Synchronizes changed model assets and effects into the GPU surfaces.
    ///
    /// Returns whether resources changed and the frame should be recomputed.
    /// Only call during a requested model rebuild.
    ///
    /// # Errors
    ///
    /// Returns a renderer error if updated media or effects cannot be registered.
    pub(super) fn refresh_media_assets(&mut self) -> Result<bool, argui_render::RendererError> {
        let Some(model) = &self.model else {
            return Ok(false);
        };
        let images = model.image_assets();
        let vectors = model.vector_assets();
        let effects = model.effect_definitions();
        let media_changed = images != self.image_assets || vectors != self.vector_assets;
        let effects_changed = effects != self.effect_definitions;
        if !media_changed && !effects_changed {
            return Ok(false);
        }
        let next_registry = effects_changed
            .then(|| combine_effect_definitions(&self.base_effects, &effects))
            .transpose()?;
        if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
            for image in &images {
                if !self.image_assets.iter().any(|previous| previous == image) {
                    renderer.register_image(image)?;
                }
            }
            for vector in &vectors {
                if !self.vector_assets.iter().any(|previous| previous == vector) {
                    renderer.register_vector(vector)?;
                }
            }
            if let Some(registry) = &next_registry {
                renderer.replace_effect_registry(registry.clone())?;
            }
        }
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        if let Some(registry) = &next_registry {
            self.popups.replace_effect_registry(registry)?;
        }
        self.layout_engine.set_assets(&images, &vectors);
        self.image_assets = images;
        self.vector_assets = vectors;
        self.effect_definitions = effects;
        if let Some(registry) = next_registry {
            self.renderer_config.effects = registry;
        }
        Ok(true)
    }

    /// Replaces JavaScript-owned custom effects on the active window.
    ///
    /// `effects` is the validated base registry. Model effects, when present,
    /// are retained. The active GPU pipelines are prepared before the stored
    /// registry is changed, so a preparation failure keeps the previous set.
    ///
    /// # Errors
    /// Returns a renderer error for conflicting definitions, GPU limits, or
    /// a WGSL pipeline that cannot be prepared.
    pub(crate) fn replace_native_effects(
        &mut self,
        effects: EffectRegistry,
    ) -> Result<(), argui_render::RendererError> {
        let complete = combine_effect_definitions(&effects, &self.effect_definitions)?;
        if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
            renderer.replace_effect_registry(complete.clone())?;
        }
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        if let Err(error) = self.popups.replace_effect_registry(&complete) {
            let previous = self.renderer_config.effects.clone();
            let _ = self.popups.replace_effect_registry(&previous);
            if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
                let _ = renderer.replace_effect_registry(previous);
            }
            return Err(error);
        }
        self.base_effects = effects;
        self.renderer_config.effects = complete;
        self.pending_ui_frame.request_paint();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Ok(())
    }

    /// Changes damage tracking for the main surface and any existing native popups.
    ///
    /// `tracking` controls whether subsequent frames may reuse retained pixels.
    pub(crate) fn set_damage_tracking(&mut self, tracking: argui_render::DamageTracking) {
        if self.renderer_config.damage_tracking == tracking {
            return;
        }
        self.renderer_config.damage_tracking = tracking;
        if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
            renderer.set_damage_tracking(tracking);
        }
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        self.popups.set_damage_tracking(tracking);
        self.pending_ui_frame.request_paint();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Changes whether this window emits renderer profiling events.
    ///
    /// `enabled` is combined with DevTools' independent GPU profiling request.
    pub(crate) fn set_renderer_profiling(&mut self, enabled: bool) {
        if enabled && !self.renderer_config.profiling {
            (self.on_event)(RuntimeEvent::CommandFailed(
                "renderer profiling was not enabled when this window was initialized".into(),
            ));
            return;
        }
        if self.renderer_profiling_requested == enabled {
            return;
        }
        self.renderer_profiling_requested = enabled;
        self.pending_ui_frame.request_paint();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub(super) fn surface_renderer_config(&self) -> argui_render::RendererConfig {
        self.renderer_config
            .clone()
            .surface_alpha(if self.window_config.transparent {
                SurfaceAlphaMode::Transparent
            } else if self.window_config.desktop_backdrop.is_some() {
                SurfaceAlphaMode::PreferTransparent
            } else {
                SurfaceAlphaMode::Opaque
            })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn initialize_renderer(
        &mut self,
        window: &Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) {
        let size = crate::host::WindowHost::drawable_size(window);
        let shared = self.renderer_device.borrow().clone();
        let config = self.surface_renderer_config();
        let renderer = shared.map_or_else(
            || {
                pollster::block_on(SurfaceRenderer::new(
                    Arc::clone(window),
                    size.width,
                    size.height,
                    config.clone(),
                ))
            },
            |device| {
                pollster::block_on(SurfaceRenderer::new_with_device(
                    Arc::clone(window),
                    size.width,
                    size.height,
                    config.clone(),
                    device,
                ))
            },
        );
        self.install_renderer(renderer, window, event_loop);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn install_renderer(
        &mut self,
        renderer: Result<SurfaceRenderer, argui_render::RendererError>,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        match renderer {
            Ok(mut renderer) => {
                if let Some(message) = renderer.initialization_fallback() {
                    (self.on_event)(RuntimeEvent::RendererFallback(message.into()));
                }
                #[cfg(all(feature = "desktop-backdrop", not(target_arch = "wasm32")))]
                if self.window_config.desktop_backdrop.is_some()
                    && (renderer.initialization_fallback().is_some() || !renderer.is_transparent())
                {
                    self.desktop_backdrop = None;
                    self.environment.desktop_backdrop_available = false;
                    self.pending_ui_frame.request_rebuild();
                    (self.on_event)(RuntimeEvent::DesktopBackdropUnavailable(
                        if renderer.initialization_fallback().is_some() {
                            "the renderer selected a compatibility fallback without Windows DirectComposition support"
                                .into()
                        } else {
                            "the GPU surface is opaque".into()
                        },
                    ));
                }
                if self.renderer_device.borrow().is_none() {
                    *self.renderer_device.borrow_mut() = Some(renderer.device_handle());
                }
                if let Err(error) = register_images(&mut renderer, &self.image_assets) {
                    (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                    self.fatal_error = Some(error.into());
                    event_loop.exit();
                    return;
                }
                if let Err(error) = register_vectors(&mut renderer, &self.vector_assets) {
                    (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                    self.fatal_error = Some(error.into());
                    event_loop.exit();
                    return;
                }
                let effects = if self.effect_definitions.is_empty() {
                    Ok(self.base_effects.clone())
                } else {
                    combine_effect_definitions(&self.base_effects, &self.effect_definitions)
                        .and_then(|registry| {
                            renderer.replace_effect_registry(registry.clone())?;
                            Ok(registry)
                        })
                };
                let registry = match effects {
                    Ok(registry) => registry,
                    Err(error) => {
                        (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                        self.fatal_error = Some(error.into());
                        event_loop.exit();
                        return;
                    }
                };
                self.renderer_config.effects = registry;
                *self.renderer.borrow_mut() = RendererState::Ready(Box::new(renderer));
                window.request_redraw();
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                self.fatal_error = Some(error.into());
                event_loop.exit();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn initialize_renderer(
        &mut self,
        window: &Arc<Window>,
        _event_loop: &ActiveEventLoop,
    ) {
        let size = crate::host::WindowHost::drawable_size(window);
        let window = Arc::clone(window);
        let renderer = Rc::clone(&self.renderer);
        let renderer_device = Rc::clone(&self.renderer_device);
        let config = self.surface_renderer_config();
        let image_assets = self.image_assets.clone();
        let vector_assets = self.vector_assets.clone();
        let base_effects = self.base_effects.clone();
        let model = self.model.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let shared = renderer_device.borrow().clone();
            let result = if let Some(device) = shared {
                SurfaceRenderer::new_with_device(
                    Arc::clone(&window),
                    size.width,
                    size.height,
                    config,
                    device,
                )
                .await
            } else {
                SurfaceRenderer::new(Arc::clone(&window), size.width, size.height, config).await
            };
            let result = match result {
                Ok(mut surface) => {
                    if renderer_device.borrow().is_none() {
                        *renderer_device.borrow_mut() = Some(surface.device_handle());
                    }
                    let current_size = crate::host::WindowHost::drawable_size(&window);
                    surface.resize(current_size.width, current_size.height);
                    register_images(&mut surface, &image_assets).and_then(|()| {
                        register_vectors(&mut surface, &vector_assets)?;
                        let definitions = model
                            .as_ref()
                            .map(crate::AnyEntity::effect_definitions)
                            .unwrap_or_default();
                        if !definitions.is_empty() {
                            let registry = combine_effect_definitions(&base_effects, &definitions)?;
                            surface.replace_effect_registry(registry)?;
                        }
                        Ok(surface)
                    })
                }
                Err(error) => Err(error),
            };
            *renderer.borrow_mut() = match result {
                Ok(renderer) => RendererState::Ready(Box::new(renderer)),
                Err(error) => RendererState::Failed(error.to_string()),
            };
            window.request_redraw();
        });
    }

    pub(super) fn render(&mut self, event_loop: &dyn crate::host::LoopControl) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let mut state = self.renderer.borrow_mut();

        #[cfg(target_arch = "wasm32")]
        if let RendererState::Failed(message) = &*state {
            (self.on_event)(RuntimeEvent::RendererFailed(message.clone()));
            event_loop.exit();
            return;
        }
        let RendererState::Ready(renderer) = &mut *state else {
            return;
        };
        if !self.renderer_announced {
            (self.on_event)(RuntimeEvent::RendererReady);
            self.renderer_announced = true;
        }

        // WebAssembly initializes the renderer asynchronously, so a command can
        // update the requested policy while the surface is still loading.
        renderer.set_damage_tracking(self.renderer_config.damage_tracking);
        let profiling_active = self.renderer_profiling_requested;
        #[cfg(feature = "inspect")]
        let profiling_active = profiling_active
            || self
                .inspector
                .as_ref()
                .is_some_and(argui_inspect::InspectorHandle::gpu_profiling);
        renderer.set_profiling_active(profiling_active);

        #[cfg(feature = "inspect")]
        if let Some(inspector) = &self.inspector {
            inspector.record_ui(FrameRecord {
                interval: self.frame_record.interval,
                model: self.frame_record.model,
                tree: self.frame_record.tree,
                layout: self.frame_record.layout,
                paint: self.frame_record.paint,
                surface: self.frame_record.surface,
                resize_events: self.frame_record.resize_events,
                update: match self.frame_record.update {
                    argui_ui::TreeUpdate::None | argui_ui::TreeUpdate::Semantics => {
                        Invalidation::None
                    }
                    argui_ui::TreeUpdate::Composite => Invalidation::Composite,
                    argui_ui::TreeUpdate::Paint | argui_ui::TreeUpdate::Scroll => {
                        Invalidation::Paint
                    }
                    argui_ui::TreeUpdate::Layout => Invalidation::Layout,
                },
                ..FrameRecord::default()
            });
        }
        if self.renderer_profiling_requested {
            (self.on_event)(RuntimeEvent::AnimationProfile(crate::AnimationProfile {
                frame_interval: self.frame_record.interval,
                model_time: self.frame_record.model,
                tree_time: self.frame_record.tree + self.frame_record.layout,
                paint_time: self.frame_record.paint + self.frame_record.surface,
                tree_update: self.frame_record.update,
            }));
        }

        let rendered = match (self.prepared_text.as_ref(), self.ui_layout.as_ref()) {
            (_, Some(layout)) if self.composite_frame => {
                renderer.render_composite_notified(&layout.display_list, self.scale_factor, || {
                    window.pre_present_notify()
                })
            }
            (Some(text), Some(layout)) => renderer.render_ui_notified(
                &mut self.text_engine,
                text,
                &layout.display_list,
                self.scale_factor,
                || window.pre_present_notify(),
            ),
            (Some(text), None) => {
                renderer.render_text_notified(&mut self.text_engine, text, || {
                    window.pre_present_notify()
                })
            }
            (None, _) => renderer.render_notified(|| window.pre_present_notify()),
        };
        for diagnostic in renderer.take_gpu_canvas_diagnostics() {
            let event = match diagnostic.kind {
                GpuCanvasDiagnosticKind::Failed => RuntimeEvent::GpuCanvasFailed(diagnostic),
                GpuCanvasDiagnosticKind::Recovered => RuntimeEvent::GpuCanvasRecovered(diagnostic),
            };
            (self.on_event)(event);
        }
        let result = match rendered {
            Ok(RenderStatus::Presented | RenderStatus::Skipped) => {
                #[cfg(feature = "inspect")]
                if let Some(inspector) = &self.inspector {
                    let profile = renderer.last_profile();
                    inspector.record_render(FrameRecord {
                        render_cpu: profile.cpu_time,
                        layers: profile.effects.offscreen_layers,
                        passes: profile.effects.filter_passes,
                        offscreen_pixels: profile.effects.offscreen_pixels,
                        cached_layers: profile.effects.cached_layers,
                        damaged_pixels: profile.damage.damaged_pixels,
                        textures: profile.texture_pool.textures
                            + 2
                            + profile.gpu_canvases.entries
                            + usize::from(profile.damage.retained_bytes > 0)
                            + 1,
                        reused_textures: profile.texture_pool.reused_this_frame
                            + profile.gpu_canvases.hits_this_frame
                            + usize::from(matches!(
                                profile.damage.mode,
                                DamageMode::Partial | DamageMode::Reused
                            )),
                        texture_bytes: profile.texture_pool.allocated_bytes
                            + profile.text_atlas.allocated_bytes
                            + profile.vector_atlas.allocated_bytes
                            + profile.gpu_canvases.allocated_bytes
                            + profile.damage.retained_bytes,
                        gpu_canvas_entries: profile.gpu_canvases.entries,
                        gpu_canvas_bytes: profile.gpu_canvases.allocated_bytes,
                        gpu_canvas_renders: profile.gpu_canvases.renders_this_frame,
                        gpu_canvas_hits: profile.gpu_canvases.hits_this_frame,
                        gpu_canvas_failures: profile.gpu_canvases.failures_this_frame,
                        gpu_canvas_encode_cpu: profile.gpu_canvases.encode_time,
                        text_atlas_bytes: profile.text_atlas.allocated_bytes,
                        text_atlas_entries: profile.text_atlas.entries,
                        text_atlas_hits: profile.text_atlas.hits_this_frame,
                        text_raster_requests: profile.text_atlas.raster_requests_this_frame,
                        text_upload_bytes: profile.text_atlas.uploaded_bytes_this_frame,
                        text_page_evictions: profile.text_atlas.evictions_this_frame,
                        vector_atlas_entries: profile.vector_atlas.entries,
                        vector_atlas_bytes: profile.vector_atlas.allocated_bytes,
                        vector_atlas_hits: profile.vector_atlas.hits_this_frame,
                        vector_rasterizations: profile.vector_atlas.rasterizations_this_frame,
                        adapter: adapter_record(&profile.adapter),
                        gpu: profile.gpu.as_ref().map(gpu_record),
                        ..FrameRecord::default()
                    });
                }
                if self.renderer_profiling_requested {
                    (self.on_event)(RuntimeEvent::RenderProfile(Box::new(
                        renderer.last_profile(),
                    )));
                }
                Ok(())
            }
            Ok(RenderStatus::Reconfigure) => {
                let size = crate::host::WindowHost::drawable_size(&window);
                renderer.resize(size.width, size.height);
                window.request_redraw();
                Ok(())
            }
            Ok(RenderStatus::RecreateSurface) => window
                .recreate_surface(renderer)
                .inspect(|()| window.request_redraw()),
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
            self.fatal_error = Some(error.into());
            event_loop.exit();
        }
    }
}

#[cfg(feature = "inspect")]
fn adapter_record(profile: &AdapterProfile) -> AdapterRecord {
    AdapterRecord {
        name: profile.name.clone(),
        vendor: profile.vendor,
        device: profile.device,
        device_type: profile.device_type.clone(),
        driver: profile.driver.clone(),
        driver_info: profile.driver_info.clone(),
        backend: profile.backend.clone(),
        features: profile.features.clone(),
        timestamp_queries: profile.timestamp_queries,
        max_texture_dimension_2d: profile.max_texture_dimension_2d,
        max_buffer_size: profile.max_buffer_size,
        max_storage_buffer_binding_size: profile.max_storage_buffer_binding_size,
        max_bind_groups: profile.max_bind_groups,
    }
}

#[cfg(feature = "inspect")]
fn gpu_record(profile: &GpuFrameProfile) -> GpuFrameRecord {
    GpuFrameRecord {
        sequence: profile.frame,
        total: profile.total,
        passes: profile
            .passes
            .iter()
            .map(|pass| GpuPassRecord {
                label: pass.label.clone(),
                start: pass.start,
                duration: pass.duration,
                pixels: pass.pixels,
                object_domain: pass.object.map(|object| format!("{:?}", object.domain)),
                object_id: pass.object.map(|object| object.value),
            })
            .collect(),
    }
}

fn register_vectors(
    renderer: &mut SurfaceRenderer,
    assets: &[argui_paint::VectorAsset],
) -> Result<(), argui_render::RendererError> {
    for asset in assets {
        renderer.register_vector(asset)?;
    }
    Ok(())
}

fn register_images(
    renderer: &mut SurfaceRenderer,
    images: &[argui_paint::ImageAsset],
) -> Result<(), argui_render::RendererError> {
    for image in images {
        renderer.register_image(image)?;
    }
    Ok(())
}

/// Combines configured effects with the model's current definitions.
///
/// `base` contains definitions supplied by renderer configuration, and
/// `definitions` contains current model-owned definitions. Returns a complete
/// validated registry, or an error for an invalid or duplicate definition.
fn combine_effect_definitions(
    base: &EffectRegistry,
    definitions: &[EffectDefinition],
) -> Result<EffectRegistry, argui_render::RendererError> {
    definitions
        .iter()
        .cloned()
        .try_fold(base.clone(), EffectRegistry::with_definition)
}
