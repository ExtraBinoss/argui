#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Native and web window/event integration.

mod clipboard;
mod error;
mod event;
mod window;

pub use clipboard::{Clipboard, ClipboardError};
pub use error::PlatformError;
pub use event::{
    ButtonState, ImeInput, Key, KeyInput, KeyState, Modifiers, PlatformEvent, PointerButton,
    ScrollDelta,
};
pub use window::WindowConfig;
