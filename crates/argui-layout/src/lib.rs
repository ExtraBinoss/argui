//! Layout boundary backed by Taffy.

mod anchor;
mod assets;
mod custom;
mod desktop_backdrop;
pub use desktop_backdrop::DesktopBackdropRegion;
mod engine;
mod error;
mod input;
mod layout_tree;
mod overlay;
mod paint;
mod reconcile;
mod scroll;
mod selection;
mod style;
mod surface;
mod text;
mod virtual_list;

pub use custom::CustomElementStats;
pub use engine::{LayoutEngine, LayoutNode, LayoutOutput, LayoutStorage, PaintStats, PortalLayout};
pub use error::LayoutError;
pub use input::TextInputRegion;
pub use selection::TextRegion;
pub use surface::NativeSurfacePaint;
