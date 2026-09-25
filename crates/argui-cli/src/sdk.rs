//! Embedded, versioned SDK source distributed with the CLI binary.

use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sdk.bundle"));

/// One app-relative source file from the CLI's SDK snapshot.
pub struct Asset<'a> {
    /// Relative path below the generated application.
    pub path: &'a str,
    /// Complete source bytes, including binary fonts and icons.
    pub body: &'a [u8],
}

/// Returns the SHA-256 of the exact bundled SDK snapshot.
pub fn digest() -> String {
    format!("{:x}", Sha256::digest(BUNDLE))
}

/// Returns whether the embedded widget sources are certified against a release tag.
pub fn release_available() -> bool {
    assets()
        .ok()
        .and_then(|assets| {
            assets
                .into_iter()
                .find(|asset| asset.path == "distribution.json")
        })
        .and_then(|asset| serde_json::from_slice::<serde_json::Value>(asset.body).ok())
        .is_some_and(|value| {
            value["version"] == env!("CARGO_PKG_VERSION") && value["sourceStatus"] == "released"
        })
}

/// Installs and verifies this CLI's SDK snapshot in a cache outside the app.
///
/// # Errors
/// Returns an error if a cached file is a symlink or cannot be refreshed.
pub fn materialize() -> Result<PathBuf, String> {
    let hash = digest();
    let root = crate::source_cache::cache_root().join(format!(
        "sdk-v{}-{}",
        env!("CARGO_PKG_VERSION"),
        &hash[..16]
    ));
    if root.is_symlink() {
        return Err(format!("SDK cache is a symlink: {}", root.display()));
    }
    for asset in assets()? {
        let path = root.join(asset.path);
        if path.is_symlink() {
            return Err(format!("SDK cache asset is a symlink: {}", path.display()));
        }
        if fs::read(&path).is_ok_and(|body| body == asset.body) {
            continue;
        }
        let parent = path.parent().ok_or("SDK cache asset lacks parent")?;
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        let stage = path.with_extension(format!("argui-stage-{}", std::process::id()));
        fs::write(&stage, asset.body).map_err(|error| format!("{}: {error}", stage.display()))?;
        fs::rename(&stage, &path).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(root)
}

/// Decodes SDK assets and returns them in deterministic path order.
///
/// # Errors
/// Returns an error when the CLI-owned bundle is truncated or malformed.
pub fn assets() -> Result<Vec<Asset<'static>>, String> {
    let mut cursor = 0;
    let mut assets = Vec::new();
    while cursor < BUNDLE.len() {
        let name_size = read::<4>(&mut cursor)?;
        let name_size = u32::from_le_bytes(name_size) as usize;
        let name = take(&mut cursor, name_size)?;
        let name = std::str::from_utf8(name).map_err(|error| error.to_string())?;
        let body_size = read::<8>(&mut cursor)?;
        let body_size =
            usize::try_from(u64::from_le_bytes(body_size)).map_err(|error| error.to_string())?;
        let body = take(&mut cursor, body_size)?;
        assets.push(Asset { path: name, body });
    }
    Ok(assets)
}

/// Reads a fixed byte array from the bundle, advancing `cursor`.
///
/// # Errors
/// Returns an error if fewer than `N` bytes remain.
fn read<const N: usize>(cursor: &mut usize) -> Result<[u8; N], String> {
    take(cursor, N)?
        .try_into()
        .map_err(|_| "truncated SDK bundle".into())
}

/// Returns `length` bytes from the bundle and advances `cursor`.
///
/// # Errors
/// Returns an error when the bundle ends before the requested bytes.
fn take(cursor: &mut usize, length: usize) -> Result<&'static [u8], String> {
    let end = cursor
        .checked_add(length)
        .ok_or("SDK bundle length overflow")?;
    let body = BUNDLE.get(*cursor..end).ok_or("truncated SDK bundle")?;
    *cursor = end;
    Ok(body)
}
