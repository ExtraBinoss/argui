use wgpu::{LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};

use crate::{
    batch::DrawKind,
    effect_graph::EffectNode,
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

use super::{SurfaceRenderer, helpers};

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    /// Records `nodes` in `encoder` for `target`, clearing it with `clear` when supplied.
    /// `profiler` and `owner` attribute GPU work; `clip` restricts output pixels.
    pub(super) fn draw_offscreen(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: TextureTarget,
        nodes: &[EffectNode],
        profiler: Option<&GpuFrameCapture>,
        owner: Option<argui_paint::RenderObjectId>,
        clip: Option<PixelRegion>,
        clear: Option<wgpu::Color>,
    ) {
        let region = target.region.as_f32();
        let kinds = helpers::draw_kind_mask(nodes);
        let quad_offset = (kinds & 1 != 0).then(|| self.quad.target_offset(&self.queue, region));
        let text_offset = (kinds & 2 != 0).then(|| self.text.target_offset(&self.queue, region));
        let image_offset = (kinds & 4 != 0).then(|| self.image.target_offset(&self.queue, region));
        let vector_offset =
            (kinds & 8 != 0).then(|| self.vector.target_offset(&self.queue, region));
        let canvas_offset =
            (kinds & 16 != 0).then(|| self.gpu_canvas.target_offset(&self.queue, region));
        let attachment = Some(RenderPassColorAttachment {
            view: self.offscreen.view(target.texture),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: clear.map_or(LoadOp::Load, LoadOp::Clear),
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
        let Some(clip) = clip.map_or(Some(target.region), |clip| target.region.intersection(clip))
        else {
            return;
        };
        let viewport = target.viewport_for(clip);
        pass.set_scissor_rect(
            viewport[0] as u32,
            viewport[1] as u32,
            (viewport[2] as u32).max(1),
            (viewport[3] as u32).max(1),
        );
        for node in nodes {
            let EffectNode::Draw(batch) = node else {
                continue;
            };
            let instances = batch.instances.clone();
            match batch.kind {
                DrawKind::Quad => {
                    self.quad
                        .draw(&mut pass, instances, quad_offset.expect("quad offset"))
                }
                DrawKind::Text => {
                    self.text
                        .draw(&mut pass, instances, text_offset.expect("text offset"))
                }
                DrawKind::Image(image, sampling) => self.image.draw(
                    &mut pass,
                    image,
                    sampling,
                    instances,
                    image_offset.expect("image offset"),
                ),
                DrawKind::GpuCanvas(index) => self.gpu_canvas.draw(
                    &mut pass,
                    index,
                    instances,
                    canvas_offset.expect("canvas offset"),
                ),
                DrawKind::Vector => {
                    self.vector
                        .draw(&mut pass, instances, vector_offset.expect("vector offset"))
                }
            }
        }
    }
}
