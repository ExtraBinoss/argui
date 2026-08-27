mod atlas;
mod pipeline;

use std::ops::Range;

use argui_core::Affine2D;
use argui_paint::{ClipChain, DisplayCommand, DisplayList};
use argui_text::{PreparedText, TextEngine};

use crate::RendererError;
use atlas::GlyphAtlas;
use pipeline::{GlyphInstance, TextClip, TextPipeline};

type PreparedGlyphs = (Vec<GlyphInstance>, Vec<Range<u32>>, Vec<TextClip>);

#[derive(Clone, Debug)]
struct BlockVisual {
    transform: Affine2D,
    clips: ClipChain,
}

#[derive(Clone, Copy)]
struct PreparedVisual {
    transform: Affine2D,
    clip_start: u32,
    clip_count: u32,
}

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
    ) -> Result<TextDraw, RendererError> {
        self.prepare_with_visuals(device, queue, engine, text, None, 1.0)
    }

    pub fn prepare_ui(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<TextDraw, RendererError> {
        self.prepare_with_visuals(
            device,
            queue,
            engine,
            text,
            Some(display_list),
            scale_factor,
        )
    }

    fn prepare_with_visuals(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
        display_list: Option<&DisplayList>,
        scale_factor: f32,
    ) -> Result<TextDraw, RendererError> {
        let visuals = display_list.map(|list| block_visuals(list, text.blocks));
        match self.prepare_once(queue, engine, text, visuals.as_deref(), scale_factor) {
            Ok((instances, ranges, clips)) => {
                self.pipeline.write(device, queue, &instances, &clips);
                Ok(TextDraw { ranges })
            }
            Err(RendererError::GlyphAtlasFull) => {
                self.atlas.reset();
                let (instances, ranges, clips) =
                    self.prepare_once(queue, engine, text, visuals.as_deref(), scale_factor)?;
                self.pipeline.write(device, queue, &instances, &clips);
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
        visuals: Option<&[Option<BlockVisual>]>,
        scale_factor: f32,
    ) -> Result<PreparedGlyphs, RendererError> {
        let mut instances = Vec::with_capacity(text.glyphs.len());
        let mut ranges = vec![0..0; text.blocks];
        let mut clips = Vec::new();
        let prepared = prepare_visuals(text, visuals, scale_factor, &mut clips);
        for glyph in &text.glyphs {
            let Some(entry) = self.atlas.get_or_insert(queue, engine, glyph.key)? else {
                continue;
            };
            let instance = instances.len() as u32;
            let visual = prepared[glyph.block];
            instances.push(GlyphInstance::new(
                *glyph,
                entry,
                self.atlas.size(),
                visual.transform,
                visual.clip_start,
                visual.clip_count,
            ));
            let range = &mut ranges[glyph.block];
            if range.start >= range.end {
                *range = instance..instance + 1;
            } else {
                range.end = instance + 1;
            }
        }
        Ok((instances, ranges, clips))
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

fn block_visuals(display_list: &DisplayList, blocks: usize) -> Vec<Option<BlockVisual>> {
    let mut visuals = vec![None; blocks];
    for command in display_list.commands() {
        if let DisplayCommand::Text {
            block,
            transform,
            clips,
        } = command
            && let Some(slot) = visuals.get_mut(*block)
        {
            *slot = Some(BlockVisual {
                transform: *transform,
                clips: clips.clone(),
            });
        }
    }
    visuals
}

fn prepare_visuals(
    text: &PreparedText,
    visuals: Option<&[Option<BlockVisual>]>,
    scale_factor: f32,
    clips: &mut Vec<TextClip>,
) -> Vec<PreparedVisual> {
    (0..text.blocks)
        .map(|block| {
            let start = clips.len() as u32;
            let transform = visuals
                .and_then(|items| items.get(block))
                .and_then(Option::as_ref)
                .map_or(Affine2D::IDENTITY, |visual| {
                    clips.extend(
                        visual
                            .clips
                            .regions()
                            .iter()
                            .filter_map(|clip| TextClip::new(*clip, scale_factor)),
                    );
                    visual.transform.scaled(scale_factor)
                });
            if clips.len() as u32 == start
                && let Some(glyph) = text.glyphs.iter().find(|glyph| glyph.block == block)
            {
                clips.push(TextClip::physical(glyph.clip));
            }
            PreparedVisual {
                transform,
                clip_start: start,
                clip_count: clips.len() as u32 - start,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use argui_core::{Affine2D, Point, Rect, Size};
    use argui_paint::{ClipChain, ClipRegion, DisplayList};
    use argui_text::PreparedText;

    use super::{TextDraw, block_visuals, prepare_visuals};

    #[test]
    fn text_draws_keep_per_block_ranges_and_full_extent() {
        let draw = TextDraw {
            ranges: vec![0..3, 0..0, 3..7, 0..0],
        };

        assert_eq!(draw.all(), 0..7);
        assert_eq!(draw.ranges(), &[0..3, 0..0, 3..7, 0..0]);
    }

    #[test]
    fn text_visuals_map_valid_blocks_and_keep_unstyled_fallbacks() {
        let clip = ClipRegion::new(
            Rect::new(Point::default(), Size::new(40.0, 20.0)),
            Affine2D::IDENTITY,
        );
        let mut list = DisplayList::new();
        list.push_text_transformed(
            1,
            Affine2D::translation(4.0, 5.0),
            ClipChain::from_regions([clip]),
        );
        list.push_text(20);
        let visuals = block_visuals(&list, 3);
        assert!(visuals[0].is_none());
        assert_eq!(
            visuals[1].as_ref().unwrap().transform.translation,
            Point::new(4.0, 5.0)
        );
        assert!(visuals[2].is_none());

        let mut text = PreparedText::default();
        text.blocks = 3;
        let mut clips = Vec::new();
        let prepared = prepare_visuals(&text, Some(&visuals), 2.0, &mut clips);
        assert_eq!(clips.len(), 1);
        assert_eq!(prepared[0].clip_count, 0);
        assert_eq!(prepared[1].clip_count, 1);
        assert_eq!(prepared[1].transform.translation, Point::new(8.0, 10.0));

        clips.clear();
        let fallback = prepare_visuals(&text, None, 1.0, &mut clips);
        assert!(fallback.iter().all(|visual| visual.clip_count == 0));
        assert!(clips.is_empty());
    }
}
