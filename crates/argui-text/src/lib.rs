//! Text shaping and font-system boundary.

mod cache;
mod engine;
mod input;
mod layout;
mod selection;
mod style;

pub use engine::TextEngine;
pub use input::{CaretScroll, CaretStop, TextInputLayout, TextInputScroll};
pub use layout::{
    GlyphContent, GlyphImage, GlyphKey, PreparedDecoration, PreparedGlyph, PreparedText,
};
pub use selection::{TextLayout, TextLineLayout, line_range, word_range};
pub use style::{
    EllipsisPosition, FontFamily, FontStretch, FontStyle, LetterSpacing, TextAlign, TextBlock,
    TextColor, TextContent, TextDecoration, TextMeasurement, TextOverflow, TextScene, TextSpan,
    TextSpanStyle, TextStyle, TextWrap, UnderlineStyle,
};
