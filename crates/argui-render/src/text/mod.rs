mod atlas;
mod pipeline;

use argui_text::{PreparedText, TextEngine};

use crate::RendererError;
use atlas::GlyphAtlas;
use pipeline::{GlyphInstance, TextPipeline};

pub(crate) struct TextGpu {
    atlas: GlyphAtlas,
    pipeline: TextPipeline,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl TextGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let atlas = GlyphAtlas::new(device, 1024);
        let pipeline = TextPipeline::new(device, format, atlas.view(), atlas.sampler());
        Self { atlas, pipeline }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
        viewport: [f32; 2],
    ) -> Result<u32, RendererError> {
        match self.prepare_once(queue, engine, text) {
            Ok(instances) => {
                let count = instances.len() as u32;
                self.pipeline.write(device, queue, &instances, viewport);
                Ok(count)
            }
            Err(RendererError::GlyphAtlasFull) => {
                self.atlas.reset();
                let instances = self.prepare_once(queue, engine, text)?;
                let count = instances.len() as u32;
                self.pipeline.write(device, queue, &instances, viewport);
                Ok(count)
            }
            Err(error) => Err(error),
        }
    }

    fn prepare_once(
        &mut self,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<Vec<GlyphInstance>, RendererError> {
        let mut instances = Vec::with_capacity(text.glyphs.len());
        for glyph in &text.glyphs {
            let Some(entry) = self.atlas.get_or_insert(queue, engine, glyph.key)? else {
                continue;
            };
            instances.push(GlyphInstance::new(*glyph, entry, self.atlas.size()));
        }
        Ok(instances)
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>, instance_count: u32) {
        self.pipeline.draw(pass, instance_count);
    }
}
