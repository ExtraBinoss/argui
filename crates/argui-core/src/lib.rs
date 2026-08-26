//! Dependency-light types shared by Argui crates.

mod color;
mod geometry;
mod input;

pub use color::Color;
pub use geometry::{Point, Rect, Size};
pub use input::ScrollDelta;
