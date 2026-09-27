#![cfg(test)]

use crate::update;

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::JoinHandle,
};

#[cfg(unix)]
use flate2::{Compression, write::GzEncoder};
#[cfg(unix)]
use std::fs;

/// Serves one local HTTP response and returns its URL and server thread.
/// `body` is sent verbatim; `length` controls whether a Content-Length header
/// is present, allowing tests to exercise both header and streaming limits.
fn serve(body: &[u8], status: &str, length: Option<u64>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_vec();
    let status = status.to_owned();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 1024];
        let received = stream.read(&mut request).unwrap();
        assert!(received > 0);
        write!(stream, "HTTP/1.1 {status}\r\nConnection: close\r\n").unwrap();
        if let Some(length) = length {
            write!(stream, "Content-Length: {length}\r\n").unwrap();
        }
        stream.write_all(b"\r\n").unwrap();
        let _ = stream.write_all(&body);
    });
    (url, server)
}

/// Serves the archive and its sidecar from one local release URL.
/// `asset` is the expected archive basename; both requests must use it.
#[cfg(unix)]
fn serve_release(asset: &str, archive: &[u8], sidecar: &[u8]) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let asset = asset.to_owned();
    let archive = archive.to_vec();
    let sidecar = sidecar.to_vec();
    let server = std::thread::spawn(move || {
        let checksum_name = format!("{asset}.sha256");
        for (suffix, body) in [
            (asset.as_str(), archive.as_slice()),
            (checksum_name.as_str(), sidecar.as_slice()),
        ] {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let read = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("GET /releases/download/v0.4.1/"));
            assert!(request.contains(suffix));
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(body).unwrap();
        }
    });
    (url, server)
}

/// Uses direct localhost HTTP connections so update tests cannot reach GitHub.
fn client() -> Client {
    Client::builder().no_proxy().build().unwrap()
}

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
    for malformed in [b"".as_slice(), b"no-asset", b"\xff"] {
        assert!(update::verify_checksum(malformed, asset, &digest).is_err());
    }
    assert!(
        update::verify_checksum(format!("{digest} {asset} extra").as_bytes(), asset, &digest)
            .is_err()
    );
    assert!(
        update::verify_checksum(
            format!("{} {asset}", "g".repeat(64)).as_bytes(),
            asset,
            &digest
        )
        .is_err()
    );
    assert!(
        update::verify_checksum(
            format!("{} {asset}", digest.to_uppercase()).as_bytes(),
            asset,
            &digest
        )
        .is_ok()
    );
}

#[test]
/// Release metadata accepts only a valid versioned tag.
fn release_metadata_rejects_missing_and_invalid_tags() {
    for (body, expected) in [
        (br#"{"tag_name":"v0.4.1"}"#.as_slice(), Some("0.4.1")),
        (br#"{"name":"missing"}"#, None),
        (br#"{"tag_name":"latest"}"#, None),
        (b"not json", None),
    ] {
        let (url, server) = serve(body, "200 OK", Some(body.len() as u64));
        let version = update::latest_release(&client(), &url);
        server.join().unwrap();
        assert_eq!(
            version.ok().map(|value| value.to_string()).as_deref(),
            expected
        );
    }
}

#[test]
/// Check mode reports newer releases without touching the running executable.
fn update_check_uses_release_version_without_installing() {
    for (version, options) in [
        (env!("CARGO_PKG_VERSION"), Vec::<String>::new()),
        ("0.4.1", vec!["--check".to_owned()]),
    ] {
        let body = format!(r#"{{"tag_name":"v{version}"}}"#);
        let (url, server) = serve(body.as_bytes(), "200 OK", Some(body.len() as u64));
        assert!(update::run_with_release_endpoint(&options, &url).is_ok());
        server.join().unwrap();
    }
}

#[test]
/// Small release responses reject failed requests and both size-limit paths.
fn bounded_metadata_download_rejects_oversized_or_failed_responses() {
    let payload = b"metadata";
    let (url, server) = serve(payload, "200 OK", Some(payload.len() as u64));
    assert_eq!(
        update::download_small(&client(), &url, 16).unwrap(),
        payload
    );
    server.join().unwrap();

    for (status, header) in [
        ("404 Not Found", Some(8)),
        ("200 OK", Some(32)),
        ("200 OK", None),
    ] {
        let (url, server) = serve(payload, status, header);
        assert!(update::download_small(&client(), &url, 4).is_err());
        server.join().unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let refused = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    assert!(update::download_small(&client(), &refused, 16).is_err());
}

#[test]
/// Archive downloads stream their bytes and verify both advertised and actual limits.
fn archive_download_hashes_bytes_and_rejects_bad_responses() {
    let payload = b"archive bytes";
    let (url, server) = serve(payload, "200 OK", Some(payload.len() as u64));
    let mut staged = tempfile::NamedTempFile::new().unwrap();
    let digest = update::download_archive(&client(), &url, staged.as_file_mut()).unwrap();
    server.join().unwrap();
    assert_eq!(digest, format!("{:x}", Sha256::digest(payload)));
    assert_eq!(std::fs::read(staged.path()).unwrap(), payload);

    for (status, length) in [
        ("404 Not Found", Some(0)),
        ("200 OK", Some(update::MAX_ARCHIVE_BYTES + 1)),
    ] {
        let (url, server) = serve(b"", status, length);
        let mut staged = tempfile::NamedTempFile::new().unwrap();
        assert!(update::download_archive(&client(), &url, staged.as_file_mut()).is_err());
        server.join().unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let refused = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let mut staged = tempfile::NamedTempFile::new().unwrap();
    assert!(update::download_archive(&client(), &refused, staged.as_file_mut()).is_err());

    let (url, server) = serve(payload, "200 OK", Some(payload.len() as u64));
    let readonly = std::fs::File::open(staged.path()).unwrap();
    let mut readonly = readonly;
    assert!(update::download_archive(&client(), &url, &mut readonly).is_err());
    server.join().unwrap();
}

#[test]
/// Truncated HTTP bodies fail instead of accepting partial metadata or archives.
fn truncated_http_bodies_are_rejected() {
    let (url, server) = serve(b"short", "200 OK", Some(40));
    assert!(update::download_small(&client(), &url, 64).is_err());
    server.join().unwrap();

    let (url, server) = serve(b"short", "200 OK", Some(40));
    let mut staged = tempfile::NamedTempFile::new().unwrap();
    assert!(update::download_archive(&client(), &url, staged.as_file_mut()).is_err());
    server.join().unwrap();
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
/// A complete update verifies the downloaded sidecar before replacing the executable.
fn release_installation_rejects_tampering_and_accepts_a_verified_archive() {
    let directory = tempfile::tempdir().unwrap();
    let installed = directory.path().join("argui");
    fs::write(&installed, b"old executable").unwrap();
    let archive = directory.path().join("release.tar.gz");
    make_tar(&archive, "argui", b"new executable");
    let body = fs::read(&archive).unwrap();
    let asset = update::asset_name("v0.4.1", "x86_64-unknown-linux-gnu");
    let checksum = format!("{:x}  {asset}\n", Sha256::digest(&body));
    let version = semver::Version::parse("0.4.1").unwrap();

    let (url, server) = serve_release(
        &asset,
        &body,
        format!("{}  {asset}\n", "0".repeat(64)).as_bytes(),
    );
    let error = update::install_release(
        &client(),
        &version,
        "x86_64-unknown-linux-gnu",
        &installed,
        &url,
    )
    .unwrap_err();
    server.join().unwrap();
    assert!(error.contains("SHA-256 mismatch"), "{error}");
    assert_eq!(fs::read(&installed).unwrap(), b"old executable");

    let (url, server) = serve_release(&asset, &body, checksum.as_bytes());
    update::install_release(
        &client(),
        &version,
        "x86_64-unknown-linux-gnu",
        &installed,
        &url,
    )
    .unwrap();
    server.join().unwrap();
    assert_eq!(fs::read(&installed).unwrap(), b"new executable");
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
    let mut staged = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    assert!(
        update::extract_binary(&archive, staged.as_file_mut(), "x86_64-pc-windows-msvc").is_err()
    );
    assert!(
        update::extract_binary(
            &directory.path().join("missing.tar.gz"),
            staged.as_file_mut(),
            "x86_64-unknown-linux-gnu"
        )
        .is_err()
    );
    assert!(update::replace_binary(staged, &directory.path().join("missing")).is_err());
}

#[cfg(unix)]
#[test]
/// Malformed tar entries and unwritable staging files never change the installed CLI.
fn archive_extraction_and_replacement_reject_unsafe_files() {
    let directory = tempfile::tempdir().unwrap();
    let installed = directory.path().join("argui");
    fs::write(&installed, b"old executable").unwrap();
    let archive = directory.path().join("release.tar.gz");
    let mut staged = tempfile::NamedTempFile::new_in(directory.path()).unwrap();

    fs::write(&archive, b"not gzip").unwrap();
    assert!(
        update::extract_binary(&archive, staged.as_file_mut(), "x86_64-unknown-linux-gnu").is_err()
    );

    let file = fs::File::create(&archive).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    tar::Builder::new(encoder)
        .into_inner()
        .unwrap()
        .finish()
        .unwrap();
    assert!(
        update::extract_binary(&archive, staged.as_file_mut(), "x86_64-unknown-linux-gnu").is_err()
    );

    let file = fs::File::create(&archive).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    for _ in 0..2 {
        let mut header = tar::Header::new_gnu();
        header.set_path("argui").unwrap();
        header.set_size(3);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append(&header, b"new".as_slice()).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap();
    assert!(
        update::extract_binary(&archive, staged.as_file_mut(), "x86_64-unknown-linux-gnu").is_err()
    );

    make_tar(&archive, "argui", b"new executable");
    let mut readonly = fs::File::open(staged.path()).unwrap();
    assert!(update::extract_binary(&archive, &mut readonly, "x86_64-unknown-linux-gnu").is_err());

    let missing_stage = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    fs::remove_file(missing_stage.path()).unwrap();
    assert!(update::replace_binary(missing_stage, &installed).is_err());
    let directory_stage = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    assert!(update::replace_binary(directory_stage, directory.path()).is_err());
    assert_eq!(fs::read(installed).unwrap(), b"old executable");
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
