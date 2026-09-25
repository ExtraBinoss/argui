#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, install};
use std::{fs, path::Path};

/// Writes a minimal project manifest for an offline component install.
fn project(path: &Path) {
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(
        path.join("argui.json"),
        r#"{"name":"demo","framework":"solid"}"#,
    )
    .unwrap();
}

#[test]
/// Registry, manifest, stage, and output failures preserve the original project.
fn install_rolls_back_filesystem_faults() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path();
    let registry = r#"{"version":1,"components":{"button":{"solid":["solid/button.tsx"]}}}"#;
    project(app);
    let manifest_path = app.join("argui.json");
    let original = fs::read(&manifest_path).unwrap();
    assert!(
        install(app, Framework::Solid, &["button".into()], "{", |_| Ok(
            "x".into()
        ))
        .is_err()
    );

    let stage = app.join(format!(".argui-component-stage-{}", std::process::id()));
    fs::create_dir(&stage).unwrap();
    let error = install(app, Framework::Solid, &["button".into()], registry, |_| {
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains(".argui-component-stage-"), "{error}");
    fs::remove_dir(stage).unwrap();

    let error = install(app, Framework::Solid, &["button".into()], registry, |_| {
        fs::remove_file(&manifest_path).unwrap();
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains("No such file"), "{error}");
    fs::write(&manifest_path, &original).unwrap();

    let output_parent = app.join("src/argui-ui/solid");
    fs::create_dir_all(output_parent.parent().unwrap()).unwrap();
    let error = install(app, Framework::Solid, &["button".into()], registry, |_| {
        fs::write(&output_parent, "user content").unwrap();
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains("solid/button.tsx"), "{error}");
    assert_eq!(fs::read(&manifest_path).unwrap(), original);
    assert_eq!(fs::read_to_string(&output_parent).unwrap(), "user content");
}
