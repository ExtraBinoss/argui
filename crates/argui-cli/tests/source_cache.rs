#![cfg(unix)]

use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

#[path = "../src/source_cache.rs"]
mod source_cache;

/// Writes the fixture curl implementation as an executable.
fn curl(path: &Path) {
    fs::write(
        path,
        r##"#!/bin/sh
printf '%s\n' "$*" >> "$ARGUI_TEST_CURL_LOG"
if [ "${ARGUI_FAKE_CURL_FAIL:-}" = 1 ]; then exit 22; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = --output ]; then shift; output="$1"; else url="$1"; fi
  shift
done
if [ "${ARGUI_FAKE_CURL_LARGE:-}" = 1 ]; then head -c 270000 /dev/zero > "$output"; exit 0; fi
if [ "${ARGUI_FAKE_CURL_BAD_UTF8:-}" = 1 ]; then printf '\377' > "$output"; exit 0; fi
case "$url" in
  */components/registry.json) cp "$ARGUI_TEST_RELEASE_ROOT/registry.json" "$output" ;;
  */packages/widgets/src/*)
    relative="${url##*/packages/widgets/src/}"
    cp "$ARGUI_TEST_RELEASE_ROOT/widgets/$relative" "$output" ;;
  *) exit 23 ;;
esac
"##,
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Creates a version-pinned standalone manifest selected as a fixture release.
fn app(root: &Path, name: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["init", "solid", "--dir", name, "--yes"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let path = root.join(name).join("argui.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    manifest["distribution"] = "release".into();
    fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

/// Invokes an app command through the fake release server.
fn cli(root: &Path, app: &str, args: &[&str], flags: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_argui"));
    command
        .args(args)
        .current_dir(root.join(app))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ARGUI_CACHE_DIR", root.join("cache"))
        .env("ARGUI_TEST_RELEASE_ROOT", root.join("release"))
        .env("ARGUI_TEST_CURL_LOG", root.join("curl.log"));
    for (key, value) in flags {
        command.env(key, value);
    }
    command.output().unwrap()
}

/// Builds the registry and source bodies for an exact synthetic version tag.
fn release(root: &Path) {
    let version = env!("CARGO_PKG_VERSION");
    let source = root.join("release/widgets");
    fs::create_dir_all(source.join("solid")).unwrap();
    fs::create_dir_all(source.join("react")).unwrap();
    let solid = "export const Badge = 'solid'\n";
    let react = "export const Badge = 'react'\n";
    fs::write(source.join("solid/badge.tsx"), solid).unwrap();
    fs::write(source.join("react/badge.tsx"), react).unwrap();
    let registry = serde_json::json!({
        "version": 2,
        "arguiVersion": version,
        "files": {
            "solid/badge.tsx": format!("{:x}", Sha256::digest(solid.as_bytes())),
            "react/badge.tsx": format!("{:x}", Sha256::digest(react.as_bytes())),
        },
        "components": {"badge": {
            "version": version,
            "source": {
                "solid": format!("https://github.com/ExtraBinoss/argui/blob/v{version}/packages/widgets/src/solid/badge.tsx"),
                "react": format!("https://github.com/ExtraBinoss/argui/blob/v{version}/packages/widgets/src/react/badge.tsx"),
            },
            "solid": ["solid/badge.tsx"],
            "react": ["react/badge.tsx"],
        }}
    });
    fs::write(
        root.join("release/registry.json"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .unwrap();
}

#[test]
/// Exact-tag registry and sources are fetched once, verified, and installed atomically.
fn tagged_sources_are_cached_and_verified() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "first");
    let output = cli(
        root,
        "first",
        &["add", "--solid", "badge", "--react", "badge"],
        &[],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("first/ui/solid-components/badge.tsx").is_file());
    assert!(root.join("first/ui/react-components/badge.tsx").is_file());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("first/argui.json")).unwrap()).unwrap();
    assert_eq!(
        manifest["components"],
        serde_json::json!(["react/badge", "solid/badge"])
    );
    let downloaded = fs::read_to_string(root.join("curl.log")).unwrap();
    assert_eq!(downloaded.lines().count(), 3);
    let listed = cli(root, "first", &["list", "components", "--json"], &[]);
    assert!(listed.status.success());
    let rows: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 2);
    assert!(rows[0]["source"].as_str().unwrap().contains("/blob/v"));
    assert_eq!(
        fs::read_to_string(root.join("curl.log")).unwrap(),
        downloaded
    );

    app(root, "second");
    let second = cli(root, "second", &["add", "badge", "--solid"], &[]);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.join("curl.log")).unwrap(),
        downloaded
    );

    let cache = root.join(format!(
        "cache/{}/widgets/solid/badge.tsx",
        env!("CARGO_PKG_VERSION")
    ));
    fs::write(&cache, "tampered cache").unwrap();
    fs::write(
        root.join("release/widgets/solid/badge.tsx"),
        "tampered release",
    )
    .unwrap();
    app(root, "third");
    let bad = cli(root, "third", &["add", "badge", "--solid"], &[]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("source checksum mismatch"));
    assert!(!root.join("third/ui").exists());
}

#[test]
/// Unavailable and oversized release sources fail before app mutation.
fn download_failures_leave_the_project_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "app");
    let original = fs::read(root.join("app/argui.json")).unwrap();
    let unavailable = cli(
        root,
        "app",
        &["add", "badge"],
        &[("ARGUI_FAKE_CURL_FAIL", "1")],
    );
    assert!(!unavailable.status.success());
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("release source unavailable"));
    let oversized = cli(
        root,
        "app",
        &["add", "badge"],
        &[("ARGUI_FAKE_CURL_LARGE", "1")],
    );
    assert!(!oversized.status.success());
    assert!(String::from_utf8_lossy(&oversized.stderr).contains("oversized"));
    assert_eq!(fs::read(root.join("app/argui.json")).unwrap(), original);
    assert!(!root.join("app/ui").exists());
}

#[test]
/// Release metadata and package errors stop mixed-adapter installs before writes.
fn release_registry_and_package_manifest_are_checked_before_install() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "wrong-version");
    let registry_path = root.join("release/registry.json");
    let original_registry = fs::read(&registry_path).unwrap();
    let mut altered: serde_json::Value = serde_json::from_slice(&original_registry).unwrap();
    altered["arguiVersion"] = "0.0.0".into();
    fs::write(&registry_path, serde_json::to_vec(&altered).unwrap()).unwrap();
    let wrong = cli(root, "wrong-version", &["add", "badge", "--react"], &[]);
    assert!(!wrong.status.success());
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("registry version does not match"));
    assert!(!root.join("wrong-version/ui").exists());

    fs::write(&registry_path, original_registry).unwrap();
    fs::remove_file(root.join(format!("cache/{}/registry.json", env!("CARGO_PKG_VERSION"))))
        .unwrap();
    for (name, package, expected) in [
        ("missing-package", None, "package.json"),
        ("malformed-package", Some("{"), "package.json"),
        (
            "missing-dependencies",
            Some(r#"{"devDependencies":{}}"#),
            "lacks dependencies",
        ),
        (
            "missing-dev",
            Some(r#"{"dependencies":{}}"#),
            "lacks devDependencies",
        ),
    ] {
        app(root, name);
        let path = root.join(name).join("package.json");
        if let Some(body) = package {
            fs::write(&path, body).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
        let before = fs::read(root.join(name).join("argui.json")).unwrap();
        let result = cli(root, name, &["add", "badge", "--react"], &[]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains(expected));
        assert_eq!(
            fs::read(root.join(name).join("argui.json")).unwrap(),
            before
        );
        assert!(!root.join(name).join("ui").exists());
    }
}

#[test]
/// An unknown source prefix cannot redirect files outside editable widget folders.
fn registry_source_prefix_is_rejected_before_fetch() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "app");
    let path = root.join("release/registry.json");
    let mut registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let hash = registry["files"]["solid/badge.tsx"].clone();
    registry["files"]
        .as_object_mut()
        .unwrap()
        .remove("solid/badge.tsx");
    registry["files"]["custom/badge.tsx"] = hash;
    registry["components"]["badge"]["solid"] = serde_json::json!(["custom/badge.tsx"]);
    registry["components"]["badge"]["source"]["solid"] = format!(
        "https://github.com/ExtraBinoss/argui/blob/v{}/packages/widgets/src/custom/badge.tsx",
        env!("CARGO_PKG_VERSION")
    )
    .into();
    fs::write(path, serde_json::to_vec(&registry).unwrap()).unwrap();
    let before = fs::read(root.join("app/argui.json")).unwrap();
    let result = cli(root, "app", &["add", "badge", "--solid"], &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unsupported widget source prefix"));
    assert_eq!(fs::read(root.join("app/argui.json")).unwrap(), before);
    assert!(!root.join("app/ui").exists());
}

#[test]
/// URL inputs reject traversal, malformed versions, and invalid checksums before network access.
fn release_source_inputs_are_validated_before_fetch() {
    let valid_hash = "a".repeat(64);
    let long_version = "1".repeat(41);
    for version in ["", "0/3", "A.1", long_version.as_str()] {
        assert!(source_cache::registry(version).is_err());
    }
    for path in [
        "",
        "../escape.tsx",
        "solid/BAD.tsx",
        "solid/a.js",
        "/x.tsx",
        "solid/a?.tsx",
    ] {
        assert!(source_cache::file("0.3.3", path, &valid_hash).is_err());
    }
    let uppercase_hash = "A".repeat(64);
    for hash in ["abc", uppercase_hash.as_str()] {
        assert!(source_cache::file("0.3.3", "solid/button.tsx", hash).is_err());
    }
}

#[test]
/// A cache directory occupying the registry file cannot be silently replaced by a download.
fn cache_destination_conflict_reports_the_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "app");
    let cache = root.join(format!("cache/{}/registry.json", env!("CARGO_PKG_VERSION")));
    fs::create_dir_all(&cache).unwrap();
    let result = cli(root, "app", &["add", "badge"], &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("registry.json"));
    assert!(cache.is_dir());
}

#[test]
/// Missing curl and malformed release bytes are actionable download errors.
fn release_download_reports_missing_tool_and_invalid_encoding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "app");
    let bad = cli(
        root,
        "app",
        &["add", "badge"],
        &[("ARGUI_FAKE_CURL_BAD_UTF8", "1")],
    );
    assert!(!bad.status.success());
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("valid UTF-8"),
        "{}",
        String::from_utf8_lossy(&bad.stderr)
    );
    let no_curl = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["add", "badge"])
        .current_dir(root.join("app"))
        .env("PATH", root.join("bin/absent"))
        .env("ARGUI_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap();
    assert!(!no_curl.status.success());
    assert!(String::from_utf8_lossy(&no_curl.stderr).contains("curl is required"));
}

#[test]
/// Release cache uses XDG or a caller-owned temporary directory when no override is set.
fn release_cache_honors_environment_fallbacks_and_parent_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("bin")).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    curl(&root.join("bin/curl"));
    release(root);
    app(root, "app");
    let invoke = |cache: Option<&Path>, xdg: Option<&Path>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_argui"));
        command
            .args(["add", "badge"])
            .current_dir(root.join("app"))
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    root.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("ARGUI_TEST_RELEASE_ROOT", root.join("release"))
            .env("ARGUI_TEST_CURL_LOG", root.join("curl.log"))
            .env("TMPDIR", root.join("tmp"))
            .env_remove("ARGUI_CACHE_DIR")
            .env_remove("XDG_CACHE_HOME");
        if let Some(cache) = cache {
            command.env("ARGUI_CACHE_DIR", cache);
        }
        if let Some(xdg) = xdg {
            command.env("XDG_CACHE_HOME", xdg);
        }
        command.output().unwrap()
    };
    let xdg = root.join("xdg");
    let first = invoke(None, Some(&xdg));
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        xdg.join(format!("argui/{}/registry.json", env!("CARGO_PKG_VERSION")))
            .is_file()
    );
    let second = invoke(None, None);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(
        root.join(format!(
            "tmp/argui-cache/{}/registry.json",
            env!("CARGO_PKG_VERSION")
        ))
        .is_file()
    );
    let blocked = root.join("blocked-cache");
    fs::write(&blocked, "occupied").unwrap();
    let failure = invoke(Some(&blocked), None);
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("Not a directory"));
}
