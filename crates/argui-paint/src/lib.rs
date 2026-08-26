//! Renderer-independent painting primitives and ordered display lists.

mod display_list;
mod style;

pub use argui_core::Color;
pub use display_list::{DisplayCommand, DisplayList};
pub use style::{
    Border, BorderWidths, ClipBehavior, CornerRadii, Fill, PaintStyle, Quad, QuadStyle,
};
