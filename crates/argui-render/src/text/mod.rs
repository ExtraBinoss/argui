mod atlas;
mod pipeline;

use std::ops::Range;

use argui_text::{PreparedText, TextEngine};

use crate::RendererError;
use atlas::GlyphAtlas;
use pipeline::{GlyphInstance, TextPipeline};

pub(crate) struct TextGpu {
    atlas: GlyphAtlas,
    pipeline: TextPipeline,
}

pub(crate) struct TextDraw {
    ranges: Vec<Range<u32>>,
}

impl TextDraw {
    pub fn all(&self) -> Range<u32> {
        0..self.ranges.iter().map(|range| range.end).max().unwrap_or(0)
    }

    pub fn ranges(&self) -> &[Range<u32>] {
        &self.ranges
    }
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
    ) -> Result<TextDraw, RendererError> {
        match self.prepare_once(queue, engine, text) {
            Ok((instances, ranges)) => {
                self.pipeline.write(device, queue, &instances, viewport);
                Ok(TextDraw { ranges })
            }
            Err(RendererError::GlyphAtlasFull) => {
                self.atlas.reset();
                let (instances, ranges) = self.prepare_once(queue, engine, text)?;
                self.pipeline.write(device, queue, &instances, viewport);
                Ok(TextDraw { ranges })
            }
            Err(error) => Err(error),
        }
    }

    fn prepare_once(
        &mut self,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<(Vec<GlyphInstance>, Vec<Range<u32>>), RendererError> {
        let mut instances = Vec::with_capacity(text.glyphs.len());
        let mut ranges = vec![0..0; text.blocks];
        for glyph in &text.glyphs {
            let Some(entry) = self.atlas.get_or_insert(queue, engine, glyph.key)? else {
                continue;
            };
            let instance = instances.len() as u32;
            instances.push(GlyphInstance::new(*glyph, entry, self.atlas.size()));
            let range = &mut ranges[glyph.block];
            if range.start >= range.end {
                *range = instance..instance + 1;
            } else {
                range.end = instance + 1;
            }
        }
        Ok((instances, ranges))
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>, instances: Range<u32>) {
        self.pipeline.draw(pass, instances);
    }
}

#[cfg(test)]
mod tests {
    use super::TextDraw;

    #[test]
    fn text_draws_keep_per_block_ranges_and_full_extent() {
        let draw = TextDraw {
            ranges: vec![0..3, 0..0, 3..7, 0..0],
        };

        assert_eq!(draw.all(), 0..7);
        assert_eq!(draw.ranges(), &[0..3, 0..0, 3..7, 0..0]);
    }
}
