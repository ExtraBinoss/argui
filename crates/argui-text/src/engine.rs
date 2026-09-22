use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

static FONT_GENERATION: AtomicUsize = AtomicUsize::new(1);

use argui_core::Size;
use cosmic_text::{
    Align, Attrs, Buffer, Color, Ellipsize, EllipsizeHeightLimit, Family, FontSystem, Metrics,
    Shaping, Weight, Wrap, fontdb,
};

use crate::{
    EllipsisPosition, FontFamily, FontStretch, FontStyle, GlyphImage, GlyphKey, LetterSpacing,
    PreparedText, TextContent, TextOverflow, TextScene, TextSpanStyle, TextStyle, TextWrap,
    UnderlineStyle,
    cache::{CachedGlyph, CachedShape, MeasureKey, TextCache, TextLayoutKey},
    layout::LogicalGlyph,
    paint::{append_shape, collect_decorations},
    raster::RasterCache,
};

pub struct TextEngine {
    pub(crate) fonts: FontSystem,
    font_generation: usize,
    rasterizer: RasterCache,
    pub(crate) stats: crate::TextStats,
    pub(crate) input_buffers: Vec<crate::input::InputBuffer>,
    pub(crate) cache: TextCache,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self {
            fonts: FontSystem::new(),
            font_generation: FONT_GENERATION.fetch_add(1, Ordering::Relaxed),
            rasterizer: RasterCache::default(),
            stats: crate::TextStats::default(),
            input_buffers: Vec::new(),
            cache: TextCache::default(),
        }
    }
}

impl TextEngine {
    /// Creates a text engine using the system font database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an engine using embedded font data and explicit family fallbacks.
    ///
    /// * `fonts` — static font-file byte slices to load.
    /// * `sans_serif` — configured sans-serif family name.
    /// * `serif` — configured serif family name.
    /// * `monospace` — configured monospace family name.
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
            font_generation: FONT_GENERATION.fetch_add(1, Ordering::Relaxed),
            rasterizer: RasterCache::default(),
            stats: crate::TextStats::default(),
            input_buffers: Vec::new(),
            cache: TextCache::default(),
        }
    }

    /// Returns mutable access to fonts, invalidating all layout and raster caches.
    ///
    /// Existing prepared text must be prepared again after changing fonts.
    pub fn fonts_mut(&mut self) -> &mut FontSystem {
        self.font_generation = FONT_GENERATION.fetch_add(1, Ordering::Relaxed);
        self.cache.clear();
        self.input_buffers.clear();
        self.rasterizer = RasterCache::default();
        &mut self.fonts
    }

    /// Returns cumulative cache activity and current bounded CPU bitmap residency.
    #[must_use]
    pub fn stats(&self) -> crate::TextStats {
        crate::TextStats {
            raster_entries: self.rasterizer.len(),
            raster_bytes: self.rasterizer.bytes,
            ..self.stats
        }
    }

    /// Measures plain text with the supplied style and optional width constraint.
    ///
    /// * `text` — UTF-8 text to measure.
    /// * `style` — shaping and line-layout configuration.
    /// * `width` — optional logical-pixel width constraint.
    #[must_use]
    pub fn measure(&mut self, text: &str, style: &TextStyle, width: Option<f32>) -> Size {
        self.measure_layout(text, style, width).size
    }

    /// Measures plain text and returns its size and baseline positions.
    ///
    /// * `text` — UTF-8 text to measure.
    /// * `style` — shaping and line-layout configuration.
    /// * `width` — optional logical-pixel width constraint.
    #[must_use]
    pub fn measure_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        width: Option<f32>,
    ) -> crate::TextMeasurement {
        self.measure_content(&TextContent::plain(text), style, width)
    }

    /// Measures plain or rich text and returns its size and baseline positions.
    ///
    /// * `content` — text and optional span styles to measure.
    /// * `style` — block-level shaping and line-layout configuration.
    /// * `width` — optional logical-pixel width constraint.
    #[must_use]
    pub fn measure_content(
        &mut self,
        content: &TextContent,
        style: &TextStyle,
        width: Option<f32>,
    ) -> crate::TextMeasurement {
        let key = MeasureKey::new(content, style, width);
        if let Some(measurement) = self.cache.measurement(&key) {
            self.stats.layout_hits += 1;
            return measurement;
        }
        self.stats.layout_misses += 1;
        let mut buffer = Buffer::new(&mut self.fonts, metrics(style));
        configure_buffer(&mut buffer, style, width, None);
        set_content(&mut buffer, content, style);
        buffer.shape_until_scroll(&mut self.fonts, false);

        let mut measurement = crate::TextMeasurement::default();
        for run in buffer.layout_runs() {
            measurement.size.width = measurement.size.width.max(run.line_w);
            measurement.size.height = measurement.size.height.max(run.line_top + run.line_height);
            measurement.first_baseline.get_or_insert(run.line_y);
            measurement.last_baseline = Some(run.line_y);
        }
        // Layout rounds boxes to pixels. Never round intrinsic text below its shaped width.
        measurement.size.width = measurement.size.width.ceil();
        self.cache.insert_measurement(key, measurement);
        measurement
    }

    /// Prepares physical glyphs and decorations, reusing paint-independent logical layouts.
    ///
    /// * `scene` — blocks to shape.
    /// * `scale_factor` — logical-to-physical pixel scale.
    pub fn prepare(&mut self, scene: &TextScene, scale_factor: f32) -> PreparedText {
        let mut prepared = PreparedText {
            blocks: scene.blocks().len(),
            scale_factor,
            ..PreparedText::default()
        };
        for (block_index, block) in scene.blocks().iter().enumerate() {
            let key = TextLayoutKey::new(&block.content, &block.style, block.bounds.size);
            let shape = if let Some(shape) = self.cache.shape(&key) {
                self.stats.layout_hits += 1;
                shape
            } else {
                self.stats.layout_misses += 1;
                let shape = self.shape_block(block);
                self.cache.insert_shape(key, shape)
            };
            append_shape(&mut prepared, block_index, block, scale_factor, &shape);
        }
        prepared
    }

    /// Shapes `block` into reusable logical glyph and decoration geometry.
    fn shape_block(&mut self, block: &crate::TextBlock) -> CachedShape {
        let mut buffer = Buffer::new(&mut self.fonts, metrics(&block.style));
        configure_buffer(
            &mut buffer,
            &block.style,
            Some(block.bounds.size.width),
            Some(block.bounds.size.height),
        );
        set_content(&mut buffer, &block.content, &block.style);
        buffer.shape_until_scroll(&mut self.fonts, false);

        let mut glyphs = Vec::new();
        let mut decorations = Vec::new();
        let line_offsets = source_line_offsets(block.content.as_str());
        for run in buffer.layout_runs() {
            let line_start = line_offsets.get(run.line_i).copied().unwrap_or_default();
            for glyph in run.glyphs {
                glyphs.push(CachedGlyph {
                    key: GlyphKey(
                        glyph.physical((0.0, 0.0), 1.0).cache_key,
                        self.font_generation,
                    ),
                    start: line_start + glyph.start,
                    end: line_start + glyph.end,
                    rtl: glyph.level.is_rtl(),
                    local: LogicalGlyph::new(glyph, run.line_y),
                    span: glyph.metadata,
                });
            }
            collect_decorations(&mut decorations, &run);
        }
        CachedShape {
            glyphs: Arc::from(glyphs),
            decorations: Arc::from(decorations),
        }
    }

    /// Returns a glyph bitmap, reusing a bounded CPU raster cache when possible.
    ///
    /// * `key` — glyph cache key produced by text shaping.
    ///
    /// Returns `None` when the rasterizer has no image for the key.
    pub fn rasterize(&mut self, key: GlyphKey) -> Option<GlyphImage> {
        let (image, hit) = self.rasterizer.rasterize(&mut self.fonts, key);
        if hit {
            self.stats.raster_hits += 1;
        } else {
            self.stats.raster_misses += 1;
        }
        image
    }
}

pub(crate) fn configure_buffer(
    buffer: &mut Buffer,
    style: &TextStyle,
    width: Option<f32>,
    height: Option<f32>,
) {
    let width = (style.wrap != TextWrap::None
        || matches!(style.overflow, TextOverflow::Ellipsis(_))
        || !matches!(
            style.align,
            crate::TextAlign::Start | crate::TextAlign::Left
        ))
    .then_some(width)
    .flatten();
    let clamped_height = style
        .line_clamp
        .map(|lines| style.line_height * lines.get() as f32);
    let height = match (height, clamped_height) {
        (Some(height), Some(clamp)) => Some(height.min(clamp)),
        (height, None) => height,
        (None, clamp) => clamp,
    };
    buffer.set_size(width, height);
    buffer.set_wrap(wrap(style.wrap));
    if let TextOverflow::Ellipsis(position) = style.overflow {
        let limit = style.line_clamp.map_or_else(
            || {
                if style.wrap == TextWrap::None {
                    EllipsizeHeightLimit::Lines(1)
                } else {
                    EllipsizeHeightLimit::Height(height.unwrap_or(f32::MAX))
                }
            },
            |lines| EllipsizeHeightLimit::Lines(lines.get()),
        );
        buffer.set_ellipsize(match position {
            EllipsisPosition::Start => Ellipsize::Start(limit),
            EllipsisPosition::Middle => Ellipsize::Middle(limit),
            EllipsisPosition::End => Ellipsize::End(limit),
        });
    }
}

/// Configures `buffer` with `content` and layout `style`; span metadata identifies paint sources.
pub(crate) fn set_content(buffer: &mut Buffer, content: &TextContent, style: &TextStyle) {
    let defaults = attrs(style);
    if content.is_rich() {
        buffer.set_rich_text(
            content
                .runs()
                .iter()
                .enumerate()
                .map(|(index, (range, span))| {
                    (
                        &content.as_str()[range.clone()],
                        span_attrs(style, span).metadata(index + 1),
                    )
                }),
            &defaults,
            Shaping::Advanced,
            text_align(style.align),
        );
    } else {
        buffer.set_text(
            content.as_str(),
            &defaults,
            Shaping::Advanced,
            text_align(style.align),
        );
    }
}

pub(crate) fn attrs(style: &TextStyle) -> Attrs<'_> {
    apply_decoration(
        Attrs::new()
            .family(family(&style.family))
            .weight(Weight(style.weight))
            .style(font_style(style.font_style))
            .stretch(font_stretch(style.stretch))
            .color(cosmic_color(style.color)),
        style.decoration,
    )
    .letter_spacing(letter_spacing(style.letter_spacing, style.font_size))
}

fn span_attrs<'a>(base: &'a TextStyle, span: &'a TextSpanStyle) -> Attrs<'a> {
    let font_size = span.font_size.unwrap_or(base.font_size);
    let line_height = span.line_height.unwrap_or(base.line_height);
    let family = span.family.as_ref().unwrap_or(&base.family);
    let mut value = Attrs::new()
        .family(self::family(family))
        .weight(Weight(span.weight.unwrap_or(base.weight)))
        .style(font_style(span.font_style.unwrap_or(base.font_style)))
        .stretch(font_stretch(span.stretch.unwrap_or(base.stretch)))
        .color(cosmic_color(span.color.unwrap_or(base.color)))
        .metrics(Metrics::new(font_size, line_height));
    value = apply_decoration(value, span.decoration.unwrap_or(base.decoration));
    value.letter_spacing(letter_spacing(
        span.letter_spacing.unwrap_or(base.letter_spacing),
        font_size,
    ))
}

fn apply_decoration(mut attrs: Attrs<'_>, value: crate::TextDecoration) -> Attrs<'_> {
    attrs = attrs.underline(match value.underline {
        UnderlineStyle::None => cosmic_text::UnderlineStyle::None,
        UnderlineStyle::Single => cosmic_text::UnderlineStyle::Single,
        UnderlineStyle::Double => cosmic_text::UnderlineStyle::Double,
    });
    if let Some(color) = value.underline_color {
        attrs = attrs.underline_color(cosmic_color(color));
    }
    if value.strikethrough {
        attrs = attrs.strikethrough();
    }
    if let Some(color) = value.strikethrough_color {
        attrs = attrs.strikethrough_color(cosmic_color(color));
    }
    attrs
}

fn metrics(style: &TextStyle) -> Metrics {
    Metrics::new(style.font_size, style.line_height)
}

fn letter_spacing(value: LetterSpacing, font_size: f32) -> f32 {
    match value {
        LetterSpacing::Normal => 0.0,
        LetterSpacing::Px(value) => value / font_size.max(f32::EPSILON),
        LetterSpacing::Em(value) => value,
    }
}

fn cosmic_color(value: argui_core::Color) -> Color {
    let [r, g, b, a] = value.to_linear_rgba();
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::rgba(channel(r), channel(g), channel(b), channel(a))
}

pub(crate) fn source_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    offsets.extend(
        text.char_indices()
            .filter_map(|(index, character)| (character == '\n').then_some(index + 1)),
    );
    offsets
}

pub(crate) const fn text_align(align: crate::TextAlign) -> Option<Align> {
    match align {
        crate::TextAlign::Start => None,
        crate::TextAlign::End => Some(Align::End),
        crate::TextAlign::Left => Some(Align::Left),
        crate::TextAlign::Right => Some(Align::Right),
        crate::TextAlign::Center => Some(Align::Center),
        crate::TextAlign::Justify => Some(Align::Justified),
    }
}

pub(crate) fn family(value: &FontFamily) -> Family<'_> {
    match value {
        FontFamily::SansSerif => Family::SansSerif,
        FontFamily::Serif => Family::Serif,
        FontFamily::Monospace => Family::Monospace,
        FontFamily::Named(name) => Family::Name(name),
    }
}

const fn font_style(value: FontStyle) -> fontdb::Style {
    match value {
        FontStyle::Normal => fontdb::Style::Normal,
        FontStyle::Italic => fontdb::Style::Italic,
        FontStyle::Oblique => fontdb::Style::Oblique,
    }
}

const fn font_stretch(value: FontStretch) -> fontdb::Stretch {
    match value {
        FontStretch::UltraCondensed => fontdb::Stretch::UltraCondensed,
        FontStretch::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
        FontStretch::Condensed => fontdb::Stretch::Condensed,
        FontStretch::SemiCondensed => fontdb::Stretch::SemiCondensed,
        FontStretch::Normal => fontdb::Stretch::Normal,
        FontStretch::SemiExpanded => fontdb::Stretch::SemiExpanded,
        FontStretch::Expanded => fontdb::Stretch::Expanded,
        FontStretch::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
        FontStretch::UltraExpanded => fontdb::Stretch::UltraExpanded,
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
