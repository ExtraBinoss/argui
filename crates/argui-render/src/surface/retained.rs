use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{
    DamageMode, DamageProfile, DamageRegion,
    batch::DrawKind,
    effect::{EffectDraw, uniform},
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::SurfaceRenderer;

struct TargetOffsets {
    quad: u32,
    text: u32,
    image: u32,
    vector: u32,
    canvas: u32,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Draws every prepared batch directly into `view` after a full clear.
    pub(super) fn draw_full_scene(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
        label: &'static str,
    ) {
        let target = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let offsets = self.target_offsets(target);
        let timestamp_writes = profiler.and_then(|capture| {
            capture.timestamp_writes(
                label,
                None,
                u64::from(target.size[0]) * u64::from(target.size[1]),
            )
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.renderer_config.wgpu_clear_color()),
                    store: StoreOp::Store,
                },
            })],
            timestamp_writes,
            ..Default::default()
        });
        self.draw_prepared_batches(&mut pass, &offsets);
    }

    /// Seeds or partially refreshes the retained root and presents it to `surface`.
    pub(super) fn draw_retained_scene(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        viewport: [f32; 2],
        regions: &[DamageRegion],
        profiler: Option<&GpuFrameCapture>,
    ) -> DamageProfile {
        let extent = [viewport[0] as u32, viewport[1] as u32];
        let seed = self
            .damage
            .ensure_target(&self.device, self.target_format, extent);
        let retained = self.damage.view().expect("retained target was ensured");
        if seed {
            self.draw_full_scene(
                encoder,
                &retained,
                viewport,
                profiler,
                "surface.damage.seed",
            );
        } else {
            self.draw_damage_regions(encoder, &retained, viewport, regions, profiler);
        }
        self.present_retained(encoder, surface, viewport, &retained, profiler);
        DamageProfile {
            mode: if seed {
                DamageMode::Seed
            } else {
                DamageMode::Partial
            },
            regions: if seed { 1 } else { regions.len() },
            damaged_pixels: if seed {
                u64::from(extent[0]) * u64::from(extent[1])
            } else {
                regions.iter().copied().map(DamageRegion::pixels).sum()
            },
            retained_bytes: self.damage.bytes(),
        }
    }

    /// Presents unchanged retained pixels without repainting the retained target.
    pub(super) fn reuse_retained_scene(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        viewport: [f32; 2],
        profiler: Option<&GpuFrameCapture>,
    ) -> DamageProfile {
        let retained = self.damage.view().expect("retained target is valid");
        self.present_retained(encoder, surface, viewport, &retained, profiler);
        DamageProfile {
            mode: DamageMode::Reused,
            retained_bytes: self.damage.bytes(),
            ..DamageProfile::default()
        }
    }

    /// Clears and redraws each changed region of `retained` under a GPU scissor.
    fn draw_damage_regions(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        retained: &wgpu::TextureView,
        viewport: [f32; 2],
        regions: &[DamageRegion],
        profiler: Option<&GpuFrameCapture>,
    ) {
        let target = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let offsets = self.target_offsets(target);
        self.damage
            .prepare_clear(&self.queue, self.renderer_config.wgpu_clear_color());
        let damaged_pixels = regions.iter().copied().map(DamageRegion::pixels).sum();
        let timestamp_writes = profiler.and_then(|capture| {
            capture.timestamp_writes("surface.damage.partial", None, damaged_pixels)
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("surface.damage.partial"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: retained,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            timestamp_writes,
            ..Default::default()
        });
        for region in regions {
            crate::damage::DamageGpu::scissor(&mut pass, *region);
            self.damage.clear(&mut pass);
        }
        if let Some(union) = regions.iter().copied().reduce(DamageRegion::union) {
            crate::damage::DamageGpu::scissor(&mut pass, union);
            self.draw_prepared_batches(&mut pass, &offsets);
        }
    }

    /// Copies the retained root across the complete swapchain surface.
    fn present_retained(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        viewport: [f32; 2],
        retained: &wgpu::TextureView,
        profiler: Option<&GpuFrameCapture>,
    ) {
        let region = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
        let target = TextureTarget::new(0, region, region.size);
        let mut params = uniform(viewport, region, target, target, region.as_rect());
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
                source: retained,
                backdrop: retained,
                uniform: params,
                shader: None,
                parameters: &[],
                profiler,
                profile_label: "surface.damage.present",
                profile_object: None,
            },
        );
    }

    /// Uploads the per-pipeline coordinate offsets for `target`.
    fn target_offsets(&mut self, target: PixelRegion) -> TargetOffsets {
        let target = target.as_f32();
        TargetOffsets {
            quad: self.quad.target_offset(&self.queue, target),
            text: self.text.target_offset(&self.queue, target),
            image: self.image.target_offset(&self.queue, target),
            vector: self.vector.target_offset(&self.queue, target),
            canvas: self.gpu_canvas.target_offset(&self.queue, target),
        }
    }

    /// Draws all prepared batches into the current scissor rectangle.
    fn draw_prepared_batches<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        offsets: &TargetOffsets,
    ) {
        for batch in &self.batches {
            match batch.kind {
                DrawKind::Quad => self.quad.draw(pass, batch.instances.clone(), offsets.quad),
                DrawKind::Text => self.text.draw(pass, batch.instances.clone(), offsets.text),
                DrawKind::Image(image, sampling) => self.image.draw(
                    pass,
                    image,
                    sampling,
                    batch.instances.clone(),
                    offsets.image,
                ),
                DrawKind::GpuCanvas(index) => {
                    self.gpu_canvas
                        .draw(pass, index, batch.instances.clone(), offsets.canvas)
                }
                DrawKind::Vector => {
                    self.vector
                        .draw(pass, batch.instances.clone(), offsets.vector);
                }
            }
        }
    }
}

/// Returns damage statistics for a direct full-viewport render.
pub(super) fn full_damage_profile(viewport: [f32; 2]) -> DamageProfile {
    DamageProfile {
        mode: DamageMode::Full,
        regions: 1,
        damaged_pixels: viewport[0] as u64 * viewport[1] as u64,
        retained_bytes: 0,
    }
}
