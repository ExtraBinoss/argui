use argui_paint::{DisplayList, Filter, LayerStyle, ShaderEffectId};
use argui_text::{PreparedText, TextEngine};
use wgpu::{
    CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, SurfaceTarget, TextureFormat, TextureViewDescriptor,
};

use crate::{
    RendererConfig, RendererError,
    batch::{DrawBatch, DrawKind, build_batches},
    effect::{
        EffectDraw, EffectGpu, EffectUniform, blend_mode, layer_radii, uniform as effect_uniform,
    },
    effect_graph::{EffectGraph, EffectNode},
    offscreen::{TexturePool, TexturePoolStats},
    quad::QuadGpu,
    text::TextGpu,
};

mod effects;
use effects::EffectPass;

enum FrameContent<'a> {
    None,
    Text {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
    },
    Ui {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
        display_list: &'a DisplayList,
        scale_factor: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStatus {
    Presented,
    Reconfigure,
    RecreateSurface,
    Skipped,
}

pub struct SurfaceRenderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    target_format: TextureFormat,
    renderer_config: RendererConfig,
    batches: Vec<DrawBatch>,
    quad: QuadGpu,
    text: TextGpu,
    effect: EffectGpu,
    offscreen: TexturePool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
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
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("argui-device"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .map_err(|error| RendererError::DeviceRequest(error.to_string()))?;
        let mut surface_config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(RendererError::UnsupportedSurface)?;
        surface_config.present_mode = renderer_config.present_mode;
        let target_format = srgb_target(surface_config.format);
        if target_format != surface_config.format {
            surface_config.view_formats.push(target_format);
        }
        surface.configure(&device, &surface_config);
        let quad = QuadGpu::new(&device, target_format);
        let text = TextGpu::new(&device, target_format);
        let effect = EffectGpu::new(&device, target_format);
        let offscreen = TexturePool::new(target_format, 128 * 1024 * 1024);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            surface_config,
            target_format,
            renderer_config,
            batches: Vec::new(),
            quad,
            text,
            effect,
            offscreen,
        })
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        let Some((width, height)) = drawable_size(width, height) else {
            return false;
        };
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        true
    }

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
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render(&mut self) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::None)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_text(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Text { engine, text })
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn render_ui(
        &mut self,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<RenderStatus, RendererError> {
        self.render_frame(FrameContent::Ui {
            engine,
            text,
            display_list,
            scale_factor,
        })
    }

    #[must_use]
    pub fn texture_pool_stats(&self) -> TexturePoolStats {
        self.offscreen.stats()
    }

    pub fn register_effect_shader(
        &mut self,
        id: ShaderEffectId,
        wgsl: &str,
    ) -> Result<(), RendererError> {
        self.effect.register(&self.device, id, wgsl)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn render_frame(&mut self, content: FrameContent<'_>) -> Result<RenderStatus, RendererError> {
        let (frame, status) = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => (frame, RenderStatus::Presented),
            CurrentSurfaceTexture::Suboptimal(frame) => (frame, RenderStatus::Reconfigure),
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                return Ok(RenderStatus::Skipped);
            }
            CurrentSurfaceTexture::Outdated => return Ok(RenderStatus::Reconfigure),
            CurrentSurfaceTexture::Lost => return Ok(RenderStatus::RecreateSurface),
            CurrentSurfaceTexture::Validation => return Err(RendererError::Validation),
        };

        let viewport = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
        let mut effect_graph = None;
        match content {
            FrameContent::None => self.batches.clear(),
            FrameContent::Text { engine, text } => {
                let draw = self
                    .text
                    .prepare(&self.device, &self.queue, engine, text, viewport)?;
                let range = draw.all();
                self.batches.clear();
                if !range.is_empty() {
                    self.batches.push(DrawBatch {
                        kind: DrawKind::Text,
                        instances: range,
                    });
                }
            }
            FrameContent::Ui {
                engine,
                text,
                display_list,
                scale_factor,
            } => {
                self.quad.prepare(
                    &self.device,
                    &self.queue,
                    display_list,
                    viewport,
                    scale_factor,
                );
                let draw = self
                    .text
                    .prepare(&self.device, &self.queue, engine, text, viewport)?;
                build_batches(display_list, draw.ranges(), &mut self.batches);
                let graph = EffectGraph::build(display_list, draw.ranges(), scale_factor)
                    .map_err(|error| RendererError::InvalidDisplayList(error.to_string()))?;
                self.validate_custom_effects(&graph)?;
                effect_graph = Some(graph);
            }
        }
        let view = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(self.target_format),
            ..Default::default()
        });
        let attachment = Some(RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(self.renderer_config.wgpu_clear_color()),
                store: StoreOp::Store,
            },
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        if let Some(graph) = effect_graph
            && graph.needs_offscreen_root()
        {
            self.render_effect_graph(&mut encoder, &view, &graph, viewport);
            self.queue.submit([encoder.finish()]);
            self.queue.present(frame);
            return Ok(status);
        }
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("argui-clear-pass"),
                color_attachments: &[attachment],
                ..Default::default()
            });
            for batch in &self.batches {
                match batch.kind {
                    DrawKind::Quad => self.quad.draw(&mut pass, batch.instances.clone()),
                    DrawKind::Text => self.text.draw(&mut pass, batch.instances.clone()),
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        Ok(status)
    }

    fn render_effect_graph(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        graph: &EffectGraph,
        viewport: [f32; 2],
    ) {
        self.offscreen.begin_frame();
        self.effect.begin_frame();
        let width = viewport[0] as u32;
        let height = viewport[1] as u32;
        let root = self.offscreen.acquire(&self.device, width, height);
        self.clear_target(encoder, root, self.renderer_config.wgpu_clear_color());
        self.render_effect_nodes(encoder, &graph.roots, root, viewport);
        let surface_attachment = Some(RenderPassColorAttachment {
            view: surface,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(self.renderer_config.wgpu_clear_color()),
                store: StoreOp::Store,
            },
        });
        drop(encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("argui-effect-surface-clear"),
            color_attachments: &[surface_attachment],
            ..Default::default()
        }));
        let uniform = EffectUniform {
            viewport,
            mode: 99,
            data: [1.0, 0.0, 0.0, 0.0],
            ..EffectUniform::default()
        };
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: surface,
                source: self.offscreen.view(root),
                backdrop: self.offscreen.view(root),
                uniform,
                shader: None,
            },
        );
    }

    fn render_effect_nodes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        nodes: &[EffectNode],
        target: usize,
        viewport: [f32; 2],
    ) {
        for node in nodes {
            match node {
                EffectNode::Draw(batch) => self.draw_offscreen(encoder, target, batch, viewport),
                EffectNode::Layer(layer) if !layer.style.requires_offscreen() => {
                    self.render_effect_nodes(encoder, &layer.children, target, viewport);
                }
                EffectNode::Layer(layer) => {
                    let layer_target = self.offscreen.acquire(
                        &self.device,
                        viewport[0] as u32,
                        viewport[1] as u32,
                    );
                    self.clear_target(encoder, layer_target, wgpu::Color::TRANSPARENT);
                    self.render_effect_nodes(encoder, &layer.children, layer_target, viewport);
                    let foreground = self.apply_filters(
                        encoder,
                        layer_target,
                        &layer.style.filters,
                        viewport,
                        layer.style.expanded_bounds(),
                    );
                    self.composite_layer(encoder, target, foreground, &layer.style, viewport);
                }
            }
        }
    }

    fn draw_offscreen(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: usize,
        batch: &DrawBatch,
        viewport: [f32; 2],
    ) {
        let attachment = Some(RenderPassColorAttachment {
            view: self.offscreen.view(target),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("argui-layer-content"),
            color_attachments: &[attachment],
            ..Default::default()
        });
        pass.set_viewport(0.0, 0.0, viewport[0], viewport[1], 0.0, 1.0);
        match batch.kind {
            DrawKind::Quad => self.quad.draw(&mut pass, batch.instances.clone()),
            DrawKind::Text => self.text.draw(&mut pass, batch.instances.clone()),
        }
    }

    fn clear_target(&self, encoder: &mut wgpu::CommandEncoder, target: usize, color: wgpu::Color) {
        let attachment = Some(RenderPassColorAttachment {
            view: self.offscreen.view(target),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(color),
                store: StoreOp::Store,
            },
        });
        drop(encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("argui-layer-clear"),
            color_attachments: &[attachment],
            ..Default::default()
        }));
    }

    fn effect_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: usize,
        viewport: [f32; 2],
        effect_pass: EffectPass,
    ) -> usize {
        let size = effect_pass
            .target_size
            .unwrap_or([viewport[0] as u32, viewport[1] as u32]);
        let target = self.offscreen.acquire(&self.device, size[0], size[1]);
        self.clear_target(encoder, target, wgpu::Color::TRANSPARENT);
        let mut uniform = effect_uniform(viewport, effect_pass.bounds);
        uniform.mode = effect_pass.mode;
        uniform.data = effect_pass.data;
        if let Some(matrix) = effect_pass.matrix {
            let rows = matrix.as_chunks::<4>().0;
            uniform.matrix.copy_from_slice(rows);
        }
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: self.offscreen.view(target),
                source: self.offscreen.view(source),
                backdrop: self.offscreen.view(source),
                uniform,
                shader: effect_pass.shader,
            },
        );
        target
    }

    fn composite_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: usize,
        foreground: usize,
        style: &LayerStyle,
        viewport: [f32; 2],
    ) {
        let snapshot = self.snapshot(encoder, target, viewport);
        let filtered_backdrop = self.apply_filters(
            encoder,
            snapshot,
            &style.backdrop_filters,
            viewport,
            style.bounds,
        );
        let mut backdrop = if style.backdrop_filters.is_empty() {
            snapshot
        } else {
            let merged =
                self.offscreen
                    .acquire(&self.device, viewport[0] as u32, viewport[1] as u32);
            self.clear_target(encoder, merged, wgpu::Color::TRANSPARENT);
            let mut uniform = effect_uniform(viewport, style.bounds);
            uniform.mode = 12;
            uniform.radii = layer_radii(style.mask);
            self.effect.draw(
                &self.device,
                &self.queue,
                encoder,
                EffectDraw {
                    target: self.offscreen.view(merged),
                    source: self.offscreen.view(filtered_backdrop),
                    backdrop: self.offscreen.view(snapshot),
                    uniform,
                    shader: None,
                },
            );
            merged
        };
        for shadow in &style.shadows {
            let blurred = self.apply_filters(
                encoder,
                foreground,
                &[Filter::Blur(shadow.blur)],
                viewport,
                style.expanded_bounds(),
            );
            let shadowed =
                self.offscreen
                    .acquire(&self.device, viewport[0] as u32, viewport[1] as u32);
            self.clear_target(encoder, shadowed, wgpu::Color::TRANSPARENT);
            let mut uniform = effect_uniform(viewport, style.bounds);
            uniform.mode = if shadow.inset { 11 } else { 10 };
            uniform.color = shadow.color.as_array();
            uniform.data = [1.0, shadow.offset[0], shadow.offset[1], shadow.spread];
            uniform.radii = layer_radii(style.mask);
            self.effect.draw(
                &self.device,
                &self.queue,
                encoder,
                EffectDraw {
                    target: self.offscreen.view(shadowed),
                    source: self.offscreen.view(blurred),
                    backdrop: self.offscreen.view(backdrop),
                    uniform,
                    shader: None,
                },
            );
            backdrop = shadowed;
        }
        let mut uniform = effect_uniform(viewport, style.bounds);
        uniform.blend = blend_mode(style.blend_mode);
        uniform.data[0] = style.opacity;
        uniform.radii = layer_radii(style.mask);
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: self.offscreen.view(target),
                source: self.offscreen.view(foreground),
                backdrop: self.offscreen.view(backdrop),
                uniform,
                shader: None,
            },
        );
    }

    fn snapshot(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: usize,
        viewport: [f32; 2],
    ) -> usize {
        let target = self
            .offscreen
            .acquire(&self.device, viewport[0] as u32, viewport[1] as u32);
        encoder.copy_texture_to_texture(
            self.offscreen.texture(source).as_image_copy(),
            self.offscreen.texture(target).as_image_copy(),
            wgpu::Extent3d {
                width: viewport[0] as u32,
                height: viewport[1] as u32,
                depth_or_array_layers: 1,
            },
        );
        target
    }
}

const fn drawable_size(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

fn srgb_target(format: TextureFormat) -> TextureFormat {
    format.add_srgb_suffix()
}

#[cfg(test)]
mod tests {
    use super::{drawable_size, srgb_target};

    #[test]
    fn zero_sized_surfaces_are_not_configured() {
        assert_eq!(drawable_size(800, 600), Some((800, 600)));
        assert_eq!(drawable_size(0, 600), None);
        assert_eq!(drawable_size(800, 0), None);
    }

    #[test]
    fn presentation_uses_an_srgb_view_when_the_surface_has_one() {
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb
        );
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Rgba8UnormSrgb),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            srgb_target(wgpu::TextureFormat::Rgba16Float),
            wgpu::TextureFormat::Rgba16Float
        );
    }
}
