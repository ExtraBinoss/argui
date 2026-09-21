use std::{fs, path::Path};

use argui_cli::{ProjectFiles, canonical_relative};

#[test]
fn project_snapshot_is_sorted_and_excludes_generated_trees() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("ui/nested")).unwrap();
    for directory in [".git", ".codex", "target", "node_modules"] {
        fs::create_dir_all(root.path().join(directory)).unwrap();
        fs::write(
            root.path().join(directory).join("ignored.argui"),
            "export component Ignored {}",
        )
        .unwrap();
    }
    fs::write(root.path().join("z.argui"), "export component Zed {}").unwrap();
    fs::write(
        root.path().join("ui/nested/a.argui"),
        "export component Alpha {}",
    )
    .unwrap();
    fs::write(root.path().join("README.md"), "not DSL").unwrap();

    let project = ProjectFiles::read(root.path()).unwrap();
    assert_eq!(
        project
            .modules
            .iter()
            .map(|module| module.path.as_str())
            .collect::<Vec<_>>(),
        ["ui/nested/a.argui", "z.argui"]
    );
    let mut database = project.semantic_database().unwrap();
    let checked = database.check();
    assert!(
        checked
            .modules
            .iter()
            .any(|module| module.path == "z.argui")
    );
}

#[test]
fn canonical_relative_normalizes_curdir_and_rejects_escape() {
    let root = Path::new("/tmp/argui-project");
    assert_eq!(
        canonical_relative(root, Path::new("/tmp/argui-project/./ui/main.argui")).unwrap(),
        "ui/main.argui"
    );
    let outside = canonical_relative(root, Path::new("/tmp/other/main.argui")).unwrap_err();
    assert_eq!(outside.kind(), std::io::ErrorKind::InvalidInput);
    let parent =
        canonical_relative(root, Path::new("/tmp/argui-project/../main.argui")).unwrap_err();
    assert_eq!(parent.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn project_read_surfaces_missing_roots_and_non_utf8_sources() {
    let missing = ProjectFiles::read("/tmp/argui-no-such-root").unwrap_err();
    assert_eq!(missing.kind(), std::io::ErrorKind::NotFound);

    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("invalid.argui"), [0xff, 0xfe]).unwrap();
    let error = ProjectFiles::read(root.path()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[cfg(unix)]
#[test]
fn canonical_relative_reports_non_utf8_component() {
    use std::os::unix::ffi::OsStrExt;

    let root = Path::new("/tmp/argui-project");
    let invalid = root.join(std::ffi::OsStr::from_bytes(b"bad\xff.argui"));
    let error = canonical_relative(root, &invalid).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}
