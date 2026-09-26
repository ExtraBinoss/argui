#![cfg(not(target_arch = "wasm32"))]

#[path = "../src/update.rs"]
mod update;

use sha2::{Digest, Sha256};

#[cfg(unix)]
use flate2::{Compression, write::GzEncoder};
#[cfg(unix)]
use std::fs;

#[test]
fn release_tags_and_assets_use_the_installer_contract() {
    assert_eq!(update::parse_tag("v0.4.0").unwrap().to_string(), "0.4.0");
    assert!(update::parse_tag("0.4.0").is_err());
    assert!(update::parse_tag("v0.4.0/other").is_err());
    assert_eq!(
        update::asset_name("v0.4.0", "x86_64-unknown-linux-gnu"),
        "argui-cli-v0.4.0-x86_64-unknown-linux-gnu.tar.gz"
    );
    assert_eq!(
        update::asset_name("v0.4.0", "x86_64-pc-windows-msvc"),
        "argui-cli-v0.4.0-x86_64-pc-windows-msvc.zip"
    );
    assert!(update::run(&["--unknown".into()]).is_err());
}

#[test]
fn checksum_requires_exact_asset_and_digest() {
    let asset = "argui-cli-v0.4.0-x86_64-unknown-linux-gnu.tar.gz";
    let digest = format!("{:x}", Sha256::digest(b"archive"));
    let sidecar = format!("{digest}  {asset}\n");
    assert!(update::verify_checksum(sidecar.as_bytes(), asset, &digest).is_ok());
    assert!(update::verify_checksum(sidecar.as_bytes(), "other.tar.gz", &digest).is_err());
    let wrong = format!("{:x}", Sha256::digest(b"tampered"));
    assert!(
        update::verify_checksum(sidecar.as_bytes(), asset, &wrong)
            .unwrap_err()
            .contains("SHA-256 mismatch")
    );
}

#[cfg(unix)]
#[test]
fn verified_archive_can_replace_a_staged_binary() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let installed = directory.path().join("argui");
    fs::write(&installed, b"old executable").unwrap();
    fs::set_permissions(&installed, fs::Permissions::from_mode(0o755)).unwrap();
    let archive = directory.path().join("release.tar.gz");
    make_tar(&archive, "argui", b"new executable");
    let mut staged = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    update::extract_binary(
        archive.as_path(),
        staged.as_file_mut(),
        "x86_64-unknown-linux-gnu",
    )
    .unwrap();
    update::replace_binary(staged, &installed).unwrap();
    assert_eq!(fs::read(&installed).unwrap(), b"new executable");
    assert_eq!(
        fs::metadata(&installed).unwrap().permissions().mode() & 0o777,
        0o755
    );
}

#[cfg(unix)]
#[test]
fn malformed_archive_never_replaces_installed_binary() {
    let directory = tempfile::tempdir().unwrap();
    let installed = directory.path().join("argui");
    fs::write(&installed, b"old executable").unwrap();
    let archive = directory.path().join("release.tar.gz");
    make_tar(&archive, "different", b"new executable");
    let mut staged = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    assert!(
        update::extract_binary(
            archive.as_path(),
            staged.as_file_mut(),
            "x86_64-unknown-linux-gnu"
        )
        .is_err()
    );
    assert_eq!(fs::read(&installed).unwrap(), b"old executable");
}

#[cfg(unix)]
fn make_tar(path: &std::path::Path, name: &str, binary: &[u8]) {
    let archive = fs::File::create(path).unwrap();
    let encoder = GzEncoder::new(archive, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_path(name).unwrap();
    header.set_size(binary.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append(&header, binary).unwrap();
    tar.into_inner().unwrap().finish().unwrap();
}

#[cfg(windows)]
#[test]
fn zip_archive_extracts_only_the_cli_binary() {
    use std::{fs, io::Write};

    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("release.zip");
    let file = fs::File::create(&archive).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file(
        "argui.exe",
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(b"new executable").unwrap();
    zip.finish().unwrap();
    let mut staged = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    update::extract_binary(
        archive.as_path(),
        staged.as_file_mut(),
        "x86_64-pc-windows-msvc",
    )
    .unwrap();
    assert_eq!(fs::read(staged.path()).unwrap(), b"new executable");
}
