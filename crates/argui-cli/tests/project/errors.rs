#![cfg(test)]

use crate::project;
use std::fs;

#[test]
/// Project writes report blocked parents and occupied output paths.
fn generated_project_writes_report_filesystem_conflicts() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_parent = directory.path().join("occupied");
    fs::write(&blocked_parent, "private").unwrap();
    assert!(project::write(&blocked_parent.join("child"), "new").is_err());
    assert_eq!(fs::read_to_string(&blocked_parent).unwrap(), "private");

    let blocked_output = directory.path().join("folder");
    fs::create_dir(&blocked_output).unwrap();
    assert!(project::write(&blocked_output, "new").is_err());
    assert!(blocked_output.is_dir());
}
