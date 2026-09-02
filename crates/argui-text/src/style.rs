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
pub enum TextWrap {
    None,
    Glyph,
    Word,
    #[default]
    WordOrGlyph,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
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
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
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
            wrap: TextWrap::WordOrGlyph,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    pub text: String,
    pub bounds: Rect,
    pub clip: Rect,
    pub style: TextStyle,
}

impl TextBlock {
    #[must_use]
    pub fn new(text: impl Into<String>, bounds: Rect) -> Self {
        Self {
            text: text.into(),
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
