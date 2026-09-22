use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{
    DamageMode, DamagePlan, DamageProfile, DamageRegion,
    effect::{EffectDraw, uniform},
    effect_graph::EffectGraph,
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::{SurfaceRenderer, effects::CacheFrameStats};

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Repaints the effect graph into its persistent root and presents that root.
    ///
    /// `plan` selects a full, regional, or reuse path; `graph` supplies the
    /// current offscreen composition; `viewport` and `surface` identify the
    /// physical output; `profiler` optionally records GPU passes.
    pub(super) fn render_effect_graph(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        graph: &EffectGraph,
        viewport: [f32; 2],
        plan: &DamagePlan,
        profiler: Option<&GpuFrameCapture>,
    ) -> (usize, u64, DamageProfile) {
        let region = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        self.prepare_effect_pool(region);
        self.effect.begin_frame();

        let had_root = self.effect_root.is_some();
        let root = self.effect_root.unwrap_or_else(|| {
            let root = self.acquire_target(region, region.size);
            self.effect_root = Some(root);
            root
        });
        let mut cache_stats = CacheFrameStats {
            used: graph.profiles(),
            ..CacheFrameStats::default()
        };
        let damage = match plan {
            DamagePlan::Partial(regions) if had_root => {
                self.clear_effect_regions(encoder, root, regions);
                if let Some(clip) = regions
                    .iter()
                    .copied()
                    .map(|damage| PixelRegion {
                        origin: [damage.x, damage.y],
                        size: [damage.width, damage.height],
                    })
                    .reduce(|acc, r| acc.union(r))
                {
                    self.render_effect_nodes(
                        encoder,
                        &graph.roots,
                        root,
                        viewport,
                        profiler,
                        None,
                        &mut cache_stats,
                        Some(clip),
                        &[],
                    );
                }
                DamageProfile {
                    mode: DamageMode::Partial,
                    regions: regions.len(),
                    damaged_pixels: regions.iter().copied().map(DamageRegion::pixels).sum(),
                    retained_bytes: self.effect_root_bytes(),
                }
            }
            DamagePlan::Unchanged if had_root => DamageProfile {
                mode: DamageMode::Reused,
                retained_bytes: self.effect_root_bytes(),
                ..DamageProfile::default()
            },
            _ => {
                self.clear_target(encoder, root, self.renderer_config.wgpu_clear_color());
                self.render_effect_nodes(
                    encoder,
                    &graph.roots,
                    root,
                    viewport,
                    profiler,
                    None,
                    &mut cache_stats,
                    None,
                    &[],
                );
                DamageProfile {
                    mode: if matches!(plan, DamagePlan::Full) {
                        DamageMode::Full
                    } else {
                        DamageMode::Seed
                    },
                    regions: 1,
                    damaged_pixels: u64::from(region.size[0]) * u64::from(region.size[1]),
                    retained_bytes: self.effect_root_bytes(),
                }
            }
        };
        self.present_effect_root(encoder, surface, root, viewport, profiler);
        self.layer_cache
            .retain(|profile, _| cache_stats.used.contains(profile));
        (cache_stats.hits, cache_stats.damaged_pixels, damage)
    }

    /// Begins a texture-pool frame and retains compatible effect resources.
    fn prepare_effect_pool(&mut self, region: PixelRegion) {
        if self.offscreen.begin_frame() {
            self.layer_cache.clear();
            self.effect_root = None;
        }
        if self
            .effect_root
            .is_some_and(|root| root.region != region || !self.offscreen.retain(root.texture))
        {
            self.effect_root = None;
        }
        self.layer_cache
            .retain(|_, cached| self.offscreen.retain(cached.target.texture));
    }

    /// Clears only `regions` in the persistent effect root.
    fn clear_effect_regions(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        root: TextureTarget,
        regions: &[DamageRegion],
    ) {
        self.damage
            .prepare_clear(&self.queue, self.renderer_config.wgpu_clear_color());
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("effect.damage.clear"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.offscreen.view(root.texture),
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        for region in regions {
            crate::damage::DamageGpu::scissor(&mut pass, *region);
            self.damage.clear(&mut pass);
        }
    }

    /// Copies the persistent effect root across the complete swapchain surface.
    fn present_effect_root(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        root: TextureTarget,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
    ) {
        let region = root.region;
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
    }

    /// Returns memory owned by the persistent effect root.
    fn effect_root_bytes(&self) -> u64 {
        self.effect_root
            .map_or(0, |root| self.offscreen.bytes(root.texture))
    }
}
