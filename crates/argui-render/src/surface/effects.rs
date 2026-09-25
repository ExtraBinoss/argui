use argui_core::Rect;
use argui_paint::{EffectId, Filter, LayerMask};

use crate::{
    RendererError,
    effect::{EffectDraw, EffectUniform, layer_radii, uniform},
    effect_graph::{EffectGraph, EffectNode},
    effect_plan::{PlannedFilter, plan_filters},
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::SurfaceRenderer;

mod draw;
mod helpers;
use helpers::{
    built_in_label, clear_view, clipped_output_region, same_layer_content, skipped_layer,
};

#[derive(Clone, Debug)]
pub(super) struct CachedLayer {
    pub(super) layer: crate::effect_graph::EffectLayer,
    pub(super) target: TextureTarget,
}

#[derive(Default)]
pub(super) struct CacheFrameStats {
    pub(super) hits: usize,
    pub(super) damaged_pixels: u64,
    pub(super) used: std::collections::HashSet<argui_paint::RenderObjectId>,
}

pub(super) struct EffectPass {
    pub(super) mode: u32,
    pub(super) data: [f32; 4],
    matrix: Option<[f32; 20]>,
    bounds: Rect,
    shader: Option<(EffectId, u64, usize)>,
    parameters: Vec<u32>,
    pub(super) extent: Option<[u32; 2]>,
    radii: [f32; 4],
    label: String,
    pub(super) object: Option<argui_paint::RenderObjectId>,
}

#[derive(Clone, Copy)]
pub(super) struct EffectSources {
    pub(super) source: TextureTarget,
    pub(super) backdrop: TextureTarget,
}

impl EffectPass {
    /// Creates a built-in pass for shader `mode`, scalar `data`, and layer `bounds`.
    ///
    /// Returns a pass at the source extent; callers may set an alternate output
    /// extent or attribution fields before encoding it.
    pub(super) fn new(mode: u32, data: [f32; 4], bounds: Rect) -> Self {
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
                .get(&effect.id)
                .ok_or_else(|| RendererError::MissingEffect(effect.id.clone()))?;
            definition.validate_instance(effect)?;
            additional_passes += definition.passes.len().saturating_sub(1);
            if !self.effect.contains(&effect.id, definition.revision) {
                for (index, pass) in definition.passes.iter().enumerate() {
                    self.effect.register(
                        &self.device,
                        definition.id.clone(),
                        definition.revision,
                        index,
                        &pass.wgsl,
                    )?;
                }
            }
        }
        Ok(additional_passes)
    }

    /// Encodes `nodes` into `target` within `clip` and the current `backdrop_stack`.
    /// `encoder` records GPU work, `viewport` supplies output size, `profiler` and
    /// `owner` label passes, and `cache_stats` tracks reuse. `clear_first_draw`
    /// clears the attachment in the first Draw pass when the caller already
    /// established that the graph begins with Draw.
    pub(super) fn render_effect_nodes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        nodes: &[EffectNode],
        target: TextureTarget,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
        owner: Option<argui_paint::RenderObjectId>,
        cache_stats: &mut CacheFrameStats,
        clip: Option<PixelRegion>,
        backdrop_stack: &[TextureTarget],
        clear_first_draw: Option<wgpu::Color>,
    ) {
        let mut index = 0;
        while index < nodes.len() {
            let start = index;
            index += 1;
            match &nodes[start] {
                EffectNode::Draw(_) => {
                    let mut merged: Option<Vec<EffectNode>> = None;
                    while let Some(node) = nodes.get(index) {
                        match node {
                            EffectNode::Draw(_) => {
                                if let Some(draws) = &mut merged {
                                    draws.push(node.clone());
                                }
                            }
                            EffectNode::Layer(layer)
                                if skipped_layer(layer, target.region, clip) =>
                            {
                                merged.get_or_insert_with(|| nodes[start..index].to_vec());
                            }
                            _ => break,
                        }
                        index += 1;
                    }
                    if let Some(draws) = merged {
                        self.draw_offscreen(
                            encoder,
                            target,
                            &draws,
                            profiler,
                            owner,
                            clip,
                            (start == 0).then_some(clear_first_draw).flatten(),
                        );
                    } else {
                        self.draw_offscreen(
                            encoder,
                            target,
                            &nodes[start..index],
                            profiler,
                            owner,
                            clip,
                            (start == 0).then_some(clear_first_draw).flatten(),
                        );
                    }
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
                        clip,
                        backdrop_stack,
                        None,
                    );
                }
                EffectNode::Layer(layer) => {
                    let Some(region) = layer.region else {
                        continue;
                    };
                    let Some(output_region) = clipped_output_region(&layer.style, target.region)
                    else {
                        continue;
                    };
                    let Some(composite_region) =
                        clip.map_or(Some(output_region), |clip| output_region.intersection(clip))
                    else {
                        continue;
                    };
                    let cacheable = !layer.content_reads_backdrop();
                    let cached = layer
                        .style
                        .profile
                        .filter(|_| cacheable)
                        .and_then(|profile| {
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
                        let children =
                            if matches!(layer.children.first(), Some(EffectNode::Draw(_))) {
                                let first_draws = layer
                                    .children
                                    .iter()
                                    .take_while(|node| matches!(node, EffectNode::Draw(_)))
                                    .count();
                                self.draw_offscreen(
                                    encoder,
                                    layer_target,
                                    &layer.children[..first_draws],
                                    profiler,
                                    layer.style.profile,
                                    None,
                                    Some(wgpu::Color::TRANSPARENT),
                                );
                                &layer.children[first_draws..]
                            } else {
                                self.clear_target(encoder, layer_target, wgpu::Color::TRANSPARENT);
                                &layer.children[..]
                            };
                        let mut child_backdrops = backdrop_stack.to_vec();
                        child_backdrops.push(target);
                        self.render_effect_nodes(
                            encoder,
                            children,
                            layer_target,
                            viewport,
                            profiler,
                            layer.style.profile,
                            cache_stats,
                            None,
                            &child_backdrops,
                            None,
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
                        if let Some(profile) = layer.style.profile.filter(|_| cacheable) {
                            self.layer_cache.insert(
                                profile,
                                CachedLayer {
                                    layer: layer.as_ref().clone(),
                                    target: foreground,
                                },
                            );
                        }
                        foreground
                    };
                    self.composite_layer(
                        encoder,
                        target,
                        foreground,
                        &layer.style,
                        viewport,
                        composite_region,
                        profiler,
                        backdrop_stack,
                    );
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
                PlannedFilter::DropShadow(shadow) => self.apply_filter_shadow(
                    encoder, current, shadow, viewport, bounds, profiler, object,
                ),
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
                    let definition = self
                        .renderer_config
                        .effects
                        .get(&effect.id)
                        .expect("validated effect registry");
                    let definition_revision = definition.revision;
                    let passes = definition.passes.clone();
                    for (pass_index, definition) in passes.iter().enumerate() {
                        let mut pass = EffectPass::new(99, [0.0; 4], bounds);
                        pass.shader = Some((effect.id.clone(), definition_revision, pass_index));
                        pass.parameters.clone_from(&parameters);
                        pass.radii = layer_radii(mask);
                        pass.label = format!("effect.{}.{}", effect.id.as_str(), definition.name);
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

    /// Encodes one full-region filter pass and returns its offscreen target.
    ///
    /// `encoder` receives the render commands, `source` supplies sampled pixels,
    /// `viewport` sets physical coordinates, `pass` selects the shader and output
    /// extent, and `profiler` records optional GPU timing.
    pub(super) fn effect_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: TextureTarget,
        viewport: [f32; 2],
        pass: EffectPass,
        profiler: Option<&GpuFrameCapture>,
    ) -> TextureTarget {
        let extent = pass.extent.unwrap_or(source.region.size);
        let target = self.acquire_target(source.region, extent);
        if !(1..=4).contains(&pass.mode) {
            self.clear_target(encoder, target, wgpu::Color::TRANSPARENT);
        }
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
        shader: Option<(EffectId, u64, usize)>,
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
