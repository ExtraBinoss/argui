//! Validated, retained SVG assets rasterized into Argui's bounded WGPU atlas.

mod library;
mod svg;

pub use library::VectorLibrary;
pub use svg::{VectorError, parse_svg};
