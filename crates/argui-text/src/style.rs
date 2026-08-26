use argui_core::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextColor([f32; 4]);

impl TextColor {
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self([red, green, blue, 1.0])
    }

    #[must_use]
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        self.0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
    Named(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub color: TextColor,
    pub family: FontFamily,
    pub weight: u16,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 20.0,
            color: TextColor::WHITE,
            family: FontFamily::SansSerif,
            weight: 400,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    pub text: String,
    pub bounds: Rect,
    pub style: TextStyle,
}

impl TextBlock {
    #[must_use]
    pub fn new(text: impl Into<String>, bounds: Rect) -> Self {
        Self {
            text: text.into(),
            bounds,
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
}
