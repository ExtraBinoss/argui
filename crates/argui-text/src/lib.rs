//! Text shaping and font-system boundary.

mod cache;
mod engine;
mod input;
mod layout;
mod style;

pub use engine::TextEngine;
pub use input::{CaretScroll, CaretStop, TextInputLayout, TextInputScroll};
pub use layout::{GlyphContent, GlyphImage, GlyphKey, PreparedGlyph, PreparedText};
pub use style::{FontFamily, TextBlock, TextColor, TextScene, TextStyle, TextWrap};
