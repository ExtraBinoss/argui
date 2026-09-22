mod atlas;
mod pages;
mod pipeline;
mod pixels;

pub use atlas::TextAtlasStats;

use std::ops::Range;

use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, DisplayCommand, DisplayList};
use argui_text::{PreparedText, TextEngine};

use crate::RendererError;
use atlas::GlyphAtlas;
use pipeline::{GlyphInstance, TextClip, TextPipeline};

type PreparedGlyphs = (
    Vec<GlyphInstance>,
    Vec<Range<u32>>,
    Vec<TextClip>,
    Vec<Option<Rect>>,
);

#[derive(Clone, Debug)]
struct BlockVisual {
    transform: Affine2D,
    clips: ClipChain,
    backdrop: Option<[f32; 3]>,
}

#[derive(Clone, Copy)]
struct PreparedVisual {
    active: bool,
    transform: Affine2D,
    clip_start: u32,
    clip_count: u32,
    backdrop: Option<[f32; 3]>,
}

pub(crate) struct TextGpu {
    atlas: GlyphAtlas,
    pipeline: TextPipeline,
}

pub(crate) struct TextDraw {
    ranges: Vec<Range<u32>>,
    bounds: Vec<Option<Rect>>,
    changed: bool,
}

impl TextDraw {
    pub fn all(&self) -> Range<u32> {
        0..self.ranges.iter().map(|range| range.end).max().unwrap_or(0)
    }

    pub fn ranges(&self) -> &[Range<u32>] {
        &self.ranges
    }

    /// Returns conservative physical bounds for every prepared text block.
    pub fn bounds(&self) -> &[Option<Rect>] {
        &self.bounds
    }

    pub const fn changed(&self) -> bool {
        self.changed
    }
}

impl TextGpu {
    /// Creates the glyph renderer on `device` for a linear-blended target `format`.
    /// Allocates bounded mask and sRGB color arrays and their shared instance pipeline.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let atlas = GlyphAtlas::new(device);
        let pipeline = TextPipeline::new(
            device,
            format,
            atlas.mask_view(),
            atlas.color_view(),
            atlas.sampler(),
        );
        Self { atlas, pipeline }
    }

    /// Prepares physical `text` without UI transforms, resolving atlas misses through
    /// `engine` and uploading changed data using `device` and `queue`.
    /// Returns ordered draw ranges and physical damage bounds.
    ///
    /// # Errors
    /// Returns an atlas capacity error when the visible glyph set cannot fit.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
    ) -> Result<TextDraw, RendererError> {
        self.prepare_with_visuals(device, queue, engine, text, None, 1.0)
    }

    /// Prepares `text` with `display_list` transforms and clips at window `scale_factor`.
    /// Uses `engine` to rasterize positive uniform scales and fractional translations
    /// at their final physical positions; `device` and `queue` receive changed GPU data.
    /// Returns ordered draw ranges and conservative physical damage bounds.
    ///
    /// # Errors
    /// Returns an atlas capacity error when the visible glyph set cannot fit.
    #[cfg_attr(coverage_nightly, coverage(off))]
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

    #[cfg_attr(coverage_nightly, coverage(off))]
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
        let (instances, ranges, clips, bounds) =
            self.prepare_once(queue, engine, text, visuals.as_deref(), scale_factor)?;
        let changed = self.pipeline.write(device, queue, &instances, &clips)
            || self.atlas.stats().uploaded_bytes_this_frame != 0;
        Ok(TextDraw {
            ranges,
            bounds,
            changed,
        })
    }

    /// Clears per-frame atlas work counters while preserving resident glyphs.
    pub fn clear_frame_stats(&mut self) {
        self.atlas.clear_frame_stats();
    }

    /// Returns bounded atlas residency and work counters for the last preparation.
    pub fn stats(&self) -> TextAtlasStats {
        self.atlas.stats()
    }

    /// Resolves physical glyphs from `text`, optional display `visuals` and window
    /// `scale_factor`, uploading cache misses through `engine` and `queue`.
    /// Returns instances, block ranges, physical clips and conservative damage bounds.
    ///
    /// # Errors
    /// Returns the atlas capacity error when the visible glyph set cannot fit.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn prepare_once(
        &mut self,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        text: &PreparedText,
        visuals: Option<&[Option<BlockVisual>]>,
        scale_factor: f32,
    ) -> Result<PreparedGlyphs, RendererError> {
        let mut instances = Vec::with_capacity(text.glyphs.len() + text.decorations.len());
        let mut ranges = vec![0..0; text.blocks];
        let mut bounds = vec![None; text.blocks];
        let mut clips = Vec::new();
        let prepared = prepare_visuals(text, visuals, scale_factor, &mut clips);
        let mut drawable = vec![Vec::new(); text.blocks];
        for glyph in &text.glyphs {
            let visual = prepared[glyph.block];
            if !visual.active {
                continue;
            }
            let (glyph, transform) = glyph
                .transform_for_raster(visual.transform)
                .map_or((*glyph, visual.transform), |glyph| {
                    (glyph, Affine2D::IDENTITY)
                });
            drawable[glyph.block].push((glyph, transform));
        }
        self.atlas
            .begin_frame(drawable.iter().flatten().map(|(glyph, _)| glyph.key));
        for (block, visual) in prepared.iter().copied().enumerate() {
            if !visual.active {
                continue;
            }
            let start = instances.len() as u32;
            instances.extend(
                text.decorations
                    .iter()
                    .filter(|item| item.block == block)
                    .map(|item| {
                        GlyphInstance::solid(
                            *item,
                            visual.transform,
                            visual.clip_start,
                            visual.clip_count,
                        )
                    }),
            );
            for (glyph, transform) in &drawable[block] {
                let Some(entry) = self.atlas.get_or_insert(queue, engine, glyph.key)? else {
                    continue;
                };
                let instance = GlyphInstance::new(
                    *glyph,
                    entry,
                    self.atlas.size(),
                    *transform,
                    visual.clip_start,
                    visual.clip_count,
                    visual.backdrop,
                );
                let physical = instance.physical_bounds();
                let clip_range =
                    visual.clip_start as usize..(visual.clip_start + visual.clip_count) as usize;
                if clips[clip_range]
                    .iter()
                    .all(|clip| clip.intersects(physical))
                {
                    instances.push(instance);
                }
            }
            let end = instances.len() as u32;
            if start != end {
                ranges[block] = start..end;
                bounds[block] = instances[start as usize..end as usize]
                    .iter()
                    .copied()
                    .map(GlyphInstance::physical_bounds)
                    .reduce(union);
            }
        }
        Ok((instances, ranges, clips, bounds))
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        instances: Range<u32>,
        viewport_offset: u32,
    ) {
        self.pipeline.draw(pass, instances, viewport_offset);
    }
}

/// Returns the axis-aligned union of two physical glyph rectangles.
fn union(left: Rect, right: Rect) -> Rect {
    let left_edge = left.origin.x.min(right.origin.x);
    let top = left.origin.y.min(right.origin.y);
    let right_edge = (left.origin.x + left.size.width).max(right.origin.x + right.size.width);
    let bottom = (left.origin.y + left.size.height).max(right.origin.y + right.size.height);
    Rect::new(
        Point::new(left_edge, top),
        Size::new(right_edge - left_edge, bottom - top),
    )
}

fn block_visuals(display_list: &DisplayList, blocks: usize) -> Vec<Option<BlockVisual>> {
    let mut visuals = vec![None; blocks];
    for command in display_list.commands() {
        if let DisplayCommand::Text {
            block,
            transform,
            clips,
            backdrop,
        } = command
            && let Some(slot) = visuals.get_mut(*block)
        {
            *slot = Some(BlockVisual {
                transform: *transform,
                clips: clips.clone(),
                backdrop: backdrop.map(|color| {
                    let [red, green, blue, _] = color.to_linear_rgba();
                    [red, green, blue]
                }),
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
            let visual = visuals
                .and_then(|items| items.get(block))
                .and_then(Option::as_ref);
            let transform = visual.map_or(Affine2D::IDENTITY, |visual| {
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
                active: visuals.is_none() || visual.is_some(),
                transform,
                clip_start: start,
                clip_count: clips.len() as u32 - start,
                backdrop: visual.and_then(|visual| visual.backdrop),
            }
        })
        .collect()
}
