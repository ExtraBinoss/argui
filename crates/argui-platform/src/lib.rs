//! Native and web window/event integration.

mod error;
mod event;
mod window;

pub use error::PlatformError;
pub use event::{ButtonState, PlatformEvent, PointerButton};
pub use window::WindowConfig;
