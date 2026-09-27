#![cfg(test)]

use super::package_assets;
use std::{fs, path::PathBuf};

/// Creates a minimal application and release folder for package validation.
fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let app = directory.path().join("app");
    let output = directory.path().join("release");
    fs::create_dir_all(app.join("assets")).unwrap();
    fs::create_dir(&output).unwrap();
    fs::write(app.join("assets/file.svg"), "<svg/>").unwrap();
    fs::write(
        app.join("assets.generated.json"),
        r#"{"assets":[{"path":"file.svg"}]}"#,
    )
    .unwrap();
    (directory, app, output)
}

#[test]
/// Release packaging rejects damaged asset manifests and missing source files.
fn release_assets_require_a_valid_manifest_and_owned_source() {
    let (_directory, app, output) = fixture();
    fs::remove_file(app.join("assets.generated.json")).unwrap();
    assert!(package_assets(&app, &output).is_err());
    fs::write(app.join("assets.generated.json"), "{").unwrap();
    assert!(package_assets(&app, &output).is_err());
    fs::write(
        app.join("assets.generated.json"),
        r#"{"assets":[{"path":"file.svg"}]}"#,
    )
    .unwrap();
    fs::remove_dir_all(app.join("assets")).unwrap();
    assert!(package_assets(&app, &output).is_err());
    fs::create_dir(app.join("assets")).unwrap();
    assert!(package_assets(&app, &output).is_err());
}

#[test]
/// Release packaging reports output conflicts without copying outside its asset folder.
fn release_assets_reject_output_conflicts_and_nonfiles() {
    let (_directory, app, output) = fixture();
    fs::write(output.join("assets"), "occupied").unwrap();
    assert!(package_assets(&app, &output).is_err());
    fs::remove_file(output.join("assets")).unwrap();

    let blocked_output = app.join("output-file");
    fs::write(&blocked_output, "occupied").unwrap();
    assert!(package_assets(&app, &blocked_output).is_err());

    fs::remove_file(app.join("assets/file.svg")).unwrap();
    fs::create_dir(app.join("assets/file.svg")).unwrap();
    assert!(package_assets(&app, &output).is_err());
    fs::remove_dir(app.join("assets/file.svg")).unwrap();
    fs::write(app.join("assets/file.svg"), "<svg/>").unwrap();

    fs::create_dir(output.join("assets.generated.json")).unwrap();
    assert!(package_assets(&app, &output).is_err());
}

#[test]
/// Asset paths cannot traverse above the app root or through a symlink.
fn release_assets_reject_unsafe_paths() {
    let (_directory, app, output) = fixture();
    fs::write(
        app.join("assets.generated.json"),
        r#"{"assets":[{"path":"../private.svg"}]}"#,
    )
    .unwrap();
    assert!(
        package_assets(&app, &output)
            .unwrap_err()
            .contains("unsafe release asset path")
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        fs::write(app.join("private.svg"), "private").unwrap();
        symlink(app.join("private.svg"), app.join("assets/escape.svg")).unwrap();
        fs::write(
            app.join("assets.generated.json"),
            r#"{"assets":[{"path":"escape.svg"}]}"#,
        )
        .unwrap();
        assert!(
            package_assets(&app, &output)
                .unwrap_err()
                .contains("escapes app root")
        );
    }
}
