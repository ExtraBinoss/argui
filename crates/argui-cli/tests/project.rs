#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, Project, run_in};
use std::fs;

#[test]
/// Public project metadata is the complete standalone v2 manifest.
fn project_model_round_trips_standalone_manifest() {
    let root = tempfile::tempdir().unwrap();
    run_in(
        root.path(),
        &[
            "init".into(),
            "solid".into(),
            "--dir".into(),
            "app".into(),
            "--name".into(),
            "demo".into(),
            "--yes".into(),
            "--no-install".into(),
        ],
    )
    .unwrap();
    let path = root.path().join("app/argui.json");
    let project: Project = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(project.project_version, 2);
    assert_eq!(project.framework, "solid");
    assert_eq!(project.targets, ["native", "web"]);
    assert_eq!(project.argui_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(project.sdk_sha256.len(), 64);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(path).unwrap()).unwrap(),
        serde_json::to_value(project).unwrap()
    );
}

#[test]
/// Component adapter names reject retired target and Rust selectors.
fn framework_names_are_only_tsx_adapters() {
    assert_eq!(Framework::parse("solid").unwrap(), Framework::Solid);
    assert_eq!(Framework::parse("react").unwrap(), Framework::React);
    for name in ["web", "native", "rust", "counter-web"] {
        assert!(Framework::parse(name).is_err());
    }
}
