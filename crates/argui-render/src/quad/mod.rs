#[cfg(all(test, not(target_arch = "wasm32")))]
mod color_tests;
mod pipeline;

use std::ops::Range;

use argui_paint::{DisplayCommand, DisplayList};
use pipeline::{ClipInstance, GradientStopInstance, QuadInstance, QuadPipeline};

use crate::RendererError;

pub(crate) struct QuadGpu {
    pipeline: QuadPipeline,
    instances: Vec<QuadInstance>,
    clips: Vec<ClipInstance>,
    gradients: Vec<GradientStopInstance>,
    gradient_stop_capacity: usize,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl QuadGpu {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        gradient_stop_capacity: usize,
    ) -> Self {
        Self {
            pipeline: QuadPipeline::new(device, format),
            instances: Vec::new(),
            clips: Vec::new(),
            gradients: Vec::new(),
            gradient_stop_capacity,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<(), RendererError> {
        self.instances.clear();
        self.clips.clear();
        self.gradients.clear();
        for command in display_list.commands() {
            if let DisplayCommand::Quad(quad) = command {
                let start = self.clips.len() as u32;
                self.clips.extend(
                    quad.clips
                        .regions()
                        .iter()
                        .filter_map(|clip| ClipInstance::new(*clip, scale_factor)),
                );
                self.instances.push(QuadInstance::new(
                    quad,
                    scale_factor,
                    start,
                    self.clips.len() as u32 - start,
                    &mut self.gradients,
                ));
            }
        }
        if self.gradients.len() > self.gradient_stop_capacity {
            return Err(RendererError::TooManyGradientStops {
                provided: self.gradients.len(),
                capacity: self.gradient_stop_capacity,
            });
        }
        self.pipeline
            .write(device, queue, &self.instances, &self.clips, &self.gradients);
        Ok(())
    }

    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        instances: Range<u32>,
        viewport_offset: u32,
    ) {
        self.pipeline.draw(pass, instances, viewport_offset);
    }
}
