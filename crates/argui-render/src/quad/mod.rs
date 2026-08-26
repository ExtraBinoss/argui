mod pipeline;

use std::ops::Range;

use argui_paint::{DisplayCommand, DisplayList};
use pipeline::{QuadInstance, QuadPipeline};

pub(crate) struct QuadGpu {
    pipeline: QuadPipeline,
    instances: Vec<QuadInstance>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl QuadGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            pipeline: QuadPipeline::new(device, format),
            instances: Vec::new(),
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        viewport: [f32; 2],
        scale_factor: f32,
    ) {
        self.instances.clear();
        self.instances.extend(
            display_list
                .commands()
                .iter()
                .filter_map(|command| match command {
                    DisplayCommand::Quad(quad) => Some(QuadInstance::new(*quad, scale_factor)),
                    DisplayCommand::Text(_)
                    | DisplayCommand::BeginLayer(_)
                    | DisplayCommand::EndLayer => None,
                }),
        );
        self.pipeline
            .write(device, queue, &self.instances, viewport);
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>, instances: Range<u32>) {
        self.pipeline.draw(pass, instances);
    }
}
