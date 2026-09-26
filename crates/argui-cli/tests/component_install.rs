#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, install, run_in};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

/// Creates a standalone app for an offline component install.
fn project(path: &Path) {
    run_in(
        path,
        &[
            "init".into(),
            "solid".into(),
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

/// Returns a pinned one-file registry whose source callback yields `x`.
fn registry() -> String {
    let release = env!("CARGO_PKG_VERSION");
    serde_json::json!({
        "version": 2,
        "arguiVersion": release,
        "files": { "solid/button.tsx": format!("{:x}", Sha256::digest(b"x")) },
        "components": {
            "button": {
                "version": release,
                "source": {
                    "solid": format!("https://github.com/ExtraBinoss/argui/blob/v{release}/packages/widgets/src/solid/button.tsx"),
                    "react": format!("https://github.com/ExtraBinoss/argui/blob/v{release}/packages/widgets/src/react/button.tsx")
                },
                "solid": ["solid/button.tsx"],
                "react": []
            }
        }
    }).to_string()
}

#[test]
/// Registry, manifest, stage, and output failures preserve the original project.
fn install_rolls_back_filesystem_faults() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path();
    let registry = registry();
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
    let error = install(app, Framework::Solid, &["button".into()], &registry, |_| {
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains(".argui-component-stage-"), "{error}");
    fs::remove_dir(stage).unwrap();

    let error = install(app, Framework::Solid, &["button".into()], &registry, |_| {
        fs::remove_file(&manifest_path).unwrap();
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains("No such file"), "{error}");
    fs::write(&manifest_path, &original).unwrap();

    let output_parent = app.join("ui/solid-components");
    if output_parent.is_dir() {
        fs::remove_dir_all(&output_parent).unwrap();
    }
    fs::create_dir_all(output_parent.parent().unwrap()).unwrap();
    let error = install(app, Framework::Solid, &["button".into()], &registry, |_| {
        fs::write(&output_parent, "user content").unwrap();
        Ok("x".into())
    })
    .unwrap_err();
    assert!(error.contains("button.tsx"), "{error}");
    assert_eq!(fs::read(&manifest_path).unwrap(), original);
    assert_eq!(fs::read_to_string(&output_parent).unwrap(), "user content");
}
