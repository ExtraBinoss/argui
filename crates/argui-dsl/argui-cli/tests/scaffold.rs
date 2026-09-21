use std::fs;

use argui_cli::new_project;

#[test]
fn scaffold_supports_absolute_destinations_and_writes_all_runtime_inputs() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("nested/absolute-app");
    let created = new_project(root.path(), destination.to_str().unwrap()).unwrap();
    assert_eq!(created, destination);
    for path in [
        "Cargo.toml",
        "build.rs",
        "src/main.rs",
        "ui/main.argui",
        ".gitignore",
    ] {
        assert!(
            destination.join(path).is_file(),
            "missing scaffold file {path}"
        );
    }
    let manifest = fs::read_to_string(destination.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("name = \"absolute-app\""));
    assert!(manifest.contains("argui-live"));
}

#[test]
fn scaffold_rejects_invalid_names_and_unwritable_parent_paths() {
    let root = tempfile::tempdir().unwrap();
    for name in ["", "123-app", "_private", "has space", "bad.dot"] {
        let error = new_project(root.path(), name).unwrap_err();
        assert!(
            error.to_string().contains("project name")
                || error.to_string().contains("already exists"),
            "unexpected error for {name:?}: {error}"
        );
    }

    fs::write(root.path().join("occupied"), "file").unwrap();
    let error = new_project(root.path(), "occupied/app").unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn scaffold_reports_existing_destinations_before_writing() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("existing")).unwrap();
    let error = new_project(root.path(), "existing").unwrap_err();
    assert!(error.to_string().contains("already exists"));
}
