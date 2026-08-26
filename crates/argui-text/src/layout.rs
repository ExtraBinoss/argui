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
    pub(crate) local: [i32; 2],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreparedText {
    pub blocks: usize,
    pub glyphs: Vec<PreparedGlyph>,
    pub(crate) scale_factor: f32,
}

impl PreparedText {
    pub fn reposition_block(
        &mut self,
        block: usize,
        origin: argui_core::Point,
        clip: argui_core::Rect,
    ) {
        let anchor = [
            (origin.x * self.scale_factor).round() as i32,
            (origin.y * self.scale_factor).round() as i32,
        ];
        let clip = [
            clip.origin.x * self.scale_factor,
            clip.origin.y * self.scale_factor,
            (clip.origin.x + clip.size.width) * self.scale_factor,
            (clip.origin.y + clip.size.height) * self.scale_factor,
        ];
        for glyph in self.glyphs.iter_mut().filter(|glyph| glyph.block == block) {
            glyph.x = anchor[0] + glyph.local[0];
            glyph.y = anchor[1] + glyph.local[1];
            glyph.clip = clip;
        }
    }
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
