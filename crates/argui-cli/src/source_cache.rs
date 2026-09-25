//! Checksummed cache of component files from one exact release tag.

use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const MAX_BYTES: u64 = 256 * 1024;

/// Returns one UTF-8 file from the app's exact Argui release tag.
///
/// `version` is the app's Argui version; `path` is a validated repository-relative
/// widget path; `expected` is the SHA-256 published in that release registry.
/// A valid cached body is reused without network access.
///
/// # Errors
/// Returns an error for a missing release, oversized body, or checksum mismatch.
pub(crate) fn file(version: &str, path: &str, expected: &str) -> Result<String, String> {
    if !safe_version(version) || !safe_path(path) || !valid_hash(expected) {
        return Err("invalid release version, source path, or checksum".into());
    }
    let cache = cache_root().join(version).join("widgets").join(path);
    if let Ok(body) = read_verified(&cache, expected) {
        return Ok(body);
    }
    let url = format!(
        "https://raw.githubusercontent.com/ExtraBinoss/argui/v{version}/packages/widgets/src/{path}"
    );
    let body = download(&url, &cache)?;
    if hash(&body) != expected {
        let _ = fs::remove_file(&cache);
        return Err(format!(
            "source checksum mismatch for `{path}` at v{version}; the release tag does not match this registry"
        ));
    }
    Ok(body)
}

/// Fetches a registry from one exact release tag when the CLI version differs.
///
/// # Errors
/// Returns an error for an invalid version, network failure, or oversized body.
pub(crate) fn registry(version: &str) -> Result<String, String> {
    if !safe_version(version) {
        return Err("invalid Argui release version".into());
    }
    let cache = cache_root().join(version).join("registry.json");
    if let Ok(body) = bounded(&cache) {
        return Ok(body);
    }
    download(
        &format!(
            "https://raw.githubusercontent.com/ExtraBinoss/argui/v{version}/components/registry.json"
        ),
        &cache,
    )
}

/// Downloads a bounded HTTPS body to a temporary file, then caches it.
///
/// # Errors
/// Returns an error when curl fails or the body exceeds the limit.
fn download(url: &str, cache: &Path) -> Result<String, String> {
    let parent = cache.parent().ok_or("cache path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let stage = cache.with_extension(format!("download-{}", std::process::id()));
    let result = (|| -> Result<String, String> {
        let status = Command::new("curl")
            .args([
                "--proto",
                "=https",
                "--tlsv1.2",
                "--fail",
                "--location",
                "--silent",
                "--show-error",
                "--max-time",
                "30",
                "--max-filesize",
                "262144",
                "--output",
            ])
            .arg(&stage)
            .arg(url)
            .status()
            .map_err(|error| {
                format!("curl is required to download Argui release sources: {error}")
            })?;
        if !status.success() {
            return Err(format!(
                "Argui release source unavailable at {url}: {status}"
            ));
        }
        let body = bounded(&stage)?;
        fs::rename(&stage, cache).map_err(|error| format!("{}: {error}", cache.display()))?;
        Ok(body)
    })();
    let _ = fs::remove_file(stage);
    result
}

/// Reads one cached source within the fixed size limit.
///
/// # Errors
/// Returns an error for an unreadable, oversized, or non-UTF-8 file.
fn bounded(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_BYTES {
        return Err(format!("{} is oversized", path.display()));
    }
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// Reads one cached source only if its SHA-256 still matches `expected`.
///
/// # Errors
/// Returns an error for unreadable or modified cached content.
fn read_verified(path: &Path, expected: &str) -> Result<String, String> {
    let body = bounded(path)?;
    if hash(&body) != expected {
        return Err("cached source checksum mismatch".into());
    }
    Ok(body)
}

/// Returns the standard cache location, allowing a caller-owned override.
pub(crate) fn cache_root() -> PathBuf {
    std::env::var_os("ARGUI_CACHE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME").map(|path| PathBuf::from(path).join("argui"))
        })
        .unwrap_or_else(|| std::env::temp_dir().join("argui-cache"))
}

/// Returns the hex SHA-256 of a source body.
fn hash(body: &str) -> String {
    format!("{:x}", Sha256::digest(body.as_bytes()))
}

/// Validates one semantic version before interpolating it into a URL or path.
fn safe_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 40
        && version.bytes().all(|byte| {
            byte.is_ascii_digit() || byte == b'.' || byte == b'-' || byte.is_ascii_lowercase()
        })
}

/// Validates a registry-controlled widget path before interpolating it into a URL.
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 180
        && !path.contains("..")
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'/' | b'-' | b'_' | b'.')
        })
        && (path.ends_with(".ts") || path.ends_with(".tsx"))
}

/// Validates one lowercase SHA-256 string.
fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
