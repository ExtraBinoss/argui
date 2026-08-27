//! SVG-to-Lyon vector assets rendered later by Argui's WGPU pipeline.

mod library;
mod svg;

pub use library::VectorLibrary;
pub use svg::{VectorError, morph_svg, parse_svg};
