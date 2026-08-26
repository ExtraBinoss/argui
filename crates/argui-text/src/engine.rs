use std::sync::Arc;

use argui_core::Size;
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, SwashContent, Weight, Wrap,
    fontdb,
};

use crate::{
    FontFamily, GlyphContent, GlyphImage, GlyphKey, PreparedGlyph, PreparedText, TextScene,
    TextWrap,
};

pub struct TextEngine {
    fonts: FontSystem,
    rasterizer: SwashCache,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self {
            fonts: FontSystem::new(),
            rasterizer: SwashCache::new(),
        }
    }
}

impl TextEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an isolated font system from assets owned by the application.
    #[must_use]
    pub fn from_embedded_fonts(
        fonts: impl IntoIterator<Item = &'static [u8]>,
        sans_serif: &str,
        serif: &str,
        monospace: &str,
    ) -> Self {
        let mut database = fontdb::Database::new();
        for font in fonts {
            let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(font);
            database.load_font_source(fontdb::Source::Binary(data));
        }
        database.set_sans_serif_family(sans_serif);
        database.set_serif_family(serif);
        database.set_monospace_family(monospace);

        Self {
            fonts: FontSystem::new_with_locale_and_db("en-US".into(), database),
            rasterizer: SwashCache::new(),
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontSystem {
        &mut self.fonts
    }

    #[must_use]
    pub fn measure(&mut self, text: &str, style: &crate::TextStyle, width: Option<f32>) -> Size {
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(&mut self.fonts, metrics);
        buffer.set_size(width, None);
        buffer.set_wrap(wrap(style.wrap));
        let family = family(&style.family);
        let attrs = Attrs::new().family(family).weight(Weight(style.weight));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.fonts, false);

        let mut measured = Size::default();
        for run in buffer.layout_runs() {
            measured.width = measured.width.max(run.line_w);
            measured.height = measured.height.max(run.line_top + run.line_height);
        }
        measured
    }

    pub fn prepare(&mut self, scene: &TextScene, scale_factor: f32) -> PreparedText {
        let mut prepared = PreparedText {
            blocks: scene.blocks().len(),
            ..PreparedText::default()
        };
        for (block_index, block) in scene.blocks().iter().enumerate() {
            let metrics = Metrics::new(block.style.font_size, block.style.line_height);
            let mut buffer = Buffer::new(&mut self.fonts, metrics);
            buffer.set_size(
                Some(block.bounds.size.width),
                Some(block.bounds.size.height),
            );
            buffer.set_wrap(wrap(block.style.wrap));

            let family = family(&block.style.family);
            let attrs = Attrs::new()
                .family(family)
                .weight(Weight(block.style.weight));
            buffer.set_text(&block.text, &attrs, Shaping::Advanced, None);
            buffer.shape_until_scroll(&mut self.fonts, false);

            let offset_x = block.bounds.origin.x * scale_factor;
            let offset_y = block.bounds.origin.y * scale_factor;
            let clip = [
                block.clip.origin.x * scale_factor,
                block.clip.origin.y * scale_factor,
                (block.clip.origin.x + block.clip.size.width) * scale_factor,
                (block.clip.origin.y + block.clip.size.height) * scale_factor,
            ];
            for run in buffer.layout_runs() {
                for glyph in run.glyphs {
                    let (start, end, rtl) = (glyph.start, glyph.end, glyph.level.is_rtl());
                    let glyph = glyph.physical(
                        (offset_x, offset_y + run.line_y * scale_factor),
                        scale_factor,
                    );
                    prepared.glyphs.push(PreparedGlyph {
                        key: GlyphKey(glyph.cache_key),
                        block: block_index,
                        start,
                        end,
                        rtl,
                        x: glyph.x,
                        y: glyph.y,
                        color: block.style.color.as_array(),
                        clip,
                    });
                }
            }
        }
        prepared
    }

    pub fn rasterize(&mut self, key: GlyphKey) -> Option<GlyphImage> {
        let image = self.rasterizer.get_image_uncached(&mut self.fonts, key.0)?;
        let content = match image.content {
            SwashContent::Mask => GlyphContent::Mask,
            SwashContent::Color => GlyphContent::Color,
            SwashContent::SubpixelMask => GlyphContent::SubpixelMask,
        };
        Some(GlyphImage {
            left: image.placement.left,
            top: image.placement.top,
            width: image.placement.width,
            height: image.placement.height,
            content,
            data: image.data,
        })
    }
}

fn family(value: &FontFamily) -> Family<'_> {
    match value {
        FontFamily::SansSerif => Family::SansSerif,
        FontFamily::Serif => Family::Serif,
        FontFamily::Monospace => Family::Monospace,
        FontFamily::Named(name) => Family::Name(name),
    }
}

const fn wrap(value: TextWrap) -> Wrap {
    match value {
        TextWrap::None => Wrap::None,
        TextWrap::Glyph => Wrap::Glyph,
        TextWrap::Word => Wrap::Word,
        TextWrap::WordOrGlyph => Wrap::WordOrGlyph,
    }
}
