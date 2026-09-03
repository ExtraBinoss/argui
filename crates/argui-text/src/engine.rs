use std::sync::Arc;

use argui_core::Size;
use cosmic_text::{
    Align, Attrs, Buffer, Color, Ellipsize, EllipsizeHeightLimit, Family, FontSystem, Metrics,
    Renderer, Shaping, SwashCache, SwashContent, Weight, Wrap, fontdb, render_decoration,
};

use crate::{
    EllipsisPosition, FontFamily, FontStretch, FontStyle, GlyphContent, GlyphImage, GlyphKey,
    LetterSpacing, PreparedDecoration, PreparedGlyph, PreparedText, TextContent, TextOverflow,
    TextScene, TextSpanStyle, TextStyle, TextWrap, UnderlineStyle,
    cache::{CachedDecoration, CachedGlyph, CachedShape, MeasureKey, ShapeKey, TextCache},
};

pub struct TextEngine {
    pub(crate) fonts: FontSystem,
    rasterizer: SwashCache,
    pub(crate) input_buffers: Vec<crate::input::InputBuffer>,
    pub(crate) cache: TextCache,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self {
            fonts: FontSystem::new(),
            rasterizer: SwashCache::new(),
            input_buffers: Vec::new(),
            cache: TextCache::default(),
        }
    }
}

impl TextEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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
            input_buffers: Vec::new(),
            cache: TextCache::default(),
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontSystem {
        self.cache.clear();
        &mut self.fonts
    }

    #[must_use]
    pub fn measure(&mut self, text: &str, style: &TextStyle, width: Option<f32>) -> Size {
        self.measure_layout(text, style, width).size
    }

    #[must_use]
    pub fn measure_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        width: Option<f32>,
    ) -> crate::TextMeasurement {
        self.measure_content(&TextContent::plain(text), style, width)
    }

    #[must_use]
    pub fn measure_content(
        &mut self,
        content: &TextContent,
        style: &TextStyle,
        width: Option<f32>,
    ) -> crate::TextMeasurement {
        let key = MeasureKey::new(content, style, width);
        if let Some(measurement) = self.cache.measurement(&key) {
            return measurement;
        }
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
        self.cache.insert_measurement(key, measurement);
        measurement
    }

    pub fn prepare(&mut self, scene: &TextScene, scale_factor: f32) -> PreparedText {
        let mut prepared = PreparedText {
            blocks: scene.blocks().len(),
            scale_factor,
            ..PreparedText::default()
        };
        for (block_index, block) in scene.blocks().iter().enumerate() {
            let key = ShapeKey::new(block, scale_factor);
            let shape = if let Some(shape) = self.cache.shape(&key) {
                shape
            } else {
                let shape = self.shape_block(block, scale_factor, key.subpixel_origin());
                self.cache.insert_shape(key, shape)
            };
            append_shape(&mut prepared, block_index, block, scale_factor, &shape);
        }
        prepared
    }

    fn shape_block(
        &mut self,
        block: &crate::TextBlock,
        scale_factor: f32,
        subpixel_origin: [f32; 2],
    ) -> CachedShape {
        let mut buffer = Buffer::new(&mut self.fonts, metrics(&block.style));
        configure_buffer(
            &mut buffer,
            &block.style,
            Some(block.bounds.size.width),
            Some(block.bounds.size.height),
        );
        set_content(&mut buffer, &block.content, &block.style);
        buffer.shape_until_scroll(&mut self.fonts, false);

        let default_color = cosmic_color(block.style.color);
        let mut glyphs = Vec::new();
        let mut decorations = DecorationCollector::new(scale_factor);
        let line_offsets = source_line_offsets(block.content.as_str());
        for run in buffer.layout_runs() {
            let line_start = line_offsets.get(run.line_i).copied().unwrap_or_default();
            for glyph in run.glyphs {
                let physical = glyph.physical(
                    (
                        subpixel_origin[0],
                        subpixel_origin[1] + run.line_y * scale_factor,
                    ),
                    scale_factor,
                );
                glyphs.push(CachedGlyph {
                    key: GlyphKey(physical.cache_key),
                    start: line_start + glyph.start,
                    end: line_start + glyph.end,
                    rtl: glyph.level.is_rtl(),
                    local: [physical.x, physical.y],
                    color: from_cosmic(glyph.color_opt.unwrap_or(default_color)),
                });
            }
            render_decoration(&mut decorations, &run, default_color);
        }
        CachedShape {
            glyphs: Arc::from(glyphs),
            decorations: Arc::from(decorations.items),
        }
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

pub(crate) fn set_content(buffer: &mut Buffer, content: &TextContent, style: &TextStyle) {
    let defaults = attrs(style);
    if content.is_rich() {
        buffer.set_rich_text(
            content
                .runs()
                .iter()
                .map(|(range, span)| (&content.as_str()[range.clone()], span_attrs(style, span))),
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

fn append_shape(
    prepared: &mut PreparedText,
    block_index: usize,
    block: &crate::TextBlock,
    scale_factor: f32,
    shape: &CachedShape,
) {
    let anchor = [
        (block.bounds.origin.x * scale_factor).round() as i32,
        (block.bounds.origin.y * scale_factor).round() as i32,
    ];
    let clip = [
        block.clip.origin.x * scale_factor,
        block.clip.origin.y * scale_factor,
        (block.clip.origin.x + block.clip.size.width) * scale_factor,
        (block.clip.origin.y + block.clip.size.height) * scale_factor,
    ];
    prepared
        .glyphs
        .extend(shape.glyphs.iter().map(|glyph| PreparedGlyph {
            key: glyph.key,
            block: block_index,
            start: glyph.start,
            end: glyph.end,
            rtl: glyph.rtl,
            x: anchor[0] + glyph.local[0],
            y: anchor[1] + glyph.local[1],
            color: glyph.color,
            clip,
            local: glyph.local,
        }));
    prepared.decorations.extend(
        shape
            .decorations
            .iter()
            .map(|decoration| PreparedDecoration {
                block: block_index,
                rect: [
                    (anchor[0] + decoration.local[0]) as f32,
                    (anchor[1] + decoration.local[1]) as f32,
                    decoration.local[2] as f32,
                    decoration.local[3] as f32,
                ],
                color: decoration.color,
                local: decoration.local,
            }),
    );
}

struct DecorationCollector {
    scale: f32,
    items: Vec<CachedDecoration>,
}

impl DecorationCollector {
    fn new(scale: f32) -> Self {
        Self {
            scale,
            items: Vec::new(),
        }
    }
}

impl Renderer for DecorationCollector {
    fn rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        self.items.push(CachedDecoration {
            local: [
                (x as f32 * self.scale).round() as i32,
                (y as f32 * self.scale).round() as i32,
                (width as f32 * self.scale).ceil() as i32,
                (height as f32 * self.scale).ceil() as i32,
            ],
            color: from_cosmic(color),
        });
    }

    fn glyph(&mut self, _glyph: cosmic_text::PhysicalGlyph, _color: Color) {}
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

fn from_cosmic(value: Color) -> [f32; 4] {
    value.as_rgba().map(|channel| f32::from(channel) / 255.0)
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
