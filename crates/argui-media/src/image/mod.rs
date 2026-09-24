//! Optional PNG/JPEG decoding for Argui image assets.

use argui_paint::{ImageAsset, ImageAssetError, ImageId};

#[derive(Clone, Debug, Default)]
pub struct ImageLibrary {
    assets: Vec<ImageAsset>,
}

impl ImageLibrary {
    /// Creates an empty image asset library.
    #[must_use]
    pub const fn new() -> Self {
        Self { assets: Vec::new() }
    }

    /// Decodes and stores an image, returning its generated identifier.
    ///
    /// # Arguments
    /// * `encoded` — encoded image bytes supported by the image decoder.
    ///
    /// # Errors
    /// Returns a decoding error for invalid or unsupported bytes, or an asset error if the image
    /// cannot be represented by Argui.
    pub fn insert(&mut self, encoded: &[u8]) -> Result<ImageId, DecodeError> {
        let id = ImageId::fresh();
        self.assets.push(decode(id, encoded)?);
        Ok(id)
    }

    #[must_use]
    /// Returns the decoded assets currently owned by this library.
    pub fn assets(&self) -> &[ImageAsset] {
        &self.assets
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("image decoding failed: {0}")]
    Decode(#[from] image::ImageError),
    #[error(transparent)]
    Asset(#[from] ImageAssetError),
}

/// Decodes image bytes into an RGBA8 asset associated with `id`.
///
/// # Arguments
/// * `id` — identifier to store in the resulting asset.
/// * `encoded` — encoded image bytes.
///
/// # Errors
/// Returns a decoding error for invalid or unsupported bytes, or an asset error if the image
/// cannot be represented by Argui.
pub fn decode(id: ImageId, encoded: &[u8]) -> Result<ImageAsset, DecodeError> {
    let decoded = image::load_from_memory(encoded)?.into_rgba8();
    ImageAsset::rgba8(id, decoded.width(), decoded.height(), decoded.into_raw()).map_err(Into::into)
}
