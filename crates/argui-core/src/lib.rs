//! Dependency-light types shared by Argui crates.

mod backdrop;
mod color;
mod environment;
mod geometry;
mod input;
mod insets;
mod keyboard;
mod text;

pub use backdrop::BackdropMaterial;
pub use color::{Color, ColorInterpolation, ParseColorError};
pub use environment::ColorScheme;
pub use geometry::{Affine2D, Point, Rect, Size, Transform2D, TransformOrigin};
pub use input::{
    PinchRecognizer, PinchUpdate, PointerButton, PointerEvent, PointerId, PointerKind,
    PointerPhase, PointerSettings, ScrollDelta,
};
pub use insets::Insets;
pub use keyboard::{ImeInput, Key, KeyInput, KeyState, Modifiers};
pub use text::{CaretAffinity, TextPosition};
