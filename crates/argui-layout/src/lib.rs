//! Layout boundary backed by Taffy.

mod assets;
mod engine;
mod error;
mod input;
mod overlay;
mod paint;
mod reconcile;
mod scroll;
mod selection;
mod style;
mod text;

pub use engine::{LayoutEngine, LayoutNode, LayoutOutput, PaintStats, PortalLayout};
pub use error::LayoutError;
pub use input::TextInputRegion;
pub use selection::TextRegion;
