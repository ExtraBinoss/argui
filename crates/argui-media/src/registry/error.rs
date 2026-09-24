use super::{AssetKey, AssetKind};

/// Source asset validation or registry update failure.
#[derive(Debug, thiserror::Error)]
pub enum AssetRegistryError {
    #[error("invalid project-relative asset key `{0}`")]
    InvalidKey(String),
    #[error("asset `{key}` is already registered as {existing:?}, not {requested:?}")]
    KindMismatch {
        key: AssetKey,
        existing: AssetKind,
        requested: AssetKind,
    },
    #[error("asset revision space exhausted for `{0}`")]
    RevisionExhausted(AssetKey),
    #[error("image `{key}` could not be decoded: {source}")]
    #[cfg(feature = "media")]
    Image {
        key: AssetKey,
        #[source]
        source: crate::image::DecodeError,
    },
    #[error("vector `{key}` could not be parsed: {source}")]
    #[cfg(feature = "media")]
    Vector {
        key: AssetKey,
        #[source]
        source: crate::svg::VectorError,
    },
}
