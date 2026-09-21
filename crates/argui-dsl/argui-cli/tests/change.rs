use argui_cli::ChangeTracker;

#[test]
fn changed_lines_have_unicode_locations_and_duplicate_events_do_not_recompile() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("main.argui");
    std::fs::write(&path, "Text { content: \"écho\" }\n").unwrap();
    let mut tracker = ChangeTracker::open(directory.path()).unwrap();
    assert!(tracker.refresh([path.clone()]).unwrap().is_empty());
    std::fs::write(&path, "Text { content: \"éclat\" }\n").unwrap();
    let changes = tracker.refresh([path.clone(), path.clone()]).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "main.argui");
    assert_eq!((changes[0].line, changes[0].column), (Some(1), Some(20)));
    assert_eq!(changes[0].before, "Text { content: \"écho\" }");
    assert_eq!(changes[0].after, "Text { content: \"éclat\" }");
    assert!(tracker.refresh([path]).unwrap().is_empty());
}

#[test]
fn added_removed_and_binary_assets_are_reported_without_fake_line_numbers() {
    let directory = tempfile::tempdir().unwrap();
    let mut tracker = ChangeTracker::open(directory.path()).unwrap();
    let path = directory.path().join("logo.png");
    std::fs::write(&path, [0, 255, 3]).unwrap();
    let added = tracker.refresh([path.clone()]).unwrap();
    assert_eq!(added[0].line, None);
    assert_eq!(added[0].before, "∅");
    assert!(added[0].after.starts_with("3 bytes"));
    std::fs::remove_file(&path).unwrap();
    let removed = tracker.refresh([path]).unwrap();
    assert_eq!(removed[0].after, "∅");
}

#[test]
fn added_and_removed_dsl_files_show_the_exact_first_line() {
    let directory = tempfile::tempdir().unwrap();
    let mut tracker = ChangeTracker::open(directory.path()).unwrap();
    let path = directory.path().join("new.argui");
    std::fs::write(&path, "export component New {}\n").unwrap();
    let added = tracker.refresh([path.clone()]).unwrap();
    assert_eq!((added[0].line, added[0].column), (Some(1), Some(1)));
    assert_eq!(added[0].before, "∅");
    assert_eq!(added[0].after, "export component New {}");
    std::fs::remove_file(&path).unwrap();
    let removed = tracker.refresh([path]).unwrap();
    assert_eq!(removed[0].before, "export component New {}");
    assert_eq!(removed[0].after, "∅");
}

/// Resolves relative notifications and ignores irrelevant or out-of-tree paths.
#[test]
fn relative_notifications_are_filtered_by_project_scope_and_extension() {
    let directory = tempfile::tempdir().unwrap();
    let nested = directory.path().join("src/main.argui");
    std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
    std::fs::write(&nested, "export component Main {}\n").unwrap();
    let outside = directory.path().parent().unwrap().join("outside.argui");
    let mut tracker = argui_cli::ChangeTracker::open(directory.path()).unwrap();

    std::fs::write(&nested, "export component Main { Text {} }\n").unwrap();
    let changes = tracker
        .refresh([
            std::path::PathBuf::from("src/main.argui"),
            std::path::PathBuf::from("README.txt"),
            outside,
        ])
        .unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/main.argui");
}

/// Traverses source directories while excluding generated and VCS trees.
#[test]
fn source_snapshots_skip_generated_directories_but_recurse_into_sources() {
    let directory = tempfile::tempdir().unwrap();
    for excluded in [".git", ".codex", "target", "node_modules"] {
        let path = directory.path().join(excluded).join("ignored.argui");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "export component Ignored {}\n").unwrap();
    }
    let source = directory.path().join("src/nested/main.argui");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, "export component Main {}\n").unwrap();
    std::fs::write(source.parent().unwrap().join("notes.txt"), "not DSL").unwrap();
    let mut tracker = argui_cli::ChangeTracker::open(directory.path()).unwrap();

    std::fs::write(&source, "export component Main { Text {} }\n").unwrap();
    let changes = tracker.refresh([source]).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "src/nested/main.argui");
}

/// A relevant path that cannot be read is an I/O failure, not a file removal.
#[test]
fn refresh_propagates_non_missing_read_errors() {
    let directory = tempfile::tempdir().unwrap();
    let mut tracker = ChangeTracker::open(directory.path()).unwrap();
    let unreadable = directory.path().join("directory.argui");
    std::fs::create_dir(&unreadable).unwrap();
    assert!(tracker.refresh([unreadable]).is_err());
}

/// Initial snapshots do not follow symbolic links masquerading as source files.
#[cfg(unix)]
#[test]
fn source_snapshots_skip_symbolic_link_files() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.argui");
    let link = directory.path().join("link.argui");
    std::fs::write(&source, "export component Main {}\n").unwrap();
    symlink(&source, &link).unwrap();
    let mut tracker = ChangeTracker::open(directory.path()).unwrap();
    std::fs::remove_file(&link).unwrap();
    assert!(tracker.refresh([link]).unwrap().is_empty());
}

/// Reports the first changed scalar on the correct line after a newline.
#[test]
fn multiline_edits_reset_line_and_column_locations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("main.argui");
    std::fs::write(&path, "first\nbefore\n").unwrap();
    let mut tracker = argui_cli::ChangeTracker::open(directory.path()).unwrap();
    std::fs::write(&path, "first\nafter\n").unwrap();

    let changes = tracker.refresh([path]).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!((changes[0].line, changes[0].column), (Some(2), Some(1)));
    assert_eq!(changes[0].before, "before");
    assert_eq!(changes[0].after, "after");
}
