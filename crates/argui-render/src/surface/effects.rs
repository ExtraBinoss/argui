use argui_core::Rect;
use argui_paint::{EffectId, Filter, LayerMask};
use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{
    RendererError,
    batch::DrawKind,
    effect::{EffectDraw, EffectUniform, layer_radii, uniform},
    effect_graph::{EffectGraph, EffectNode},
    effect_plan::{PlannedFilter, plan_filters},
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::SurfaceRenderer;

#[derive(Clone, Debug)]
pub(super) struct CachedLayer {
    layer: crate::effect_graph::EffectLayer,
    target: TextureTarget,
}

#[derive(Default)]
struct CacheFrameStats {
    hits: usize,
    damaged_pixels: u64,
    used: std::collections::HashSet<argui_paint::RenderObjectId>,
}

struct EffectPass {
    mode: u32,
    data: [f32; 4],
    matrix: Option<[f32; 20]>,
    bounds: Rect,
    shader: Option<(EffectId, usize)>,
    parameters: Vec<u32>,
    extent: Option<[u32; 2]>,
    radii: [f32; 4],
    label: String,
    object: Option<argui_paint::RenderObjectId>,
}

#[derive(Clone, Copy)]
pub(super) struct EffectSources {
    pub(super) source: TextureTarget,
    pub(super) backdrop: TextureTarget,
}

impl EffectPass {
    fn new(mode: u32, data: [f32; 4], bounds: Rect) -> Self {
        Self {
            mode,
            data,
            matrix: None,
            bounds,
            shader: None,
            parameters: Vec::new(),
            extent: None,
            radii: [0.0; 4],
            label: built_in_label(mode).into(),
            object: None,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::too_many_arguments)]
impl SurfaceRenderer {
    pub(super) fn validate_custom_effects(
        &mut self,
        graph: &EffectGraph,
    ) -> Result<usize, RendererError> {
        let mut additional_passes = 0;
        for effect in graph.effects() {
            let definition = self
                .renderer_config
                .effects
                .get(effect.id)
                .ok_or(RendererError::MissingEffect(effect.id.0))?;
            definition.validate_instance(effect)?;
            additional_passes += definition.passes.len().saturating_sub(1);
            if !self.effect.contains(effect.id) {
                for (index, pass) in definition.passes.iter().enumerate() {
                    self.effect
                        .register(&self.device, definition.id, index, pass.wgsl)?;
                }
            }
        }
        Ok(additional_passes)
    }

    pub(super) fn render_effect_graph(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        graph: &EffectGraph,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
    ) -> (usize, u64) {
        if self.offscreen.begin_frame() {
            self.layer_cache.clear();
        }
        self.layer_cache
            .retain(|_, cached| self.offscreen.retain(cached.target.texture));
        self.effect.begin_frame();
        let mut cache_stats = CacheFrameStats::default();
        let region = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let root = self.acquire_target(region, region.size);
        self.clear_target(encoder, root, self.renderer_config.wgpu_clear_color());
        self.render_effect_nodes(
            encoder,
            &graph.roots,
            root,
            viewport,
            profiler,
            None,
            &mut cache_stats,
        );
        clear_view(encoder, surface, self.renderer_config.wgpu_clear_color());
        let mut params = uniform(viewport, region, root, root, region.as_rect());
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
                parameters: &[],
                profiler,
                profile_label: "composite.present",
                profile_object: None,
            },
        );
        self.layer_cache
            .retain(|profile, _| cache_stats.used.contains(profile));
        (cache_stats.hits, cache_stats.damaged_pixels)
    }

    fn render_effect_nodes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        nodes: &[EffectNode],
        target: TextureTarget,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
        owner: Option<argui_paint::RenderObjectId>,
        cache_stats: &mut CacheFrameStats,
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
                    self.draw_offscreen(encoder, target, &nodes[start..index], profiler, owner);
                }
                EffectNode::Layer(layer) if layer.style.opacity <= 0.0 => {
                    if let Some(profile) = layer.style.profile {
                        cache_stats.used.insert(profile);
                    }
                }
                EffectNode::Layer(layer) if !layer.style.requires_offscreen() => {
                    self.render_effect_nodes(
                        encoder,
                        &layer.children,
                        target,
                        viewport,
                        profiler,
                        layer.style.profile,
                        cache_stats,
                    );
                }
                EffectNode::Layer(layer) => {
                    let Some(region) = layer.region else {
                        continue;
                    };
                    let cached = layer.style.profile.and_then(|profile| {
                        cache_stats.used.insert(profile);
                        self.layer_cache
                            .get(&profile)
                            .filter(|cached| same_layer_content(&cached.layer, layer))
                            .map(|cached| cached.target)
                    });
                    let foreground = if let Some(cached) = cached {
                        cache_stats.hits += 1;
                        cached
                    } else {
                        cache_stats.damaged_pixels +=
                            u64::from(region.size[0]) * u64::from(region.size[1]);
                        let layer_target = self.acquire_target(region, region.size);
                        self.clear_target(encoder, layer_target, wgpu::Color::TRANSPARENT);
                        self.render_effect_nodes(
                            encoder,
                            &layer.children,
                            layer_target,
                            viewport,
                            profiler,
                            layer.style.profile,
                            cache_stats,
                        );
                        let foreground = self.apply_filters(
                            encoder,
                            layer_target,
                            &layer.style.filters,
                            viewport,
                            layer.style.bounds,
                            layer.style.mask,
                            profiler,
                            layer.style.profile,
                        );
                        if let Some(profile) = layer.style.profile {
                            self.layer_cache.insert(
                                profile,
                                CachedLayer {
                                    layer: layer.clone(),
                                    target: foreground,
                                },
                            );
                        }
                        foreground
                    };
                    let Some(output_region) =
                        PixelRegion::from_rect(layer.style.transformed_bounds(), target.region)
                    else {
                        continue;
                    };
                    self.composite_layer(
                        encoder,
                        target,
                        foreground,
                        &layer.style,
                        viewport,
                        output_region,
                        profiler,
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
        profiler: Option<&GpuFrameCapture>,
        owner: Option<argui_paint::RenderObjectId>,
    ) {
        let region = target.region.as_f32();
        let quad_offset = self.quad.target_offset(&self.queue, region);
        let text_offset = self.text.target_offset(&self.queue, region);
        let image_offset = self.image.target_offset(&self.queue, region);
        let vector_offset = self.vector.target_offset(&self.queue, region);
        let canvas_offset = self.gpu_canvas.target_offset(&self.queue, region);
        let attachment = Some(RenderPassColorAttachment {
            view: self.offscreen.view(target.texture),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
        });
        let timestamp_writes = profiler.and_then(|profiler| {
            profiler.timestamp_writes(
                "layer.content",
                owner,
                u64::from(target.region.size[0]) * u64::from(target.region.size[1]),
            )
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("argui-layer-content"),
            color_attachments: &[attachment],
            timestamp_writes,
            ..Default::default()
        });
        pass.set_viewport(
            0.0,
            0.0,
            target.extent[0] as f32,
            target.extent[1] as f32,
            0.0,
            1.0,
        );
        pass.set_scissor_rect(0, 0, target.extent[0], target.extent[1]);
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
                DrawKind::GpuCanvas(index) => {
                    self.gpu_canvas
                        .draw(&mut pass, index, batch.instances.clone(), canvas_offset)
                }
                DrawKind::Vector => {
                    self.vector
                        .draw(&mut pass, batch.instances.clone(), vector_offset);
                }
            }
        }
    }

    pub(super) fn apply_filters(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        filters: &[Filter],
        viewport: [f32; 2],
        bounds: Rect,
        mask: LayerMask,
        profiler: Option<&GpuFrameCapture>,
        object: Option<argui_paint::RenderObjectId>,
    ) -> TextureTarget {
        let mut current = source;
        for filter in plan_filters(filters) {
            current = match filter {
                PlannedFilter::Blur(radius) => {
                    self.apply_blur(encoder, current, radius, viewport, bounds, profiler, object)
                }
                PlannedFilter::ColorMatrix(matrix) => {
                    let mut pass = EffectPass::new(8, [0.0; 4], bounds);
                    pass.matrix = Some(matrix);
                    pass.object = object;
                    self.effect_pass(encoder, current, viewport, pass, profiler)
                }
                PlannedFilter::Refraction(value) => {
                    let mut pass = EffectPass::new(
                        9,
                        [value.strength, value.chromatic_aberration, value.edge, 0.0],
                        bounds,
                    );
                    let divisor = self
                        .renderer_config
                        .effect_quality
                        .settings()
                        .spatial_effect_divisor
                        .max(1);
                    if divisor > 1 {
                        pass.extent = Some([
                            (current.region.size[0] / divisor).max(1),
                            (current.region.size[1] / divisor).max(1),
                        ]);
                    }
                    pass.object = object;
                    self.effect_pass(encoder, current, viewport, pass, profiler)
                }
                PlannedFilter::Effect(effect) => {
                    let parameters = effect.packed_words();
                    let passes = self
                        .renderer_config
                        .effects
                        .get(effect.id)
                        .expect("validated effect registry")
                        .passes;
                    for (pass_index, definition) in passes.iter().enumerate() {
                        let mut pass = EffectPass::new(99, [0.0; 4], bounds);
                        pass.shader = Some((effect.id, pass_index));
                        pass.parameters.clone_from(&parameters);
                        pass.radii = layer_radii(mask);
                        pass.label = format!("effect.{}.{}", effect.id.0, definition.name);
                        pass.object = object;
                        let divisor = definition
                            .scale_divisor
                            .saturating_mul(
                                self.renderer_config
                                    .effect_quality
                                    .settings()
                                    .spatial_effect_divisor,
                            )
                            .max(1);
                        if divisor > 1 {
                            pass.extent = Some([
                                (current.region.size[0] / divisor).max(1),
                                (current.region.size[1] / divisor).max(1),
                            ]);
                        }
                        current = self.effect_pass(encoder, current, viewport, pass, profiler);
                    }
                    current
                }
            };
        }
        current
    }

    pub(super) fn apply_blur(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        mut current: TextureTarget,
        radius: f32,
        viewport: [f32; 2],
        bounds: Rect,
        profiler: Option<&GpuFrameCapture>,
        object: Option<argui_paint::RenderObjectId>,
    ) -> TextureTarget {
        if radius <= 0.01 {
            return current;
        }
        let downsample = blur_downsample(
            radius,
            self.renderer_config
                .effect_quality
                .settings()
                .blur_downsample_bias,
        );
        if downsample > 1 {
            let mut pass = EffectPass::new(99, [0.0; 4], bounds);
            pass.extent = Some([
                (current.region.size[0] / downsample).max(1),
                (current.region.size[1] / downsample).max(1),
            ]);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        let sample_radius = radius / downsample as f32 * 0.35;
        for mode in [1, 2] {
            let mut pass = EffectPass::new(mode, [sample_radius, 0.0, 0.0, 0.0], bounds);
            pass.extent = Some(current.extent);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        if downsample > 1 {
            let mut pass = EffectPass::new(99, [0.0; 4], bounds);
            pass.object = object;
            current = self.effect_pass(encoder, current, viewport, pass, profiler);
        }
        current
    }

    fn effect_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        viewport: [f32; 2],
        pass: EffectPass,
        profiler: Option<&GpuFrameCapture>,
    ) -> TextureTarget {
        let extent = pass.extent.unwrap_or(source.region.size);
        let target = self.acquire_target(source.region, extent);
        self.clear_target(encoder, target, wgpu::Color::TRANSPARENT);
        let mut params = uniform(viewport, target.region, source, source, pass.bounds);
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
            &pass.parameters,
            profiler,
            &pass.label,
            pass.object,
        );
        target
    }

    pub(super) fn draw_effect(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        output: PixelRegion,
        sources: EffectSources,
        params: EffectUniform,
        shader: Option<(EffectId, usize)>,
        parameters: &[u32],
        profiler: Option<&GpuFrameCapture>,
        label: &str,
        object: Option<argui_paint::RenderObjectId>,
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
                parameters,
                profiler,
                profile_label: label,
                profile_object: object,
            },
        );
    }

    pub(super) fn acquire_target(
        &mut self,
        region: PixelRegion,
        extent: [u32; 2],
    ) -> TextureTarget {
        let texture = self.offscreen.acquire(&self.device, extent[0], extent[1]);
        TextureTarget::with_allocation(texture, region, extent, self.offscreen.extent(texture))
    }

    pub(super) fn clear_target(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        color: wgpu::Color,
    ) {
        clear_view(encoder, self.offscreen.view(target.texture), color);
    }
}

/// Returns whether a cached foreground remains valid across composition-only changes.
fn same_layer_content(
    cached: &crate::effect_graph::EffectLayer,
    current: &crate::effect_graph::EffectLayer,
) -> bool {
    cached.region == current.region
        && cached.content_revision == current.content_revision
        && cached.children == current.children
        && cached.style.bounds == current.style.bounds
        && cached.style.filters == current.style.filters
        && cached.style.mask == current.style.mask
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

fn blur_downsample(radius: f32, bias: u32) -> u32 {
    let base = if radius >= 12.0 {
        4
    } else if radius >= 6.0 {
        2
    } else {
        1
    };
    base * bias.max(1)
}

const fn built_in_label(mode: u32) -> &'static str {
    match mode {
        1 => "effect.blur-horizontal",
        2 => "effect.blur-vertical",
        8 => "effect.color-matrix",
        9 => "effect.refraction",
        10 => "composite.drop-shadow",
        11 => "composite.inset-shadow",
        12 => "composite.backdrop",
        _ => "composite.copy",
    }
}
