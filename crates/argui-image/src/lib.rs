//! Optional PNG/JPEG decoding for Argui image assets.

use argui_paint::{ImageAsset, ImageAssetError, ImageId};

#[derive(Clone, Debug, Default)]
pub struct ImageLibrary {
    assets: Vec<ImageAsset>,
}

impl ImageLibrary {
    #[must_use]
    pub const fn new() -> Self {
        Self { assets: Vec::new() }
    }

    pub fn insert(&mut self, encoded: &[u8]) -> Result<ImageId, DecodeError> {
        let id = ImageId::fresh();
        self.assets.push(decode(id, encoded)?);
        Ok(id)
    }

    #[must_use]
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

pub fn decode(id: ImageId, encoded: &[u8]) -> Result<ImageAsset, DecodeError> {
    let decoded = image::load_from_memory(encoded)?.into_rgba8();
    ImageAsset::rgba8(id, decoded.width(), decoded.height(), decoded.into_raw()).map_err(Into::into)
}
