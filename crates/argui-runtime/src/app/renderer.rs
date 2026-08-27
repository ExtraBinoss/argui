use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use argui_inspect::FrameRecord;
use argui_render::{EffectShader, RenderStatus, SurfaceRenderer};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{RuntimeEvent, app::Application};

use super::RendererState;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn initialize_renderer(
        &mut self,
        window: &Arc<Window>,
        event_loop: &ActiveEventLoop,
    ) {
        let size = window.inner_size();
        match pollster::block_on(SurfaceRenderer::new(
            Arc::clone(window),
            size.width,
            size.height,
            self.renderer_config,
        )) {
            Ok(mut renderer) => {
                if let Err(error) = register_effect_shaders(&mut renderer, &self.effect_shaders) {
                    (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                    self.fatal_error = Some(error.into());
                    event_loop.exit();
                    return;
                }
                if let Err(error) = register_images(&mut renderer, &self.image_assets) {
                    (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                    self.fatal_error = Some(error.into());
                    event_loop.exit();
                    return;
                }
                register_vectors(&mut renderer, &self.vector_assets);
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
        let config = self.renderer_config;
        let effect_shaders = self.effect_shaders.clone();
        let image_assets = self.image_assets.clone();
        let vector_assets = self.vector_assets.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result =
                SurfaceRenderer::new(Arc::clone(&window), size.width, size.height, config).await;
            let result = match result {
                Ok(mut surface) => {
                    let current_size = window.inner_size();
                    surface.resize(current_size.width, current_size.height);
                    register_effect_shaders(&mut surface, &effect_shaders)
                        .and_then(|()| register_images(&mut surface, &image_assets))
                        .map(|()| {
                            register_vectors(&mut surface, &vector_assets);
                            surface
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

        let rendered = match (self.prepared_text.as_ref(), self.ui_layout.as_ref()) {
            (Some(text), Some(layout)) => renderer.render_ui(
                &mut self.text_engine,
                text,
                &layout.display_list,
                self.scale_factor,
            ),
            (Some(text), None) => renderer.render_text(&mut self.text_engine, text),
            (None, _) => renderer.render(),
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
                        textures: profile.texture_pool.textures,
                        reused_textures: profile.texture_pool.reused_this_frame,
                        texture_bytes: profile.texture_pool.allocated_bytes,
                        ..FrameRecord::default()
                    });
                }
                if self.renderer_config.profiling {
                    (self.on_event)(RuntimeEvent::RenderProfile(renderer.last_profile()));
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

fn register_vectors(renderer: &mut SurfaceRenderer, assets: &[argui_paint::VectorAsset]) {
    for asset in assets {
        renderer.register_vector(asset);
    }
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

fn register_effect_shaders(
    renderer: &mut SurfaceRenderer,
    shaders: &[EffectShader],
) -> Result<(), argui_render::RendererError> {
    for shader in shaders {
        renderer.register_effect_shader(shader.id, shader.wgsl)?;
    }
    Ok(())
}
