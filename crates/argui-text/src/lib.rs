//! Text shaping and font-system boundary.

mod engine;
mod input;
mod layout;
mod style;

pub use engine::TextEngine;
pub use input::{CaretStop, TextInputLayout};
pub use layout::{GlyphContent, GlyphImage, GlyphKey, PreparedGlyph, PreparedText};
pub use style::{FontFamily, TextBlock, TextColor, TextScene, TextStyle, TextWrap};
