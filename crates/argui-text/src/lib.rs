//! Text shaping and font-system boundary.

mod engine;
mod layout;
mod style;

pub use engine::TextEngine;
pub use layout::{GlyphContent, GlyphImage, GlyphKey, PreparedGlyph, PreparedText};
pub use style::{FontFamily, TextBlock, TextColor, TextScene, TextStyle, TextWrap};
