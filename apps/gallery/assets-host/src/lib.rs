//! Embedded asset imports for the QuickJS gallery runner.

use argui_paint::{ImageId, VectorId};
use argui_runtime::NativeHostAssets;

/// Decoder selected by the source asset's file format.
#[derive(Clone, Copy)]
pub enum AssetKind {
    /// Encoded PNG, JPEG, or WebP raster.
    Image,
    /// Encoded SVG vector.
    Svg,
}

/// Encoded image or SVG to register with the native renderer.
pub struct AssetInput<'a> {
    /// Logical file name included in decode errors.
    pub key: &'a str,
    /// Decoder for the source bytes.
    pub kind: AssetKind,
    /// Stable ID shared with the JavaScript asset reference.
    pub id: u64,
    /// Encoded source bytes.
    pub bytes: &'a [u8],
}

include!("../../assets.generated.rs");

/// Decodes all embedded image and SVG imports with the IDs exported to Solid and React.
///
/// # Errors
/// Returns an error naming the asset if its encoded bytes cannot be decoded or parsed.
pub fn load() -> Result<NativeHostAssets, String> {
    decode_assets(EMBEDDED)
}

/// Decodes a manifest of raster images and SVGs into native render assets.
///
/// The `sources` entries provide stable IDs and encoded bytes. Returns the
/// decoded assets in their input order, split by raster and vector format.
///
/// # Errors
/// Returns an error naming the first source whose bytes cannot be decoded.
pub fn decode_assets(sources: &[AssetInput<'_>]) -> Result<NativeHostAssets, String> {
    let mut assets = NativeHostAssets::default();
    for asset in sources {
        match asset.kind {
            AssetKind::Image => assets.images.push(
                argui_media::image::decode(ImageId(asset.id), asset.bytes)
                    .map_err(|error| format!("{}: {error}", asset.key))?,
            ),
            AssetKind::Svg => assets.vectors.push(
                argui_media::svg::parse_svg(VectorId(asset.id), asset.bytes)
                    .map_err(|error| format!("{}: {error}", asset.key))?,
            ),
        }
    }
    Ok(assets)
}
