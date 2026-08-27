use argui_core::Rect;
use argui_paint::{Filter, LayerMask, LayerStyle, ShaderEffectId};
use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{
    RendererError,
    batch::DrawKind,
    effect::{EffectDraw, EffectUniform, blend_mode, layer_radii, uniform},
    effect_graph::{EffectGraph, EffectNode},
    effect_plan::{PlannedFilter, plan_filters},
    target::{PixelRegion, TextureTarget},
};

use super::SurfaceRenderer;

struct EffectPass {
    mode: u32,
    data: [f32; 4],
    matrix: Option<[f32; 20]>,
    bounds: Rect,
    shader: Option<ShaderEffectId>,
    extent: Option<[u32; 2]>,
    radii: [f32; 4],
}

#[derive(Clone, Copy)]
struct EffectSources {
    source: TextureTarget,
    backdrop: TextureTarget,
}

impl EffectPass {
    const fn new(mode: u32, data: [f32; 4], bounds: Rect) -> Self {
        Self {
            mode,
            data,
            matrix: None,
            bounds,
            shader: None,
            extent: None,
            radii: [0.0; 4],
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    pub(super) fn validate_custom_effects(&self, graph: &EffectGraph) -> Result<(), RendererError> {
        for effect in graph.custom_effects() {
            if effect.parameters.len() > 24 {
                return Err(RendererError::TooManyEffectParameters {
                    provided: effect.parameters.len(),
                    maximum: 24,
                });
            }
            if !self.effect.contains(effect.shader) {
                return Err(RendererError::MissingShader(effect.shader.0));
            }
        }
        Ok(())
    }

    pub(super) fn render_effect_graph(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        graph: &EffectGraph,
        viewport: [f32; 2],
    ) {
        self.offscreen.begin_frame();
        self.effect.begin_frame();
        let region = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let root = self.acquire_target(region, region.size);
        self.clear_target(encoder, root, self.renderer_config.wgpu_clear_color());
        self.render_effect_nodes(encoder, &graph.roots, root, viewport);
        clear_view(encoder, surface, self.renderer_config.wgpu_clear_color());
        let mut params = uniform(viewport, region, root.region, root.region, region.as_rect());
        params.mode = 99;
        params.data[0] = 1.0;
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: surface,
                target_region: region,
                target_extent: region.size,
                output_region: region,
                source: self.offscreen.view(root.texture),
                backdrop: self.offscreen.view(root.texture),
                uniform: params,
                shader: None,
            },
        );
    }

    fn render_effect_nodes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        nodes: &[EffectNode],
        target: TextureTarget,
        viewport: [f32; 2],
    ) {
        let mut index = 0;
        while index < nodes.len() {
            let start = index;
            index += 1;
            match &nodes[start] {
                EffectNode::Draw(_) => {
                    while index < nodes.len() && matches!(nodes[index], EffectNode::Draw(_)) {
                        index += 1;
                    }
                    self.draw_offscreen(encoder, target, &nodes[start..index]);
                }
                EffectNode::Layer(layer) if layer.style.opacity <= 0.0 => {}
                EffectNode::Layer(layer) if !layer.style.requires_offscreen() => {
                    self.render_effect_nodes(encoder, &layer.children, target, viewport);
                }
                EffectNode::Layer(layer) => {
                    let Some(region) = layer.region else {
                        continue;
                    };
                    let layer_target = self.acquire_target(region, region.size);
                    self.clear_target(encoder, layer_target, wgpu::Color::TRANSPARENT);
                    self.render_effect_nodes(encoder, &layer.children, layer_target, viewport);
                    let foreground = self.apply_filters(
                        encoder,
                        layer_target,
                        &layer.style.filters,
                        viewport,
                        layer.style.bounds,
                        layer.style.mask,
                    );
                    self.composite_layer(
                        encoder,
                        target,
                        foreground,
                        &layer.style,
                        viewport,
                        region,
                    );
                }
            }
        }
    }

    fn draw_offscreen(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        nodes: &[EffectNode],
    ) {
        let region = target.region.as_f32();
        let quad_offset = self.quad.target_offset(&self.queue, region);
        let text_offset = self.text.target_offset(&self.queue, region);
        let image_offset = self.image.target_offset(&self.queue, region);
        let vector_offset = self.vector.target_offset(&self.queue, region);
        let attachment = Some(RenderPassColorAttachment {
            view: self.offscreen.view(target.texture),
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
        for node in nodes {
            let EffectNode::Draw(batch) = node else {
                continue;
            };
            match batch.kind {
                DrawKind::Quad => self
                    .quad
                    .draw(&mut pass, batch.instances.clone(), quad_offset),
                DrawKind::Text => self
                    .text
                    .draw(&mut pass, batch.instances.clone(), text_offset),
                DrawKind::Image(image, sampling) => self.image.draw(
                    &mut pass,
                    image,
                    sampling,
                    batch.instances.clone(),
                    image_offset,
                ),
                DrawKind::Vector(vector) => {
                    self.vector
                        .draw(&mut pass, vector, batch.instances.clone(), vector_offset);
                }
            }
        }
    }

    fn apply_filters(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        filters: &[Filter],
        viewport: [f32; 2],
        bounds: Rect,
        mask: LayerMask,
    ) -> TextureTarget {
        let mut current = source;
        for filter in plan_filters(filters) {
            current = match filter {
                PlannedFilter::Blur(radius) => {
                    self.apply_blur(encoder, current, radius, viewport, bounds)
                }
                PlannedFilter::ColorMatrix(matrix) => {
                    let mut pass = EffectPass::new(8, [0.0; 4], bounds);
                    pass.matrix = Some(matrix);
                    self.effect_pass(encoder, current, viewport, pass)
                }
                PlannedFilter::Refraction(value) => self.effect_pass(
                    encoder,
                    current,
                    viewport,
                    EffectPass::new(
                        9,
                        [value.strength, value.chromatic_aberration, value.edge, 0.0],
                        bounds,
                    ),
                ),
                PlannedFilter::Custom(effect) => {
                    let mut values = [0.0; 24];
                    values[..effect.parameters.len()].copy_from_slice(&effect.parameters);
                    let mut pass = EffectPass::new(
                        99,
                        values[..4].try_into().expect("four custom parameters"),
                        bounds,
                    );
                    pass.matrix = Some(values[4..].try_into().expect("twenty custom parameters"));
                    pass.shader = Some(effect.shader);
                    pass.radii = layer_radii(mask);
                    self.effect_pass(encoder, current, viewport, pass)
                }
            };
        }
        current
    }

    fn apply_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mut current: TextureTarget,
        radius: f32,
        viewport: [f32; 2],
        bounds: Rect,
    ) -> TextureTarget {
        if radius <= 0.01 {
            return current;
        }
        let downsample = blur_downsample(radius);
        if downsample > 1 {
            let mut pass = EffectPass::new(99, [0.0; 4], bounds);
            pass.extent = Some([
                (current.region.size[0] / downsample).max(1),
                (current.region.size[1] / downsample).max(1),
            ]);
            current = self.effect_pass(encoder, current, viewport, pass);
        }
        let sample_radius = radius / downsample as f32 * 0.35;
        for mode in [1, 2] {
            let mut pass = EffectPass::new(mode, [sample_radius, 0.0, 0.0, 0.0], bounds);
            pass.extent = Some(current.extent);
            current = self.effect_pass(encoder, current, viewport, pass);
        }
        if downsample > 1 {
            current = self.effect_pass(
                encoder,
                current,
                viewport,
                EffectPass::new(99, [0.0; 4], bounds),
            );
        }
        current
    }

    fn effect_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        viewport: [f32; 2],
        pass: EffectPass,
    ) -> TextureTarget {
        let extent = pass.extent.unwrap_or(source.region.size);
        let target = self.acquire_target(source.region, extent);
        self.clear_target(encoder, target, wgpu::Color::TRANSPARENT);
        let mut params = uniform(
            viewport,
            target.region,
            source.region,
            source.region,
            pass.bounds,
        );
        params.mode = pass.mode;
        params.data = pass.data;
        if let Some(matrix) = pass.matrix {
            params.matrix.copy_from_slice(matrix.as_chunks::<4>().0);
        }
        params.radii = pass.radii;
        self.draw_effect(
            encoder,
            target,
            target.region,
            EffectSources {
                source,
                backdrop: source,
            },
            params,
            pass.shader,
        );
        target
    }

    fn composite_layer(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        foreground: TextureTarget,
        style: &LayerStyle,
        viewport: [f32; 2],
        region: PixelRegion,
    ) {
        let snapshot = self.snapshot(encoder, target, region);
        let filtered = self.apply_filters(
            encoder,
            snapshot,
            &style.backdrop_filters,
            viewport,
            style.bounds,
            style.mask,
        );
        let mut backdrop = if style.backdrop_filters.is_empty() {
            snapshot
        } else {
            let merged = self.acquire_target(region, region.size);
            self.clear_target(encoder, merged, wgpu::Color::TRANSPARENT);
            let mut params = uniform(
                viewport,
                region,
                filtered.region,
                snapshot.region,
                style.bounds,
            );
            params.mode = 12;
            params.data[0] = style.opacity.clamp(0.0, 1.0);
            params.radii = layer_radii(style.mask);
            self.draw_effect(
                encoder,
                merged,
                region,
                EffectSources {
                    source: filtered,
                    backdrop: snapshot,
                },
                params,
                None,
            );
            merged
        };
        for shadow in &style.shadows {
            let blurred = self.apply_blur(
                encoder,
                foreground,
                shadow.blur,
                viewport,
                style.expanded_bounds(),
            );
            let shadowed = self.acquire_target(region, region.size);
            self.clear_target(encoder, shadowed, wgpu::Color::TRANSPARENT);
            let mut params = uniform(
                viewport,
                region,
                blurred.region,
                backdrop.region,
                style.bounds,
            );
            params.mode = if shadow.inset { 11 } else { 10 };
            params.color = shadow.color.as_array();
            params.color[3] *= style.opacity.clamp(0.0, 1.0);
            params.data = [1.0, shadow.offset[0], shadow.offset[1], shadow.spread];
            params.radii = layer_radii(style.mask);
            self.draw_effect(
                encoder,
                shadowed,
                region,
                EffectSources {
                    source: blurred,
                    backdrop,
                },
                params,
                None,
            );
            backdrop = shadowed;
        }
        let expansion = style.foreground_expansion();
        let mut params = uniform(
            viewport,
            region,
            foreground.region,
            backdrop.region,
            style.foreground_bounds(),
        );
        params.blend = blend_mode(style.blend_mode);
        params.data[0] = style.opacity;
        params.radii = expanded_radii(style.mask, expansion);
        self.draw_effect(
            encoder,
            target,
            region,
            EffectSources {
                source: foreground,
                backdrop,
            },
            params,
            None,
        );
    }

    fn draw_effect(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        output: PixelRegion,
        sources: EffectSources,
        params: EffectUniform,
        shader: Option<ShaderEffectId>,
    ) {
        self.effect.draw(
            &self.device,
            &self.queue,
            encoder,
            EffectDraw {
                target: self.offscreen.view(target.texture),
                target_region: target.region,
                target_extent: target.extent,
                output_region: output,
                source: self.offscreen.view(sources.source.texture),
                backdrop: self.offscreen.view(sources.backdrop.texture),
                uniform: params,
                shader,
            },
        );
    }

    fn snapshot(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        region: PixelRegion,
    ) -> TextureTarget {
        let target = self.acquire_target(region, region.size);
        let mut source_copy = self.offscreen.texture(source.texture).as_image_copy();
        source_copy.origin = wgpu::Origin3d {
            x: region.origin[0] - source.region.origin[0],
            y: region.origin[1] - source.region.origin[1],
            z: 0,
        };
        encoder.copy_texture_to_texture(
            source_copy,
            self.offscreen.texture(target.texture).as_image_copy(),
            wgpu::Extent3d {
                width: region.size[0],
                height: region.size[1],
                depth_or_array_layers: 1,
            },
        );
        target
    }

    fn acquire_target(&mut self, region: PixelRegion, extent: [u32; 2]) -> TextureTarget {
        let texture = self.offscreen.acquire(&self.device, extent[0], extent[1]);
        TextureTarget::new(texture, region, extent)
    }

    fn clear_target(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        color: wgpu::Color,
    ) {
        clear_view(encoder, self.offscreen.view(target.texture), color);
    }
}

fn clear_view(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, color: wgpu::Color) {
    let attachment = Some(RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(color),
            store: StoreOp::Store,
        },
    });
    drop(encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("argui-effect-clear"),
        color_attachments: &[attachment],
        ..Default::default()
    }));
}

fn expanded_radii(mask: LayerMask, expansion: f32) -> [f32; 4] {
    match mask {
        LayerMask::Rounded(radii) => radii.as_array().map(|radius| radius + expansion),
        LayerMask::None | LayerMask::Bounds => [0.0; 4],
    }
}

fn blur_downsample(radius: f32) -> u32 {
    if radius >= 12.0 {
        4
    } else if radius >= 6.0 {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use argui_paint::{CornerRadii, LayerMask};

    use super::{blur_downsample, expanded_radii};

    #[test]
    fn blur_sampling_and_outer_radii_have_explicit_thresholds() {
        assert_eq!(blur_downsample(5.9), 1);
        assert_eq!(blur_downsample(6.0), 2);
        assert_eq!(blur_downsample(11.9), 2);
        assert_eq!(blur_downsample(12.0), 4);
        assert_eq!(expanded_radii(LayerMask::None, 5.0), [0.0; 4]);
        assert_eq!(expanded_radii(LayerMask::Bounds, 5.0), [0.0; 4]);
        assert_eq!(
            expanded_radii(LayerMask::Rounded(CornerRadii::all(4.0)), 5.0),
            [9.0; 4]
        );
    }
}
