use std::{num::NonZeroUsize, ops::Range};

use argui_core::{Rect, Size};

pub use argui_core::Color as TextColor;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
    Named(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LetterSpacing {
    #[default]
    Normal,
    Px(f32),
    Em(f32),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum UnderlineStyle {
    #[default]
    None,
    Single,
    Double,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextDecoration {
    pub underline: UnderlineStyle,
    pub underline_color: Option<TextColor>,
    pub strikethrough: bool,
    pub strikethrough_color: Option<TextColor>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWrap {
    None,
    Glyph,
    Word,
    #[default]
    WordOrGlyph,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EllipsisPosition {
    Start,
    Middle,
    #[default]
    End,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis(EllipsisPosition),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMeasurement {
    pub size: Size,
    pub first_baseline: Option<f32>,
    pub last_baseline: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub color: TextColor,
    pub family: FontFamily,
    pub weight: u16,
    pub font_style: FontStyle,
    pub stretch: FontStretch,
    pub letter_spacing: LetterSpacing,
    pub decoration: TextDecoration,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
    pub line_clamp: Option<NonZeroUsize>,
    pub align: TextAlign,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 20.0,
            color: TextColor::WHITE,
            family: FontFamily::SansSerif,
            weight: 400,
            font_style: FontStyle::Normal,
            stretch: FontStretch::Normal,
            letter_spacing: LetterSpacing::Normal,
            decoration: TextDecoration::default(),
            wrap: TextWrap::WordOrGlyph,
            overflow: TextOverflow::Clip,
            line_clamp: None,
            align: TextAlign::Start,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextSpanStyle {
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub color: Option<TextColor>,
    pub family: Option<FontFamily>,
    pub weight: Option<u16>,
    pub font_style: Option<FontStyle>,
    pub stretch: Option<FontStretch>,
    pub letter_spacing: Option<LetterSpacing>,
    pub decoration: Option<TextDecoration>,
}

impl TextSpanStyle {
    #[must_use]
    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }

    #[must_use]
    pub const fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    #[must_use]
    pub const fn color(mut self, color: TextColor) -> Self {
        self.color = Some(color);
        self
    }

    #[must_use]
    pub const fn weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    #[must_use]
    pub fn family(mut self, family: FontFamily) -> Self {
        self.family = Some(family);
        self
    }

    #[must_use]
    pub const fn font_style(mut self, style: FontStyle) -> Self {
        self.font_style = Some(style);
        self
    }

    #[must_use]
    pub const fn stretch(mut self, stretch: FontStretch) -> Self {
        self.stretch = Some(stretch);
        self
    }

    #[must_use]
    pub const fn letter_spacing(mut self, spacing: LetterSpacing) -> Self {
        self.letter_spacing = Some(spacing);
        self
    }

    #[must_use]
    pub const fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.decoration = Some(decoration);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub content: String,
    pub style: TextSpanStyle,
}

impl TextSpan {
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextSpanStyle::default(),
        }
    }

    #[must_use]
    pub fn style(mut self, style: TextSpanStyle) -> Self {
        self.style = style;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextContent {
    text: String,
    runs: Vec<(Range<usize>, TextSpanStyle)>,
}

impl TextContent {
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            runs: Vec::new(),
        }
    }

    #[must_use]
    pub fn rich(spans: impl IntoIterator<Item = TextSpan>) -> Self {
        let mut text = String::new();
        let mut runs = Vec::new();
        for span in spans {
            let start = text.len();
            text.push_str(&span.content);
            let end = text.len();
            if start != end {
                runs.push((start..end, span.style));
            }
        }
        Self { text, runs }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn is_rich(&self) -> bool {
        !self.runs.is_empty()
    }

    pub(crate) fn runs(&self) -> &[(Range<usize>, TextSpanStyle)] {
        &self.runs
    }
}

impl From<&str> for TextContent {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl From<String> for TextContent {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    pub content: TextContent,
    pub bounds: Rect,
    pub clip: Rect,
    pub style: TextStyle,
}

impl TextBlock {
    #[must_use]
    pub fn new(content: impl Into<TextContent>, bounds: Rect) -> Self {
        Self {
            content: content.into(),
            bounds,
            clip: bounds,
            style: TextStyle::default(),
        }
    }

    #[must_use]
    pub fn size(mut self, font_size: f32) -> Self {
        self.style.font_size = font_size;
        self.style.line_height = font_size * 1.25;
        self
    }

    #[must_use]
    pub const fn line_height(mut self, line_height: f32) -> Self {
        self.style.line_height = line_height;
        self
    }

    #[must_use]
    pub const fn color(mut self, color: TextColor) -> Self {
        self.style.color = color;
        self
    }

    #[must_use]
    pub fn family(mut self, family: FontFamily) -> Self {
        self.style.family = family;
        self
    }

    #[must_use]
    pub const fn weight(mut self, weight: u16) -> Self {
        self.style.weight = weight;
        self
    }

    #[must_use]
    pub const fn font_style(mut self, style: FontStyle) -> Self {
        self.style.font_style = style;
        self
    }

    #[must_use]
    pub const fn stretch(mut self, stretch: FontStretch) -> Self {
        self.style.stretch = stretch;
        self
    }

    #[must_use]
    pub const fn letter_spacing(mut self, spacing: LetterSpacing) -> Self {
        self.style.letter_spacing = spacing;
        self
    }

    #[must_use]
    pub const fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.style.decoration = decoration;
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: TextWrap) -> Self {
        self.style.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.style.overflow = overflow;
        self
    }

    #[must_use]
    pub const fn line_clamp(mut self, line_clamp: Option<NonZeroUsize>) -> Self {
        self.style.line_clamp = line_clamp;
        self
    }

    #[must_use]
    pub const fn align(mut self, align: TextAlign) -> Self {
        self.style.align = align;
        self
    }

    #[must_use]
    pub const fn clip(mut self, clip: Rect) -> Self {
        self.clip = clip;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextScene {
    blocks: Vec<TextBlock>,
}

impl TextScene {
    #[must_use]
    pub const fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    #[must_use]
    pub fn with(mut self, block: TextBlock) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn push(&mut self, block: TextBlock) {
        self.blocks.push(block);
    }

    #[must_use]
    pub fn blocks(&self) -> &[TextBlock] {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut [TextBlock] {
        &mut self.blocks
    }
}
