//! Stable media identities, authored paths, and optional image/SVG decoding.

#[cfg(feature = "media")]
pub mod image;
pub mod path;
pub mod registry;
#[cfg(feature = "media")]
pub mod svg;
pub mod vector;

#[cfg(feature = "media")]
pub use image::{DecodeError, ImageLibrary, decode};
pub use path::{PathCommand, PathError, PathStyle, path_asset};
pub use registry::{
    AssetChange, AssetHandle, AssetKey, AssetKind, AssetRecord, AssetRegistry, AssetRegistryError,
    AssetRevision,
};
#[cfg(feature = "media")]
pub use svg::{VectorError, parse_svg};
pub use vector::VectorLibrary;
