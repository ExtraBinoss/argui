#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, Project, catalog, component_files, install, run_in};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

/// Returns the pinned fixture registry at the crate version under test.
fn fixture_registry() -> &'static str {
    static REGISTRY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            include_str!("fixtures/registry.json").replace("0.3.3", env!("CARGO_PKG_VERSION"))
        })
        .as_str()
}

/// Builds one release-pinned registry fixture with an exact source checksum.
fn versioned_registry(body: &str) -> String {
    serde_json::json!({
        "version": 2,
        "arguiVersion": env!("CARGO_PKG_VERSION"),
        "files": { "solid/badge.tsx": format!("{:x}", Sha256::digest(body.as_bytes())) },
        "components": {
            "badge": {
                "version": env!("CARGO_PKG_VERSION"),
                "source": { "solid": format!("https://github.com/ExtraBinoss/argui/blob/v{}/packages/widgets/src/solid/badge.tsx", env!("CARGO_PKG_VERSION")),
                    "react": format!("https://github.com/ExtraBinoss/argui/blob/v{}/packages/widgets/src/react/badge.tsx", env!("CARGO_PKG_VERSION")) },
                "solid": ["solid/badge.tsx"],
                "react": []
            }
        }
    }).to_string()
}

/// Builds one v2 button registry with a selected source path and checksum.
fn button_registry(path: &str, body: &str) -> String {
    let release = env!("CARGO_PKG_VERSION");
    serde_json::json!({
        "version": 2,
        "arguiVersion": release,
        "files": { path: format!("{:x}", Sha256::digest(body.as_bytes())) },
        "components": {
            "button": {
                "version": release,
                "source": {
                    "solid": format!("https://github.com/ExtraBinoss/argui/blob/v{release}/packages/widgets/src/solid/button.tsx"),
                    "react": format!("https://github.com/ExtraBinoss/argui/blob/v{release}/packages/widgets/src/react/button.tsx")
                },
                "solid": [path],
                "react": []
            }
        }
    }).to_string()
}

#[test]
/// Versioned installs validate source bytes before mutation and protect edited local files.
fn versioned_install_rejects_checksum_mismatch_and_local_edits() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "solid");
    let registry = versioned_registry("trusted source");
    let mismatched = registry.replace(env!("CARGO_PKG_VERSION"), "0.0.0");
    assert!(
        install(
            temp.path(),
            Framework::Solid,
            &["badge".into()],
            &mismatched,
            |_| Ok("trusted source".into())
        )
        .unwrap_err()
        .contains("registry version does not match")
    );
    assert!(
        install(
            temp.path(),
            Framework::Solid,
            &["badge".into()],
            &registry,
            |_| Ok("changed source".into())
        )
        .unwrap_err()
        .contains("checksum mismatch")
    );
    assert!(!temp.path().join("ui/solid-components/badge.tsx").exists());
    install(
        temp.path(),
        Framework::Solid,
        &["badge".into()],
        &registry,
        |_| Ok("trusted source".into()),
    )
    .unwrap();
    let installed = temp.path().join("ui/solid-components/badge.tsx");
    let manifest: Project =
        serde_json::from_slice(&fs::read(temp.path().join("argui.json")).unwrap()).unwrap();
    assert!(manifest.component_checksums.contains_key("solid/badge.tsx"));
    assert_eq!(
        manifest.component_versions["solid/badge"],
        env!("CARGO_PKG_VERSION")
    );
    fs::write(&installed, "local change").unwrap();
    assert!(
        install(
            temp.path(),
            Framework::Solid,
            &["badge".into()],
            &registry,
            |_| Ok("trusted source".into())
        )
        .unwrap_err()
        .contains("locally modified")
    );
    assert_eq!(fs::read_to_string(installed).unwrap(), "local change");
}

#[test]
/// The release catalog reports pinned source, adapter, installed state, and file paths.
fn catalog_reports_versioned_component_metadata() {
    let registry = versioned_registry("trusted source");
    let output = catalog(&registry, None, Some(Framework::Solid), true).unwrap();
    let entries: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["name"], "badge");
    assert_eq!(entries[0]["framework"], "solid");
    assert_eq!(entries[0]["files"][0], "solid/badge.tsx");
    assert!(entries[0]["source"].as_str().unwrap().contains("/blob/v"));
}

#[test]
/// Versioned entries require complete provenance and checksums for every selected file.
fn versioned_registry_rejects_incomplete_release_metadata() {
    let base: serde_json::Value =
        serde_json::from_str(&versioned_registry("trusted source")).unwrap();
    type RegistryChange = (Box<dyn Fn(&mut serde_json::Value)>, &'static str);
    let changes: [RegistryChange; 7] = [
        (
            Box::new(|value| value["arguiVersion"] = "".into()),
            "lacks arguiVersion",
        ),
        (
            Box::new(|value| value["components"]["badge"]["version"] = "".into()),
            "immutable source URL or version",
        ),
        (
            Box::new(|value| {
                value["components"]["badge"]["source"]
                    .as_object_mut()
                    .unwrap()
                    .remove("solid");
            }),
            "source does not match",
        ),
        (
            Box::new(|value| {
                value["components"]["badge"]["source"]["solid"] =
                    "https://elsewhere.test/blob/v0".into()
            }),
            "source does not match",
        ),
        (
            Box::new(|value| {
                value["files"].as_object_mut().unwrap().clear();
            }),
            "lacks a SHA-256 checksum",
        ),
        (
            Box::new(|value| value["files"]["solid/badge.tsx"] = "ABC".into()),
            "lacks a SHA-256 checksum",
        ),
        (
            Box::new(|value| {
                value["components"]["badge"] =
                    serde_json::json!({"solid":["solid/badge.tsx"],"react":[]})
            }),
            "missing field",
        ),
    ];
    for (change, expected) in changes {
        let mut registry = base.clone();
        change(&mut registry);
        let error = component_files(&registry.to_string(), Framework::Solid, &["badge".into()])
            .unwrap_err();
        assert!(error.contains(expected), "{registry}: {error}");
    }
    let filtered = catalog(&base.to_string(), None, Some(Framework::Solid), false).unwrap();
    assert!(filtered.contains("badge"));
    assert!(!filtered.contains("react"));
}

/// Creates a standalone v2 app for offline component installation tests.
fn project(path: &Path, framework: &str) {
    run_in(
        path,
        &[
            "init".into(),
            framework.into(),
            "--dir".into(),
            path.to_string_lossy().into_owned(),
            "--name".into(),
            "demo".into(),
            "--yes".into(),
            "--no-install".into(),
        ],
    )
    .unwrap();
}

#[test]
/// Shared component dependencies are only downloaded once per install.
fn resolves_component_dependencies_once() {
    let paths = component_files(
        fixture_registry(),
        Framework::Solid,
        &["button".into(), "popover".into()],
    )
    .unwrap();
    assert_eq!(paths.len(), 5);
    assert!(paths.contains(&"solid/button.tsx".into()));
    assert!(paths.contains(&"solid/popover.tsx".into()));
    assert!(
        component_files(
            fixture_registry(),
            Framework::React,
            &["input-field".into()]
        )
        .unwrap()
        .contains(&"shared/input-text.ts".into())
    );
}

#[test]
/// Registry input cannot select unknown names or escape the source tree.
fn rejects_unknown_components_and_unsafe_registry_paths() {
    assert!(component_files(fixture_registry(), Framework::Solid, &["missing".into()]).is_err());
    assert!(component_files(fixture_registry(), Framework::Solid, &["../bad".into()]).is_err());
    let unsafe_registry = button_registry("../escape.tsx", "x");
    assert!(component_files(&unsafe_registry, Framework::Solid, &["button".into()]).is_err());
    let mut missing_variant: serde_json::Value =
        serde_json::from_str(&button_registry("solid/button.tsx", "x")).unwrap();
    missing_variant["components"]["button"]
        .as_object_mut()
        .unwrap()
        .remove("solid");
    assert!(
        component_files(
            &missing_variant.to_string(),
            Framework::Solid,
            &["button".into()]
        )
        .is_err()
    );
    let mut old_version: serde_json::Value =
        serde_json::from_str(&button_registry("solid/button.tsx", "x")).unwrap();
    old_version["version"] = 1.into();
    assert!(
        component_files(
            &old_version.to_string(),
            Framework::Solid,
            &["button".into()]
        )
        .is_err()
    );
    assert!(component_files("not JSON", Framework::Solid, &["button".into()]).is_err());
    for path in [
        "",
        "Solid/button.tsx",
        "solid/button.js",
        "/tmp/a.ts",
        "solid/../a.ts",
    ] {
        let registry = button_registry(path, "x");
        assert!(component_files(&registry, Framework::Solid, &["button".into()]).is_err());
    }
    let long_path = format!("solid/{}.tsx", "a".repeat(180));
    let registry = button_registry(&long_path, "x");
    assert!(component_files(&registry, Framework::Solid, &["button".into()]).is_err());
}

#[test]
/// Installs the declared dependency files and skips tracked files on repeat runs.
fn install_tracks_and_reuses_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "solid");
    let source = |path: &str| Ok(format!("fixture source: {path}"));
    install(
        temp.path(),
        Framework::Solid,
        &["button".into()],
        fixture_registry(),
        source,
    )
    .unwrap();
    let manifest: Project =
        serde_json::from_slice(&fs::read(temp.path().join("argui.json")).unwrap()).unwrap();
    assert_eq!(manifest.components, ["solid/button"]);
    assert_eq!(manifest.component_files.len(), 4);
    assert!(temp.path().join("ui/solid-components/button.tsx").is_file());
    install(
        temp.path(),
        Framework::Solid,
        &["button".into(), "popover".into()],
        fixture_registry(),
        |path| Ok(format!("fixture source: {path}")),
    )
    .unwrap();
    let manifest: Project =
        serde_json::from_slice(&fs::read(temp.path().join("argui.json")).unwrap()).unwrap();
    assert_eq!(manifest.components, ["solid/button", "solid/popover"]);
}

#[test]
/// Rejects local conflicts and leaves the project unchanged when a source fails.
fn install_rejects_conflicts_and_download_failures() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "react");
    assert!(
        install(
            temp.path(),
            Framework::Solid,
            &["button".into()],
            fixture_registry(),
            |_| Ok(String::new())
        )
        .is_err()
    );
    let target = temp.path().join("ui/react-components/button.tsx");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "my version").unwrap();
    assert!(
        install(
            temp.path(),
            Framework::React,
            &["button".into()],
            fixture_registry(),
            |_| Ok(String::new())
        )
        .unwrap_err()
        .contains("conflicts")
    );
    fs::remove_file(target).unwrap();
    assert!(
        install(
            temp.path(),
            Framework::React,
            &["button".into()],
            fixture_registry(),
            |_| Err("offline".into())
        )
        .is_err()
    );
    assert!(!temp.path().join("ui/shared/types.ts").exists());
}

#[test]
/// An output created while reading sources is never overwritten.
fn install_rejects_a_new_output_during_staging() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "solid");
    let registry = button_registry("solid/button.tsx", "downloaded content");
    let output = temp.path().join("ui/solid-components/button.tsx");
    let error = install(
        temp.path(),
        Framework::Solid,
        &["button".into()],
        &registry,
        |_| {
            fs::create_dir_all(output.parent().unwrap()).unwrap();
            fs::write(&output, "user content").unwrap();
            Ok("downloaded content".into())
        },
    )
    .unwrap_err();
    assert!(error.contains("appeared while reading sources"));
    assert_eq!(fs::read_to_string(output).unwrap(), "user content");
}

#[test]
/// Every production registry entry includes its existing local TSX imports.
fn production_registry_matches_split_widget_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = fs::read_to_string(root.join("components/registry.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&registry).unwrap();
    let names = manifest["components"]
        .as_object()
        .unwrap()
        .keys()
        .collect::<Vec<_>>();
    for framework in [Framework::Solid, Framework::React] {
        for name in &names {
            let files = component_files(&registry, framework, &[name.as_str().into()]).unwrap();
            let expected = format!("{}/{name}.tsx", framework.name());
            assert!(files.contains(&expected), "{name} lacks its own file");
            for file in &files {
                let source = root.join("packages/widgets/src").join(file);
                let text = fs::read_to_string(&source).unwrap();
                assert_eq!(
                    manifest["files"][file].as_str(),
                    Some(format!("{:x}", Sha256::digest(text.as_bytes())).as_str()),
                    "{name} has a stale source checksum for {file}",
                );
                for line in text
                    .lines()
                    .filter(|line| line.contains(" from './") || line.contains(" from '../"))
                {
                    let import = line.split(" from '").nth(1).unwrap().trim_end_matches('\'');
                    let sibling = source.parent().unwrap().join(import);
                    let dependency = if sibling.with_extension("tsx").is_file() {
                        sibling.with_extension("tsx")
                    } else {
                        sibling.with_extension("ts")
                    };
                    let relative = dependency
                        .canonicalize()
                        .unwrap()
                        .strip_prefix(root.join("packages/widgets/src").canonicalize().unwrap())
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    assert!(files.contains(&relative), "{name} misses {relative}");
                }
            }
        }
    }
}
