//! Loads an application's generated assets for its windowless automation scene.

use argui_gallery_assets::{AssetInput, AssetKind, decode_assets};
use argui_runtime::NativeHostAssets;
use serde_json::Value;
use std::{
    fs,
    path::{Component, Path},
};

/// Decodes the app-local `manifest`, rejecting invalid IDs and paths escaping its assets directory.
pub(super) fn load(manifest: &Path) -> Result<NativeHostAssets, String> {
    let document: Value =
        serde_json::from_slice(&fs::read(manifest).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let sources = document
        .get("assets")
        .and_then(Value::as_array)
        .ok_or("missing assets array")?;
    let root = manifest
        .parent()
        .ok_or("missing manifest directory")?
        .join("assets");
    let root = fs::canonicalize(root).map_err(|error| error.to_string())?;
    let mut assets = NativeHostAssets::default();
    for source in sources {
        let key = source
            .get("key")
            .and_then(Value::as_str)
            .ok_or("missing asset key")?;
        let path = source
            .get("path")
            .and_then(Value::as_str)
            .ok_or("missing asset path")?;
        let id = source
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id > 0 && *id < 1_u64 << 53)
            .ok_or("invalid asset ID")?;
        let kind = match source.get("kind").and_then(Value::as_str) {
            Some("image") => AssetKind::Image,
            Some("svg") => AssetKind::Svg,
            _ => return Err(format!("{key}: unsupported asset kind")),
        };
        let relative = Path::new(path);
        if !relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(format!("{key}: unsafe asset path"));
        }
        let file =
            fs::canonicalize(root.join(relative)).map_err(|error| format!("{key}: {error}"))?;
        if !file.starts_with(&root) {
            return Err(format!("{key}: asset escapes app root"));
        }
        let bytes = fs::read(file).map_err(|error| format!("{key}: {error}"))?;
        let decoded = decode_assets(&[AssetInput {
            key,
            kind,
            id,
            bytes: &bytes,
        }])?;
        assets.images.extend(decoded.images);
        assets.vectors.extend(decoded.vectors);
    }
    Ok(assets)
}

#[cfg(test)]
#[path = "../../tests/automation/assets.rs"]
mod tests;
