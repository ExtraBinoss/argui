use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    FontFamily, FontStretch, FontStyle, GlyphKey, LetterSpacing, TextAlign, TextDecoration,
    TextLayout, TextMeasurement, TextOverflow, TextSpanStyle, TextStyle, TextWrap, UnderlineStyle,
};

const CACHE_CAPACITY: usize = 512;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct MeasureKey {
    text: String,
    runs: Vec<RunKey>,
    style: StyleKey,
    width: Option<u32>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct TextLayoutKey {
    measurement: MeasureKey,
    height: u32,
}

impl TextLayoutKey {
    /// Creates a logical cache identity from `content`, `style`, and viewport `size`.
    pub(crate) fn new(
        content: &crate::TextContent,
        style: &TextStyle,
        size: argui_core::Size,
    ) -> Self {
        Self {
            measurement: MeasureKey::new(content, style, Some(size.width)),
            height: size.height.to_bits(),
        }
    }
}

impl MeasureKey {
    /// Creates a paint-independent measurement key for `content`, `style`, and `width`.
    pub(crate) fn new(content: &crate::TextContent, style: &TextStyle, width: Option<f32>) -> Self {
        Self {
            text: content.as_str().to_owned(),
            runs: content
                .runs()
                .iter()
                .map(|(range, style)| RunKey {
                    range: [range.start, range.end],
                    style: SpanStyleKey::new(style),
                })
                .collect(),
            style: StyleKey::new(style),
            width: if style.wrap == TextWrap::None
                && style.overflow == TextOverflow::Clip
                && matches!(style.align, TextAlign::Start | TextAlign::Left)
            {
                None
            } else {
                width.map(f32::to_bits)
            },
        }
    }
}

/// Returns whether `left` and `right` contain the same text and layout span overrides.
pub(crate) fn same_content_layout(left: &crate::TextContent, right: &crate::TextContent) -> bool {
    left.as_str() == right.as_str()
        && left.runs().len() == right.runs().len()
        && left
            .runs()
            .iter()
            .zip(right.runs())
            .all(|((left_range, left), (right_range, right))| {
                left_range == right_range && SpanStyleKey::new(left) == SpanStyleKey::new(right)
            })
}

/// Returns whether `left` and `right` have equal logical layout properties.
pub(crate) fn same_style_layout(left: &TextStyle, right: &TextStyle) -> bool {
    StyleKey::new(left) == StyleKey::new(right)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StyleKey {
    font_size: u32,
    line_height: u32,
    family: FontFamily,
    weight: u16,
    font_style: FontStyle,
    stretch: FontStretch,
    letter_spacing: LetterSpacingKey,
    decoration: DecorationKey,
    align: TextAlign,
    wrap: TextWrap,
    overflow: TextOverflow,
    line_clamp: Option<usize>,
}

impl StyleKey {
    /// Retains the layout-affecting properties of `style`.
    fn new(style: &TextStyle) -> Self {
        Self {
            font_size: style.font_size.to_bits(),
            line_height: style.line_height.to_bits(),
            family: style.family.clone(),
            weight: style.weight,
            font_style: style.font_style,
            stretch: style.stretch,
            letter_spacing: LetterSpacingKey::new(style.letter_spacing),
            decoration: DecorationKey::new(style.decoration),
            align: style.align,
            wrap: style.wrap,
            overflow: style.overflow,
            line_clamp: style.line_clamp.map(std::num::NonZeroUsize::get),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RunKey {
    range: [usize; 2],
    style: SpanStyleKey,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SpanStyleKey {
    font_size: Option<u32>,
    line_height: Option<u32>,
    family: Option<FontFamily>,
    weight: Option<u16>,
    font_style: Option<FontStyle>,
    stretch: Option<FontStretch>,
    letter_spacing: Option<LetterSpacingKey>,
    decoration: Option<DecorationKey>,
}

impl SpanStyleKey {
    /// Retains the layout-affecting overrides of `style`.
    fn new(style: &TextSpanStyle) -> Self {
        Self {
            font_size: style.font_size.map(f32::to_bits),
            line_height: style.line_height.map(f32::to_bits),
            family: style.family.clone(),
            weight: style.weight,
            font_style: style.font_style,
            stretch: style.stretch,
            letter_spacing: style.letter_spacing.map(LetterSpacingKey::new),
            decoration: style.decoration.map(DecorationKey::new),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum LetterSpacingKey {
    Normal,
    Px(u32),
    Em(u32),
}

impl LetterSpacingKey {
    fn new(value: LetterSpacing) -> Self {
        match value {
            LetterSpacing::Normal => Self::Normal,
            LetterSpacing::Px(value) => Self::Px(value.to_bits()),
            LetterSpacing::Em(value) => Self::Em(value.to_bits()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct DecorationKey {
    underline: UnderlineStyle,
    strikethrough: bool,
}

impl DecorationKey {
    /// Retains decoration geometry from `value`, excluding its paint colors.
    fn new(value: TextDecoration) -> Self {
        Self {
            underline: value.underline,
            strikethrough: value.strikethrough,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CachedGlyph {
    pub(crate) key: GlyphKey,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) rtl: bool,
    pub(crate) local: crate::layout::LogicalGlyph,
    pub(crate) span: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct CachedDecoration {
    pub(crate) local: [f32; 4],
    pub(crate) span: usize,
    pub(crate) strikethrough: bool,
}

#[derive(Clone, Default)]
pub(crate) struct CachedShape {
    pub(crate) glyphs: Arc<[CachedGlyph]>,
    pub(crate) decorations: Arc<[CachedDecoration]>,
}

#[derive(Default)]
pub(crate) struct TextCache {
    measurements: HashMap<MeasureKey, TextMeasurement>,
    measurement_order: VecDeque<MeasureKey>,
    shapes: HashMap<TextLayoutKey, CachedShape>,
    shape_order: VecDeque<TextLayoutKey>,
    layouts: HashMap<TextLayoutKey, Arc<TextLayout>>,
    layout_order: VecDeque<TextLayoutKey>,
}

impl TextCache {
    pub(crate) fn measurement(&self, key: &MeasureKey) -> Option<TextMeasurement> {
        self.measurements.get(key).copied()
    }

    pub(crate) fn insert_measurement(&mut self, key: MeasureKey, measurement: TextMeasurement) {
        insert_bounded(
            &mut self.measurements,
            &mut self.measurement_order,
            key,
            measurement,
        );
    }

    /// Returns cached logical glyphs for `key`, or `None` on a miss.
    pub(crate) fn shape(&self, key: &TextLayoutKey) -> Option<CachedShape> {
        self.shapes.get(key).cloned()
    }

    /// Caches logical `shape` under `key` and returns its shared storage.
    pub(crate) fn insert_shape(&mut self, key: TextLayoutKey, shape: CachedShape) -> CachedShape {
        insert_bounded(&mut self.shapes, &mut self.shape_order, key, shape.clone());
        shape
    }

    pub(crate) fn layout(&self, key: &TextLayoutKey) -> Option<Arc<TextLayout>> {
        self.layouts.get(key).cloned()
    }

    pub(crate) fn insert_layout(
        &mut self,
        key: TextLayoutKey,
        layout: Arc<TextLayout>,
    ) -> Arc<TextLayout> {
        insert_bounded(
            &mut self.layouts,
            &mut self.layout_order,
            key,
            Arc::clone(&layout),
        );
        layout
    }

    pub(crate) fn clear(&mut self) {
        self.measurements.clear();
        self.measurement_order.clear();
        self.shapes.clear();
        self.shape_order.clear();
        self.layouts.clear();
        self.layout_order.clear();
    }
}

fn insert_bounded<K, V>(entries: &mut HashMap<K, V>, order: &mut VecDeque<K>, key: K, value: V)
where
    K: Clone + Eq + std::hash::Hash,
{
    if let std::collections::hash_map::Entry::Occupied(mut entry) = entries.entry(key.clone()) {
        entry.insert(value);
        return;
    }
    if entries.len() == CACHE_CAPACITY
        && let Some(oldest) = order.pop_front()
    {
        entries.remove(&oldest);
    }
    order.push_back(key.clone());
    entries.insert(key, value);
}
