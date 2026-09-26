//! Verified, in-place updates of the released Argui CLI binary.

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use reqwest::blocking::Client;
use semver::Version;
use sha2::{Digest, Sha256};

const REPOSITORY: &str = "https://github.com/ExtraBinoss/argui";
const LATEST: &str = "https://api.github.com/repos/ExtraBinoss/argui/releases/latest";
const MAX_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_CHECKSUM_BYTES: u64 = 1024;

/// Checks the latest release and installs a newer verified CLI archive.
///
/// `options` accepts only `--check`, which reports availability without
/// downloading the archive. Returns an error when lookup, verification,
/// extraction, or replacement fails; the installed binary remains unchanged
/// until a fully verified replacement has been staged.
pub(crate) fn run(options: &[String]) -> Result<(), String> {
    let check_only = match options {
        [] => false,
        [option] if option == "--check" => true,
        _ => return Err("usage: argui update [--check]".into()),
    };
    let target = release_target()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(45))
        .user_agent(format!("argui-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("cannot configure update client: {error}"))?;
    let latest = latest_release(&client)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("invalid installed CLI version: {error}"))?;
    if latest <= current {
        println!("Argui CLI v{current} is up to date.");
        return Ok(());
    }
    if check_only {
        println!("Argui CLI v{latest} is available (installed: v{current}).");
        return Ok(());
    }
    let executable = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|error| format!("cannot locate the running Argui CLI: {error}"))?;
    let destination = executable
        .parent()
        .ok_or("the running Argui CLI has no parent directory")?;
    let tag = format!("v{latest}");
    let asset = asset_name(&tag, target);
    let url = format!("{REPOSITORY}/releases/download/{tag}/{asset}");
    let mut archive = tempfile::NamedTempFile::new_in(destination)
        .map_err(|error| format!("cannot stage update beside the CLI: {error}"))?;
    let digest = download_archive(&client, &url, archive.as_file_mut())?;
    let checksum = download_small(&client, &format!("{url}.sha256"), MAX_CHECKSUM_BYTES)?;
    verify_checksum(&checksum, &asset, &digest)?;
    let mut replacement = tempfile::NamedTempFile::new_in(destination)
        .map_err(|error| format!("cannot stage replacement beside the CLI: {error}"))?;
    extract_binary(archive.path(), replacement.as_file_mut(), target)?;
    replace_binary(replacement, &executable)?;
    println!(
        "Updated Argui CLI from v{current} to v{latest} at {}.",
        executable.display()
    );
    Ok(())
}

/// Returns the supported release triple for the compilation target.
/// Unsupported targets cannot safely consume another platform's binary.
fn release_target() -> Result<&'static str, String> {
    match () {
        #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
        () => Ok("x86_64-unknown-linux-gnu"),
        #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
        () => Ok("aarch64-unknown-linux-gnu"),
        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        () => Ok("x86_64-apple-darwin"),
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        () => Ok("aarch64-apple-darwin"),
        #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
        () => Ok("x86_64-pc-windows-msvc"),
        #[allow(unreachable_patterns)]
        _ => Err("this platform has no published Argui CLI archive".into()),
    }
}

/// Looks up the latest stable GitHub release and returns its semantic version.
/// `client` applies an HTTP timeout and a GitHub-compatible user agent.
fn latest_release(client: &Client) -> Result<Version, String> {
    let bytes = download_small(client, LATEST, 64 * 1024)?;
    let body: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid GitHub release response: {error}"))?;
    let tag = body["tag_name"]
        .as_str()
        .ok_or("latest GitHub release has no tag_name")?;
    parse_tag(tag)
}

/// Parses one release tag, requiring the `v` prefix used by the installers.
/// `tag` must be a canonical semantic version with no path delimiters.
pub(crate) fn parse_tag(tag: &str) -> Result<Version, String> {
    let version = tag
        .strip_prefix('v')
        .ok_or("release tag must start with v")?;
    let parsed = Version::parse(version)
        .map_err(|error| format!("invalid release version `{tag}`: {error}"))?;
    if parsed.to_string() != version || version.contains('/') || version.contains('\\') {
        return Err(format!("invalid release tag `{tag}`"));
    }
    Ok(parsed)
}

/// Builds the release archive filename for `tag` and `target`.
/// Returns the `.zip` name on Windows or the `.tar.gz` name elsewhere.
pub(crate) fn asset_name(tag: &str, target: &str) -> String {
    let suffix = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("argui-cli-{tag}-{target}.{suffix}")
}

/// Downloads a bounded response in memory for metadata and checksums.
/// `client` sends the request, `url` is a fixed release endpoint, and `limit`
/// bounds allocation. Returns response bytes or an error for HTTP failure or
/// oversized bodies.
fn download_small(client: &Client, url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("cannot download {url}: {error}"))?;
    if response.content_length().is_some_and(|size| size > limit) {
        return Err(format!("download exceeds {limit} bytes: {url}"));
    }
    let mut body = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut body)
        .map_err(|error| format!("cannot read {url}: {error}"))?;
    if body.len() as u64 > limit {
        return Err(format!("download exceeds {limit} bytes: {url}"));
    }
    Ok(body)
}

/// Streams an archive to `output` while computing its SHA-256 digest.
/// `client` sends the request and `url` identifies a release asset. Returns
/// the hexadecimal digest or an error for HTTP failure, oversized content,
/// or an incomplete write.
fn download_archive(client: &Client, url: &str, output: &mut File) -> Result<String, String> {
    let mut response = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("cannot download CLI archive: {error}"))?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_ARCHIVE_BYTES)
    {
        return Err("CLI archive exceeds the download limit".into());
    }
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("cannot read CLI archive: {error}"))?;
        if read == 0 {
            break;
        }
        count += read as u64;
        if count > MAX_ARCHIVE_BYTES {
            return Err("CLI archive exceeds the download limit".into());
        }
        digest.update(&buffer[..read]);
        output
            .write_all(&buffer[..read])
            .map_err(|error| format!("cannot stage CLI archive: {error}"))?;
    }
    output
        .sync_all()
        .map_err(|error| format!("cannot sync staged CLI archive: {error}"))?;
    Ok(format!("{:x}", digest.finalize()))
}

/// Checks the published checksum against the archive digest and exact asset.
/// `contents` are the `.sha256` response, `asset` is the expected basename,
/// and `actual` is the lowercase digest of the downloaded archive. Returns an
/// error when the sidecar format, asset name, or digest differs.
pub(crate) fn verify_checksum(contents: &[u8], asset: &str, actual: &str) -> Result<(), String> {
    let text = std::str::from_utf8(contents).map_err(|_| "checksum is not UTF-8")?;
    let mut fields = text.split_whitespace();
    let expected = fields.next().ok_or("checksum file is empty")?;
    let filename = fields.next().ok_or("checksum file lacks an asset name")?;
    if fields.next().is_some()
        || expected.len() != 64
        || !expected.bytes().all(|byte| byte.is_ascii_hexdigit())
        || filename != asset
    {
        return Err("invalid SHA-256 file for the CLI asset".into());
    }
    if !expected.eq_ignore_ascii_case(actual) {
        return Err("CLI archive SHA-256 mismatch; installation stopped".into());
    }
    Ok(())
}

/// Extracts only the expected CLI executable from a verified release archive.
/// `archive` is a staged path, `replacement` is an open staged file, and `target` selects tar.gz or
/// zip. Returns an error for extra files, unsafe names, or an oversized binary;
/// otherwise the replacement file contains the executable.
pub(crate) fn extract_binary(
    archive: &Path,
    replacement: &mut File,
    target: &str,
) -> Result<(), String> {
    #[cfg(windows)]
    if target.contains("windows") {
        return extract_zip(archive, replacement);
    }
    if target.contains("windows") {
        return Err("Windows CLI archives require a Windows updater".into());
    }
    extract_tar(archive, replacement)
}

/// Extracts one regular `argui` entry from a gzip-compressed tar archive.
/// `archive` is the verified input and `replacement` receives its binary.
/// Returns an error if the archive does not contain exactly one safe entry.
fn extract_tar(archive: &Path, replacement: &mut File) -> Result<(), String> {
    let source =
        File::open(archive).map_err(|error| format!("cannot open CLI archive: {error}"))?;
    let decoder = flate2::read::GzDecoder::new(source);
    let mut archive = tar::Archive::new(decoder);
    let mut found = false;
    for entry in archive
        .entries()
        .map_err(|error| format!("invalid CLI tar archive: {error}"))?
    {
        let mut entry = entry.map_err(|error| format!("invalid CLI tar entry: {error}"))?;
        if found
            || entry.path().map_err(|error| error.to_string())?.as_ref() != Path::new("argui")
            || !entry.header().entry_type().is_file()
            || entry.size() > MAX_ARCHIVE_BYTES
        {
            return Err("CLI archive must contain exactly one regular `argui` file".into());
        }
        std::io::copy(&mut entry, replacement)
            .map_err(|error| format!("cannot extract CLI executable: {error}"))?;
        replacement
            .sync_all()
            .map_err(|error| format!("cannot sync CLI executable: {error}"))?;
        found = true;
    }
    if !found {
        return Err("CLI archive contains no `argui` executable".into());
    }
    Ok(())
}

/// Extracts one regular `argui.exe` entry from a verified Windows ZIP.
/// `archive` is the verified input and `replacement` receives its binary.
/// Returns an error if the archive does not contain exactly one safe entry.
#[cfg(windows)]
fn extract_zip(archive: &Path, replacement: &mut File) -> Result<(), String> {
    let source =
        File::open(archive).map_err(|error| format!("cannot open CLI archive: {error}"))?;
    let mut package = zip::ZipArchive::new(source)
        .map_err(|error| format!("invalid CLI ZIP archive: {error}"))?;
    if package.len() != 1 {
        return Err("CLI archive must contain exactly one `argui.exe` file".into());
    }
    let mut entry = package
        .by_index(0)
        .map_err(|error| format!("invalid CLI ZIP entry: {error}"))?;
    if entry.name() != "argui.exe" || entry.is_dir() || entry.size() > MAX_ARCHIVE_BYTES {
        return Err("CLI archive must contain exactly one `argui.exe` file".into());
    }
    std::io::copy(&mut entry, replacement)
        .map_err(|error| format!("cannot extract CLI executable: {error}"))?;
    replacement
        .sync_all()
        .map_err(|error| format!("cannot sync CLI executable: {error}"))?;
    Ok(())
}

/// Installs a staged executable next to the running binary.
/// `replacement` is a fully verified temporary file; `destination` is the
/// canonical current CLI path. Unix uses an atomic rename; Windows restores
/// the old executable if the staged rename fails.
pub(crate) fn replace_binary(
    replacement: tempfile::NamedTempFile,
    destination: &Path,
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(destination)
        .map_err(|error| format!("cannot inspect installed CLI: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("installed CLI must be a regular file".into());
    }
    #[cfg(windows)]
    {
        replace_running_windows(replacement, destination, metadata.permissions())?;
    }
    #[cfg(not(windows))]
    {
        fs::set_permissions(replacement.path(), metadata.permissions())
            .map_err(|error| format!("cannot set CLI permissions: {error}"))?;
        replacement
            .as_file()
            .sync_all()
            .map_err(|error| format!("cannot sync replacement CLI: {error}"))?;
        replacement
            .persist(destination)
            .map_err(|error| format!("cannot replace installed CLI: {}", error.error))?;
    }
    Ok(())
}

/// Replaces the running Windows executable with rollback around the staged rename.
/// `replacement` is the verified binary, `destination` is the current executable,
/// and `permissions` preserve its file attributes. Returns an error if staging or
/// either rename fails, including the backup path when rollback also fails.
#[cfg(windows)]
fn replace_running_windows(
    replacement: tempfile::NamedTempFile,
    destination: &Path,
    permissions: fs::Permissions,
) -> Result<(), String> {
    let current = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|error| format!("cannot locate running CLI: {error}"))?;
    if current != fs::canonicalize(destination).map_err(|error| error.to_string())? {
        return Err("Windows updates can replace only the running CLI".into());
    }
    fs::set_permissions(replacement.path(), permissions)
        .map_err(|error| format!("cannot set CLI permissions: {error}"))?;
    replacement
        .as_file()
        .sync_all()
        .map_err(|error| format!("cannot sync replacement CLI: {error}"))?;
    let parent = destination
        .parent()
        .ok_or("the running CLI has no parent directory")?;
    let backup = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("cannot reserve CLI backup path: {error}"))?;
    let backup_path = backup.path().to_path_buf();
    backup
        .close()
        .map_err(|error| format!("cannot prepare CLI backup path: {error}"))?;
    fs::rename(destination, &backup_path)
        .map_err(|error| format!("cannot move installed CLI to backup: {error}"))?;
    let staged_path = replacement.into_temp_path();
    if let Err(error) = fs::rename(&staged_path, destination) {
        if let Err(restore) = fs::rename(&backup_path, destination) {
            return Err(format!(
                "cannot install CLI: {error}; restoration failed: {restore}; old CLI is at {}",
                backup_path.display()
            ));
        }
        return Err(format!("cannot install CLI; old version restored: {error}"));
    }
    if let Err(error) = self_replace::self_delete_at(&backup_path) {
        eprintln!(
            "Argui CLI updated, but could not schedule removal of old binary at {}: {error}",
            backup_path.display()
        );
    }
    Ok(())
}
