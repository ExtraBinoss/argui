use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use argui_inspect::{AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord, Invalidation};
use argui_render::{
    AdapterProfile, GpuFrameProfile, RenderStatus, SurfaceAlphaMode, SurfaceRenderer,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{RuntimeEvent, app::Application};

use super::RendererState;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    fn surface_renderer_config(&self) -> argui_render::RendererConfig {
        self.renderer_config
            .clone()
            .surface_alpha(if self.window_config.transparent {
                SurfaceAlphaMode::Transparent
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
        let size = window.inner_size();
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
        match renderer {
            Ok(mut renderer) => {
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
        let size = window.inner_size();
        let window = Arc::clone(window);
        let renderer = Rc::clone(&self.renderer);
        let renderer_device = Rc::clone(&self.renderer_device);
        let config = self.surface_renderer_config();
        let image_assets = self.image_assets.clone();
        let vector_assets = self.vector_assets.clone();

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
                    let current_size = window.inner_size();
                    surface.resize(current_size.width, current_size.height);
                    register_images(&mut surface, &image_assets).and_then(|()| {
                        register_vectors(&mut surface, &vector_assets)?;
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

    pub(super) fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
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

        renderer.set_profiling_active(self.inspector.as_ref().map_or(
            self.renderer_config.profiling,
            argui_inspect::InspectorHandle::recording,
        ));

        if let Some(inspector) = &self.inspector {
            inspector.record_ui(self.frame_record.clone());
        }
        if self.renderer_config.profiling {
            (self.on_event)(RuntimeEvent::AnimationProfile(crate::AnimationProfile {
                frame_interval: self.frame_record.interval,
                model_time: self.frame_record.model,
                tree_time: self.frame_record.tree + self.frame_record.layout,
                paint_time: self.frame_record.paint + self.frame_record.surface,
                tree_update: match self.frame_record.update {
                    Invalidation::None => argui_ui::TreeUpdate::None,
                    Invalidation::Paint => argui_ui::TreeUpdate::Paint,
                    Invalidation::Layout => argui_ui::TreeUpdate::Layout,
                },
            }));
        }

        let rendered = match (self.prepared_text.as_ref(), self.ui_layout.as_ref()) {
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
        let result = match rendered {
            Ok(RenderStatus::Presented | RenderStatus::Skipped) => {
                if let Some(inspector) = &self.inspector {
                    let profile = renderer.last_profile();
                    inspector.record_render(FrameRecord {
                        render_cpu: profile.cpu_time,
                        layers: profile.effects.offscreen_layers,
                        passes: profile.effects.filter_passes,
                        offscreen_pixels: profile.effects.offscreen_pixels,
                        cached_layers: profile.effects.cached_layers,
                        damaged_pixels: profile.effects.damaged_pixels,
                        textures: profile.texture_pool.textures + 1,
                        reused_textures: profile.texture_pool.reused_this_frame,
                        texture_bytes: profile.texture_pool.allocated_bytes
                            + profile.vector_atlas.allocated_bytes,
                        vector_atlas_entries: profile.vector_atlas.entries,
                        vector_atlas_hits: profile.vector_atlas.hits_this_frame,
                        vector_rasterizations: profile.vector_atlas.rasterizations_this_frame,
                        adapter: adapter_record(&profile.adapter),
                        gpu: profile.gpu.as_ref().map(gpu_record),
                        ..FrameRecord::default()
                    });
                }
                if self.renderer_config.profiling {
                    (self.on_event)(RuntimeEvent::RenderProfile(Box::new(
                        renderer.last_profile(),
                    )));
                }
                Ok(())
            }
            Ok(RenderStatus::Reconfigure) => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
                window.request_redraw();
                Ok(())
            }
            Ok(RenderStatus::RecreateSurface) => renderer
                .recreate_surface(Arc::clone(&window))
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
