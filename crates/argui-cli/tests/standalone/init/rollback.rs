#![cfg(test)]

use super::install;
use std::{collections::BTreeMap, fs, path::PathBuf};

#[test]
/// A staging path conflict removes every newly created temporary file.
fn staged_project_conflicts_rollback_before_creating_the_app() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("app");
    let files = BTreeMap::from([
        (PathBuf::from("nested"), b"file".to_vec()),
        (PathBuf::from("nested/child"), b"child".to_vec()),
    ]);
    assert!(install(&output, &files).is_err());
    assert!(!output.exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
/// An occupied application path remains untouched when staged placement fails.
fn existing_project_conflicts_rollback_staged_files() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("app");
    fs::create_dir_all(output.join("occupied")).unwrap();
    let files = BTreeMap::from([(PathBuf::from("occupied"), b"new file".to_vec())]);
    assert!(install(&output, &files).is_err());
    assert!(output.join("occupied").is_dir());
    assert_eq!(fs::read_dir(&output).unwrap().count(), 1);
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}
