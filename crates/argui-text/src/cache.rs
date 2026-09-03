use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    FontFamily, FontStretch, FontStyle, GlyphKey, LetterSpacing, TextAlign, TextBlock,
    TextDecoration, TextLayout, TextMeasurement, TextOverflow, TextSpanStyle, TextStyle, TextWrap,
    UnderlineStyle,
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
    width: u32,
    height: u32,
}

impl TextLayoutKey {
    pub(crate) fn new(
        content: &crate::TextContent,
        style: &TextStyle,
        size: argui_core::Size,
    ) -> Self {
        Self {
            measurement: MeasureKey::new(content, style, Some(size.width)),
            width: size.width.to_bits(),
            height: size.height.to_bits(),
        }
    }
}

impl MeasureKey {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShapeKey {
    text: String,
    runs: Vec<RunKey>,
    style: StyleKey,
    size: [u32; 2],
    scale_factor: u32,
    subpixel_origin: [u32; 2],
    align: TextAlign,
    color: [u32; 4],
}

impl ShapeKey {
    pub(crate) fn new(block: &TextBlock, scale_factor: f32) -> Self {
        let pixel = [
            block.bounds.origin.x * scale_factor,
            block.bounds.origin.y * scale_factor,
        ];
        Self {
            text: block.content.as_str().to_owned(),
            runs: block
                .content
                .runs()
                .iter()
                .map(|(range, style)| RunKey {
                    range: [range.start, range.end],
                    style: SpanStyleKey::new(style),
                })
                .collect(),
            style: StyleKey::new(&block.style),
            size: [
                if block.style.wrap == TextWrap::None
                    && block.style.overflow == TextOverflow::Clip
                    && matches!(block.style.align, TextAlign::Start | TextAlign::Left)
                {
                    0
                } else {
                    block.bounds.size.width.to_bits()
                },
                block.bounds.size.height.to_bits(),
            ],
            scale_factor: scale_factor.to_bits(),
            subpixel_origin: [
                (pixel[0] - pixel[0].round()).to_bits(),
                (pixel[1] - pixel[1].round()).to_bits(),
            ],
            align: block.style.align,
            color: block.style.color.as_array().map(f32::to_bits),
        }
    }

    pub(crate) fn subpixel_origin(&self) -> [f32; 2] {
        self.subpixel_origin.map(f32::from_bits)
    }
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
    wrap: TextWrap,
    overflow: TextOverflow,
    line_clamp: Option<usize>,
}

impl StyleKey {
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
    color: Option<[u32; 4]>,
    family: Option<FontFamily>,
    weight: Option<u16>,
    font_style: Option<FontStyle>,
    stretch: Option<FontStretch>,
    letter_spacing: Option<LetterSpacingKey>,
    decoration: Option<DecorationKey>,
}

impl SpanStyleKey {
    fn new(style: &TextSpanStyle) -> Self {
        Self {
            font_size: style.font_size.map(f32::to_bits),
            line_height: style.line_height.map(f32::to_bits),
            color: style.color.map(|value| value.as_array().map(f32::to_bits)),
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
    underline_color: Option<[u32; 4]>,
    strikethrough: bool,
    strikethrough_color: Option<[u32; 4]>,
}

impl DecorationKey {
    fn new(value: TextDecoration) -> Self {
        Self {
            underline: value.underline,
            underline_color: value
                .underline_color
                .map(|color| color.as_array().map(f32::to_bits)),
            strikethrough: value.strikethrough,
            strikethrough_color: value
                .strikethrough_color
                .map(|color| color.as_array().map(f32::to_bits)),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CachedGlyph {
    pub(crate) key: GlyphKey,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) rtl: bool,
    pub(crate) local: [i32; 2],
    pub(crate) color: [f32; 4],
}

#[derive(Clone, Copy)]
pub(crate) struct CachedDecoration {
    pub(crate) local: [i32; 4],
    pub(crate) color: [f32; 4],
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
    shapes: HashMap<ShapeKey, CachedShape>,
    shape_order: VecDeque<ShapeKey>,
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

    pub(crate) fn shape(&self, key: &ShapeKey) -> Option<CachedShape> {
        self.shapes.get(key).cloned()
    }

    pub(crate) fn insert_shape(&mut self, key: ShapeKey, shape: CachedShape) -> CachedShape {
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

#[cfg(test)]
mod tests {
    use super::{CACHE_CAPACITY, MeasureKey, ShapeKey, TextCache};
    use crate::{TextBlock, TextContent, TextMeasurement, TextStyle, TextWrap};
    use argui_core::{Point, Rect, Size};

    #[test]
    fn cache_hits_and_stays_bounded() {
        let mut cache = TextCache::default();
        for index in 0..=CACHE_CAPACITY {
            cache.insert_measurement(
                MeasureKey::new(
                    &TextContent::plain(index.to_string()),
                    &TextStyle::default(),
                    None,
                ),
                TextMeasurement {
                    size: Size::new(index as f32, 1.0),
                    ..TextMeasurement::default()
                },
            );
        }
        assert!(
            cache
                .measurement(&MeasureKey::new(
                    &TextContent::plain("0"),
                    &TextStyle::default(),
                    None,
                ))
                .is_none()
        );
        assert_eq!(
            cache.measurement(&MeasureKey::new(
                &TextContent::plain(CACHE_CAPACITY.to_string()),
                &TextStyle::default(),
                None
            )),
            Some(TextMeasurement {
                size: Size::new(CACHE_CAPACITY as f32, 1.0),
                ..TextMeasurement::default()
            })
        );
        cache.clear();
        assert!(cache.measurements.is_empty());
    }

    #[test]
    fn unwrapped_text_cache_keys_ignore_width_changes() {
        let style = TextStyle {
            wrap: TextWrap::None,
            ..TextStyle::default()
        };
        assert_eq!(
            MeasureKey::new(&TextContent::plain("fixed"), &style, Some(100.0)),
            MeasureKey::new(&TextContent::plain("fixed"), &style, Some(900.0))
        );
        let block = |width| {
            let mut block =
                TextBlock::new("fixed", Rect::new(Point::default(), Size::new(width, 24.0)));
            block.style = style.clone();
            block
        };
        assert_eq!(
            ShapeKey::new(&block(100.0), 1.0),
            ShapeKey::new(&block(900.0), 1.0)
        );
    }
}
