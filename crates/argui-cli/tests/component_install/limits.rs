#![cfg(test)]

use crate::component_install::{MAX_FILE, read_bounded};
use std::fs;

#[test]
/// Installed widget files must be readable UTF-8 within the fixed size limit.
fn local_component_source_rejects_missing_directory_binary_and_large_files() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("widget.tsx");
    assert!(read_bounded(&file).is_err());
    assert!(read_bounded(directory.path()).is_err());

    fs::write(&file, b"\xff").unwrap();
    assert!(read_bounded(&file).unwrap_err().contains("invalid UTF-8"));

    fs::write(&file, vec![b'x'; MAX_FILE as usize + 1]).unwrap();
    assert!(read_bounded(&file).unwrap_err().contains("exceeds"));
    fs::write(&file, vec![b'x'; MAX_FILE as usize]).unwrap();
    assert_eq!(read_bounded(&file).unwrap().len(), MAX_FILE as usize);
}
