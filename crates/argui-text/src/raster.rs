use std::collections::{HashMap, VecDeque};

use cosmic_text::{FontSystem, SwashCache, SwashContent};

use crate::{GlyphContent, GlyphImage, GlyphKey};

const MAX_ENTRIES: usize = 1024;
const MAX_BYTES: usize = 8 * 1024 * 1024;

/// Cumulative text-cache activity and current CPU bitmap residency.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextStats {
    /// Logical layout requests satisfied without shaping.
    pub layout_hits: u64,
    /// Logical layout requests that created and shaped a buffer.
    pub layout_misses: u64,
    /// Raster requests served by the CPU bitmap cache, including absent glyphs.
    pub raster_hits: u64,
    /// Raster requests that invoked Swash.
    pub raster_misses: u64,
    /// Current cached bitmap and absent-glyph entries (at most 1024).
    pub raster_entries: usize,
    /// Current cached bitmap bytes (at most 8 MiB).
    pub raster_bytes: usize,
}

pub(crate) struct RasterCache {
    swash: SwashCache,
    images: HashMap<GlyphKey, Option<GlyphImage>>,
    order: VecDeque<GlyphKey>,
    pub(crate) bytes: usize,
}

impl Default for RasterCache {
    /// Creates an empty bounded raster cache with a reusable Swash scale context.
    fn default() -> Self {
        Self {
            swash: SwashCache::new(),
            images: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
        }
    }
}

impl RasterCache {
    /// Returns the number of resident bitmap and absent-glyph entries.
    pub(crate) fn len(&self) -> usize {
        self.images.len()
    }

    /// Returns `key`'s bitmap using `fonts`, plus whether the CPU cache supplied it.
    ///
    /// A missing bitmap is cached as well. Oversized bitmaps are returned without
    /// residency; least recently used entries are evicted to respect both limits.
    pub(crate) fn rasterize(
        &mut self,
        fonts: &mut FontSystem,
        key: GlyphKey,
    ) -> (Option<GlyphImage>, bool) {
        if let Some(image) = self.images.get(&key) {
            self.order.retain(|cached| *cached != key);
            self.order.push_back(key);
            return (image.clone(), true);
        }
        let image = self
            .swash
            .get_image_uncached(fonts, key.0)
            .map(|mut image| {
                // Swash composites COLR layers in premultiplied encoded RGB, whereas
                // embedded bitmap strikes already use straight RGBA.
                if matches!(image.source, swash::scale::Source::ColorOutline(_)) {
                    for pixel in image.data.as_chunks_mut::<4>().0 {
                        let alpha = u32::from(pixel[3]);
                        for channel in &mut pixel[..3] {
                            *channel = (u32::from(*channel) * 255 + alpha / 2)
                                .checked_div(alpha)
                                .unwrap_or(0)
                                .min(255) as u8;
                        }
                    }
                }
                GlyphImage {
                    left: image.placement.left,
                    top: image.placement.top,
                    width: image.placement.width,
                    height: image.placement.height,
                    content: match image.content {
                        SwashContent::Mask => GlyphContent::Mask,
                        SwashContent::Color => GlyphContent::Color,
                        SwashContent::SubpixelMask => GlyphContent::SubpixelMask,
                    },
                    data: image.data,
                }
            });
        let bytes = image.as_ref().map_or(0, |image| image.data.len());
        if bytes <= MAX_BYTES {
            while self.images.len() >= MAX_ENTRIES || self.bytes + bytes > MAX_BYTES {
                let Some(oldest) = self.order.pop_front() else {
                    break;
                };
                if let Some(Some(image)) = self.images.remove(&oldest) {
                    self.bytes -= image.data.len();
                }
            }
            self.bytes += bytes;
            self.order.push_back(key);
            self.images.insert(key, image.clone());
        }
        (image, false)
    }
}
