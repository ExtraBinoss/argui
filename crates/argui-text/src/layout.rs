#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GlyphKey(pub(crate) cosmic_text::CacheKey);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedGlyph {
    pub key: GlyphKey,
    pub block: usize,
    pub start: usize,
    pub end: usize,
    pub rtl: bool,
    pub x: i32,
    pub y: i32,
    pub color: [f32; 4],
    pub clip: [f32; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreparedText {
    pub blocks: usize,
    pub glyphs: Vec<PreparedGlyph>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphContent {
    Mask,
    Color,
    SubpixelMask,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphImage {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub content: GlyphContent,
    pub data: Vec<u8>,
}
