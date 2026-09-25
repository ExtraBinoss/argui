//! Packs the CLI-owned SDK snapshot into the installable binary.

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Packs every file under `assets` in stable path order for the generated app.
///
/// # Panics
/// Panics when an owned asset cannot be read or the bundle cannot be written.
fn main() {
    println!("cargo:rerun-if-changed=assets");
    let mut files = Vec::new();
    collect(Path::new("assets"), &mut files);
    files.sort();
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut bundle = fs::File::create(output.join("sdk.bundle")).expect("create SDK bundle");
    for path in files {
        let name = path
            .strip_prefix("assets")
            .expect("asset prefix")
            .to_string_lossy();
        let name = name.replace('\\', "/");
        let body = fs::read(&path).expect("read SDK asset");
        bundle
            .write_all(&(name.len() as u32).to_le_bytes())
            .expect("write name length");
        bundle.write_all(name.as_bytes()).expect("write name");
        bundle
            .write_all(&(body.len() as u64).to_le_bytes())
            .expect("write body length");
        bundle.write_all(&body).expect("write body");
    }
}

/// Adds every regular file under `directory` to `files`.
///
/// # Panics
/// Panics when a directory cannot be traversed.
fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read SDK asset directory") {
        let path = entry.expect("SDK asset entry").path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}
