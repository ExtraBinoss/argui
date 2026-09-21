//! Stable source identities and transactional revisions for live image and SVG assets.

mod error;
mod key;
mod registry;

pub use error::AssetRegistryError;
pub use key::{AssetKey, AssetRevision};
pub use registry::{AssetChange, AssetHandle, AssetKind, AssetRecord, AssetRegistry};
