use std::collections::{HashMap, HashSet, VecDeque};

use argui_text::{GlyphKey, TextEngine};

use super::{pages::AtlasPages, pixels::padded_pixels};
use crate::RendererError;

const PAGE_SIZE: u32 = 1024;
const MASK_PAGES: usize = 4;
const COLOR_PAGES: usize = 2;
const EMPTY_CAPACITY: usize = 4096;

/// Bounded glyph residency and work performed during the most recent preparation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextAtlasStats {
    /// Resident drawable glyphs across both texture arrays.
    pub entries: usize,
    /// Glyph lookups served by existing atlas or empty-glyph entries.
    pub hits_this_frame: usize,
    /// Requests sent to the CPU raster cache during this preparation.
    pub raster_requests_this_frame: usize,
    /// Padded texture bytes uploaded during this preparation.
    pub uploaded_bytes_this_frame: u64,
    /// Old pages evicted during this preparation.
    pub evictions_this_frame: usize,
    /// Populated monochrome pages, out of four available pages.
    pub mask_pages: usize,
    /// Populated color pages, out of two available pages.
    pub color_pages: usize,
    /// Fixed GPU residency budget including currently empty texture-array layers.
    pub allocated_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub color: bool,
    pub page: usize,
}

struct AtlasTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pages: AtlasPages,
}

impl AtlasTexture {
    /// Creates `count` bounded array layers in `format` using `device`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, count: usize) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("argui-glyph-atlas"),
            size: wgpu::Extent3d {
                width: PAGE_SIZE,
                height: PAGE_SIZE,
                depth_or_array_layers: count as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        Self {
            texture,
            view,
            pages: AtlasPages::new(PAGE_SIZE, count),
        }
    }
}

pub(super) struct GlyphAtlas {
    mask: AtlasTexture,
    color: AtlasTexture,
    sampler: wgpu::Sampler,
    entries: HashMap<GlyphKey, AtlasEntry>,
    empty: HashSet<GlyphKey>,
    empty_order: VecDeque<GlyphKey>,
    stats: TextAtlasStats,
}

impl GlyphAtlas {
    /// Creates separate mask and sRGB color arrays with a total 12 MiB budget.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            mask: AtlasTexture::new(device, wgpu::TextureFormat::R8Unorm, MASK_PAGES),
            color: AtlasTexture::new(device, wgpu::TextureFormat::Rgba8UnormSrgb, COLOR_PAGES),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("argui-glyph-sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            entries: HashMap::new(),
            empty: HashSet::new(),
            empty_order: VecDeque::new(),
            stats: TextAtlasStats::default(),
        }
    }

    /// Returns the common square page side in physical texels.
    pub const fn size(&self) -> u32 {
        PAGE_SIZE
    }

    /// Returns the monochrome coverage texture-array view.
    pub const fn mask_view(&self) -> &wgpu::TextureView {
        &self.mask.view
    }

    /// Returns the sRGB color texture-array view.
    pub const fn color_view(&self) -> &wgpu::TextureView {
        &self.color.view
    }

    /// Returns the shared linear sampler without mipmaps.
    pub const fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    /// Clears preparation counters without changing resident glyphs or page pins.
    pub fn clear_frame_stats(&mut self) {
        self.stats = TextAtlasStats::default();
    }

    /// Protects every resident glyph in `keys` before allocating any new glyphs.
    /// This prevents eviction of layers referenced by earlier or later frame instances.
    pub fn begin_frame(&mut self, keys: impl IntoIterator<Item = GlyphKey>) {
        self.clear_frame_stats();
        self.mask.pages.begin_frame();
        self.color.pages.begin_frame();
        for key in keys {
            if let Some(entry) = self.entries.get(&key) {
                let texture = if entry.color {
                    &mut self.color
                } else {
                    &mut self.mask
                };
                texture.pages.pin(entry.page);
            }
        }
    }

    /// Returns residency and preparation counters for both bounded texture arrays.
    pub fn stats(&self) -> TextAtlasStats {
        TextAtlasStats {
            entries: self.entries.len(),
            mask_pages: self.mask.pages.used(),
            color_pages: self.color.pages.used(),
            allocated_bytes: u64::from(PAGE_SIZE).pow(2) * (MASK_PAGES + COLOR_PAGES * 4) as u64,
            ..self.stats
        }
    }

    /// Resolves `key`, rasterizing through `engine` and uploading via `queue` on misses.
    /// Returns the resident entry, or `None` for a glyph without visible pixels.
    ///
    /// # Errors
    /// Returns `GlyphAtlasFull` when the glyph is oversized or the visible frame
    /// cannot fit without evicting a page already referenced by that frame.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn get_or_insert(
        &mut self,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        key: GlyphKey,
    ) -> Result<Option<AtlasEntry>, RendererError> {
        if let Some(entry) = self.entries.get(&key) {
            self.stats.hits_this_frame += 1;
            return Ok(Some(*entry));
        }
        if self.empty.contains(&key) {
            self.stats.hits_this_frame += 1;
            return Ok(None);
        }
        self.stats.raster_requests_this_frame += 1;
        let Some(image) = engine
            .rasterize(key)
            .filter(|image| image.width != 0 && image.height != 0)
        else {
            if self.empty.len() == EMPTY_CAPACITY
                && let Some(oldest) = self.empty_order.pop_front()
            {
                self.empty.remove(&oldest);
            }
            self.empty.insert(key);
            self.empty_order.push_back(key);
            return Ok(None);
        };
        let color = image.content == argui_text::GlyphContent::Color;
        let texture = if color {
            &mut self.color
        } else {
            &mut self.mask
        };
        let allocation = texture
            .pages
            .allocate(image.width, image.height)
            .ok_or(RendererError::GlyphAtlasFull)?;
        if allocation.evicted {
            self.entries
                .retain(|_, entry| entry.color != color || entry.page != allocation.page);
            self.stats.evictions_this_frame += 1;
        }
        let pixels = padded_pixels(&image);
        let channels = if color { 4 } else { 1 };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.origin[0],
                    y: allocation.origin[1],
                    z: allocation.page as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((image.width + 2) * channels),
                rows_per_image: Some(image.height + 2),
            },
            wgpu::Extent3d {
                width: image.width + 2,
                height: image.height + 2,
                depth_or_array_layers: 1,
            },
        );
        self.stats.uploaded_bytes_this_frame += pixels.len() as u64;
        let entry = AtlasEntry {
            x: allocation.origin[0] + 1,
            y: allocation.origin[1] + 1,
            width: image.width,
            height: image.height,
            left: image.left,
            top: image.top,
            color,
            page: allocation.page,
        };
        self.entries.insert(key, entry);
        Ok(Some(entry))
    }
}
