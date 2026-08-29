//! Dependency-light types shared by Argui crates.

mod color;
mod geometry;
mod input;
mod keyboard;
mod text;

pub use color::Color;
pub use geometry::{Affine2D, Point, Rect, Size, Transform2D, TransformOrigin};
pub use input::{PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase, ScrollDelta};
pub use keyboard::{ImeInput, Key, KeyInput, KeyState, Modifiers};
pub use text::{CaretAffinity, TextPosition};
