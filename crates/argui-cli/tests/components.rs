#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, Project, component_files, install, run_in};
use std::{fs, path::Path};

const REGISTRY: &str = include_str!("fixtures/registry.json");

/// Writes a minimal project manifest for an offline component install.
fn project(path: &Path, framework: &str) {
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(
        path.join("argui.json"),
        format!(r#"{{"name":"demo","framework":"{framework}"}}"#),
    )
    .unwrap();
}

#[test]
/// Shared component dependencies are only downloaded once per install.
fn resolves_component_dependencies_once() {
    let paths = component_files(
        REGISTRY,
        Framework::Solid,
        &["button".into(), "popover".into()],
    )
    .unwrap();
    assert_eq!(paths.len(), 5);
    assert!(paths.contains(&"solid/button.tsx".into()));
    assert!(paths.contains(&"solid/popover.tsx".into()));
    assert!(
        component_files(REGISTRY, Framework::React, &["input-field".into()])
            .unwrap()
            .contains(&"shared/input-text.ts".into())
    );
}

#[test]
/// Registry input cannot select unknown names or escape the source tree.
fn rejects_unknown_components_and_unsafe_registry_paths() {
    assert!(component_files(REGISTRY, Framework::Solid, &["missing".into()]).is_err());
    assert!(component_files(REGISTRY, Framework::Solid, &["../bad".into()]).is_err());
    let unsafe_registry = r#"{"version":1,"components":{"button":{"solid":["../escape.tsx"]}}}"#;
    assert!(component_files(unsafe_registry, Framework::Solid, &["button".into()]).is_err());
    let missing_variant = r#"{"version":1,"components":{"button":{"react":["react/button.tsx"]}}}"#;
    assert!(component_files(missing_variant, Framework::Solid, &["button".into()]).is_err());
    let newer_version = r#"{"version":2,"components":{}}"#;
    assert!(component_files(newer_version, Framework::Solid, &["button".into()]).is_err());
    assert!(component_files("not JSON", Framework::Solid, &["button".into()]).is_err());
    for path in [
        "",
        "Solid/button.tsx",
        "solid/button.js",
        "/tmp/a.ts",
        "solid/../a.ts",
    ] {
        let registry =
            format!(r#"{{"version":1,"components":{{"button":{{"solid":["{path}"]}}}}}}"#);
        assert!(component_files(&registry, Framework::Solid, &["button".into()]).is_err());
    }
    let long_path = format!("solid/{}.tsx", "a".repeat(180));
    let registry =
        format!(r#"{{"version":1,"components":{{"button":{{"solid":["{long_path}"]}}}}}}"#);
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
        REGISTRY,
        source,
    )
    .unwrap();
    let manifest: Project =
        serde_json::from_slice(&fs::read(temp.path().join("argui.json")).unwrap()).unwrap();
    assert_eq!(manifest.components, ["solid/button"]);
    assert_eq!(manifest.component_files.len(), 4);
    assert!(temp.path().join("src/argui-ui/solid/button.tsx").is_file());
    install(
        temp.path(),
        Framework::Solid,
        &["button".into(), "popover".into()],
        REGISTRY,
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
            REGISTRY,
            |_| Ok(String::new())
        )
        .is_err()
    );
    let target = temp.path().join("src/argui-ui/react/button.tsx");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "my version").unwrap();
    assert!(
        install(
            temp.path(),
            Framework::React,
            &["button".into()],
            REGISTRY,
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
            REGISTRY,
            |_| Err("offline".into())
        )
        .is_err()
    );
    assert!(!temp.path().join("src/argui-ui/shared/types.ts").exists());
}

#[test]
/// An output created while reading sources is never overwritten.
fn install_rejects_a_new_output_during_staging() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "solid");
    let registry = r#"{"version":1,"components":{"button":{"solid":["solid/button.tsx"]}}}"#;
    let output = temp.path().join("src/argui-ui/solid/button.tsx");
    let error = install(
        temp.path(),
        Framework::Solid,
        &["button".into()],
        registry,
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

#[cfg(unix)]
#[test]
/// Symlinked destinations and source escapes cannot redirect component files.
fn add_and_install_reject_symlink_redirects() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let app = root.join("apps/demo");
    project(&app, "solid");
    let registry = r#"{"version":1,"components":{"button":{"solid":["solid/button.tsx"]}}}"#;
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(app.join("src/argui-ui")).unwrap();
    symlink(&outside, app.join("src/argui-ui/solid")).unwrap();
    let error = install(&app, Framework::Solid, &["button".into()], registry, |_| {
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains("symlink conflict"));
    fs::remove_file(app.join("src/argui-ui/solid")).unwrap();
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::create_dir_all(root.join("components")).unwrap();
    fs::create_dir_all(root.join("packages/widgets/src/solid")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(root.join("components/registry.json"), registry).unwrap();
    fs::write(outside.join("button.tsx"), "private").unwrap();
    symlink(
        outside.join("button.tsx"),
        root.join("packages/widgets/src/solid/button.tsx"),
    )
    .unwrap();
    let error = run_in(&app, &["add".into(), "solid".into(), "button".into()]).unwrap_err();
    assert!(error.contains("escapes widget directory"));
}

#[test]
/// The command copies only the declared files from its versioned checkout.
fn add_reads_the_local_registry_and_component_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let app = root.join("apps/demo");
    project(&app, "solid");
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::create_dir_all(root.join("components")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(root.join("components/registry.json"), REGISTRY).unwrap();
    for path in component_files(REGISTRY, Framework::Solid, &["input-field".into()]).unwrap() {
        let source = root.join("packages/widgets/src").join(&path);
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(source, format!("fixture source: {path}")).unwrap();
    }
    run_in(&app, &["add".into(), "solid".into(), "input-field".into()]).unwrap();
    assert!(app.join("src/argui-ui/solid/input-field.tsx").is_file());
    assert!(app.join("src/argui-ui/solid/button.tsx").is_file());
    assert!(!app.join("src/argui-ui/solid/popover.tsx").exists());
    assert!(!app.join("src/argui-ui/react/button.tsx").exists());
}

#[test]
/// Oversized checkout sources never reach the application directory.
fn add_rejects_oversized_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let app = root.join("apps/demo");
    project(&app, "solid");
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::create_dir_all(root.join("components")).unwrap();
    fs::create_dir_all(root.join("packages/widgets/src/solid")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(
        root.join("components/registry.json"),
        r#"{"version":1,"components":{"button":{"solid":["solid/button.tsx"]}}}"#,
    )
    .unwrap();
    fs::write(
        root.join("packages/widgets/src/solid/button.tsx"),
        vec![b'x'; 256 * 1024 + 1],
    )
    .unwrap();
    let error = run_in(&app, &["add".into(), "solid".into(), "button".into()]).unwrap_err();
    assert!(error.contains("exceeds 262144 bytes"));
    assert!(!app.join("src/argui-ui/solid/button.tsx").exists());
}

#[test]
/// Missing or malformed checkout files fail before an app receives components.
fn add_explains_incomplete_checkout_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let app = root.join("apps/demo");
    project(&app, "solid");
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::create_dir_all(root.join("components")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    let args = ["add".into(), "solid".into(), "button".into()];
    assert!(run_in(&app, &args).unwrap_err().contains("registry.json"));
    fs::write(root.join("components/registry.json"), vec![0xff]).unwrap();
    assert!(run_in(&app, &args).unwrap_err().contains("invalid UTF-8"));
    fs::write(
        root.join("components/registry.json"),
        r#"{"version":1,"components":{"button":{"solid":["solid/button.tsx"]}}}"#,
    )
    .unwrap();
    assert!(
        run_in(&app, &args)
            .unwrap_err()
            .contains("widget source directory")
    );
    fs::create_dir_all(root.join("packages/widgets/src")).unwrap();
    assert!(
        run_in(&app, &args)
            .unwrap_err()
            .contains("solid/button.tsx")
    );
    fs::create_dir_all(root.join("packages/widgets/src/solid")).unwrap();
    fs::write(
        root.join("packages/widgets/src/solid/button.tsx"),
        vec![0xff],
    )
    .unwrap();
    assert!(run_in(&app, &args).unwrap_err().contains("invalid UTF-8"));
    assert!(!app.join("src/argui-ui/solid/button.tsx").exists());
}

#[test]
/// Every production registry entry includes its existing local TSX imports.
fn production_registry_matches_split_widget_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = fs::read_to_string(root.join("components/registry.json")).unwrap();
    for framework in [Framework::Solid, Framework::React] {
        for name in ["button", "input-field", "select", "popover", "dialog"] {
            let files = component_files(&registry, framework, &[name.into()]).unwrap();
            let expected = format!("{}/{name}.tsx", framework.name());
            assert!(files.contains(&expected), "{name} lacks its own file");
            for file in &files {
                let source = root.join("packages/widgets/src").join(file);
                let text = fs::read_to_string(&source).unwrap();
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
