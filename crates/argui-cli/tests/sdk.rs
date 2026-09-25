#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::{Command, Output},
};

/// Creates an app and a fake Bun executable for SDK cache checks.
fn fixture(root: &Path) -> String {
    fs::create_dir(root.join("bin")).unwrap();
    let bun = root.join("bin/bun");
    fs::write(
        &bun,
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 1.4.0; fi\n",
    )
    .unwrap();
    fs::set_permissions(&bun, fs::Permissions::from_mode(0o755)).unwrap();
    let init = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args([
            "init",
            "solid",
            "--dir",
            "app",
            "--targets",
            "native",
            "--yes",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(init.status.success());
    fs::create_dir_all(root.join("app/node_modules/typescript")).unwrap();
    fs::create_dir_all(root.join("app/node_modules/vite")).unwrap();
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("app/argui.json")).unwrap()).unwrap();
    state["sdkSha256"].as_str().unwrap()[..16].to_owned()
}

/// Runs check with all SDK source files stored outside the app directory.
fn check(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_argui"))
        .arg("check")
        .current_dir(root.join("app"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ARGUI_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap()
}

#[test]
/// Corrupt cached SDK bytes are repaired from the CLI's embedded snapshot.
fn sdk_cache_repairs_changed_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let digest = fixture(root);
    assert!(check(root).status.success());
    let cache = root.join(format!("cache/sdk-v{}-{digest}", env!("CARGO_PKG_VERSION")));
    let source = cache.join("sdk/host/protocol.ts");
    let original = fs::read(&source).unwrap();
    fs::write(&source, "altered").unwrap();
    assert!(check(root).status.success());
    assert_eq!(fs::read(source).unwrap(), original);
    assert!(!root.join("app/sdk").exists());
}

#[test]
/// Cached file and root symlinks are rejected before the CLI writes through them.
fn sdk_cache_refuses_symlink_redirects() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let digest = fixture(root);
    assert!(check(root).status.success());
    let cache = root.join(format!("cache/sdk-v{}-{digest}", env!("CARGO_PKG_VERSION")));
    let source = cache.join("sdk/host/protocol.ts");
    let external = root.join("protected.ts");
    fs::write(&external, "private").unwrap();
    fs::remove_file(&source).unwrap();
    symlink(&external, &source).unwrap();
    let rejected = check(root);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("SDK cache asset is a symlink"));
    assert_eq!(fs::read_to_string(&external).unwrap(), "private");
    fs::remove_dir_all(&cache).unwrap();
    symlink(root.join("elsewhere"), &cache).unwrap();
    let rejected = check(root);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("SDK cache is a symlink"));
}

#[test]
/// Cache parent and destination conflicts fail before SDK bytes reach the app.
fn sdk_cache_reports_directory_conflicts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let digest = fixture(root);
    let cache = root.join(format!("cache/sdk-v{}-{digest}", env!("CARGO_PKG_VERSION")));
    fs::create_dir(root.join("cache")).unwrap();
    fs::write(&cache, "occupied").unwrap();
    let parent_error = check(root);
    assert!(!parent_error.status.success());
    assert!(String::from_utf8_lossy(&parent_error.stderr).contains("sdk-v"));
    fs::remove_file(&cache).unwrap();
    assert!(check(root).status.success());
    let asset = cache.join("sdk/host/protocol.ts");
    fs::remove_file(&asset).unwrap();
    fs::create_dir(&asset).unwrap();
    let destination_error = check(root);
    assert!(!destination_error.status.success());
    assert!(String::from_utf8_lossy(&destination_error.stderr).contains("protocol.ts"));
    assert!(!root.join("app/sdk").exists());
}
