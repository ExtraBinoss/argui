use crate::{AssetKey, AssetKind};

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
    Image {
        key: AssetKey,
        #[source]
        source: argui_image::DecodeError,
    },
    #[error("vector `{key}` could not be parsed: {source}")]
    Vector {
        key: AssetKey,
        #[source]
        source: argui_vector::VectorError,
    },
}
