//! Layout boundary backed by Taffy.

mod anchor;
mod assets;
mod composite;
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
#[cfg(feature = "metrics")]
pub use engine::LayoutProfile;
pub use engine::{LayoutEngine, LayoutNode, LayoutOutput, LayoutStorage, PaintStats, PortalLayout};
pub use error::LayoutError;
pub use input::TextInputRegion;
pub use selection::{SelectionHandleGeometry, TextRegion};
pub use surface::NativeSurfacePaint;
