//! Validated, retained SVG assets rasterized into Argui's bounded WGPU atlas.

mod library;
mod path;
mod svg;

pub use library::VectorLibrary;
pub use path::{PathCommand, PathError, PathStyle, path_asset};
pub use svg::{VectorError, parse_svg};
