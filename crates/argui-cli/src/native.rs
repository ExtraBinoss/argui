//! Native application launch and source rebuilds for development.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime},
};

use super::project;

/// Launches the built native app in `cwd`. Development builds watch TSX sources
/// and rebuild the bundle; `release` launches the optimized package without a
/// watcher. Returns when the native host exits.
///
/// # Errors
/// Returns an error for a missing bundle or executable, or a failed host.
pub(crate) fn run(cwd: &Path, release: bool) -> Result<(), String> {
    let root = project::find_root(cwd)?;
    let project = project::load(cwd)?;
    let executable = project::native_target(&root)
        .join(if release { "release" } else { "debug" })
        .join(project::binary_name());
    let bundle = cwd.join("dist/app.mjs");
    if !executable.is_file() {
        return Err(format!("native host missing: {}", executable.display()));
    }
    if !bundle.is_file() {
        return Err(format!("TSX bundle missing: {}", bundle.display()));
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let watcher = if release {
        None
    } else {
        let app = cwd.to_path_buf();
        let signal = Arc::clone(&stopped);
        Some(thread::spawn(move || watch_tsx(&root, &app, &signal)))
    };
    let result = Command::new(executable)
        .env("ARGUI_APP_BUNDLE", bundle)
        .env("ARGUI_APP_TITLE", project.name)
        .current_dir(cwd)
        .status()
        .map_err(|error| format!("native host: {error}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("native host exited with {status}"))
            }
        });
    stopped.store(true, Ordering::Relaxed);
    if watcher.is_some_and(|watcher| watcher.join().is_err()) {
        return Err("native TSX source watcher stopped unexpectedly".into());
    }
    result
}

/// Rebuilds the app bundle after TSX or configuration changes while `stopped`
/// remains false. `root` contains Vite and `app` is the selected project.
fn watch_tsx(root: &Path, app: &Path, stopped: &AtomicBool) {
    let mut previous = source_snapshot(app);
    while !stopped.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
        let current = source_snapshot(app);
        if current == previous {
            continue;
        }
        previous = current;
        thread::sleep(Duration::from_millis(300));
        let stable = source_snapshot(app);
        if stable != previous {
            previous = stable;
        }
        eprintln!("TSX sources changed; rebuilding Argui native bundle…");
        let result = Command::new("bun")
            .arg(root.join("node_modules/vite/bin/vite.js"))
            .args(["build", "--config", "vite.config.ts"])
            .current_dir(app)
            .status();
        match result {
            Ok(status) if status.success() => {
                eprintln!("Native bundle rebuilt; Argui host will reload it.")
            }
            Ok(status) => {
                eprintln!("Native bundle rebuild exited with {status}; keeping the previous scene.")
            }
            Err(error) => {
                eprintln!("Native bundle rebuild failed: {error}; keeping the previous scene.")
            }
        }
    }
}

/// Captures relevant TSX and configuration file metadata beneath `app`.
/// The returned map contains each file's modification time and size.
fn source_snapshot(app: &Path) -> BTreeMap<PathBuf, (Option<SystemTime>, u64)> {
    let mut files = BTreeMap::new();
    visit_source(&app.join("src"), &mut files);
    for path in ["package.json", "tsconfig.json", "vite.config.ts"] {
        record_source(&app.join(path), &mut files);
    }
    files
}

/// Recursively records TS, TSX, and CSS sources beneath `path` into `files`.
fn visit_source(path: &Path, files: &mut BTreeMap<PathBuf, (Option<SystemTime>, u64)>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_source(&path, files);
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("ts" | "tsx" | "css")
        ) {
            record_source(&path, files);
        }
    }
}

/// Adds readable metadata for `path` to the source snapshot `files`.
fn record_source(path: &Path, files: &mut BTreeMap<PathBuf, (Option<SystemTime>, u64)>) {
    if let Ok(metadata) = fs::metadata(path) {
        files.insert(
            path.to_path_buf(),
            (metadata.modified().ok(), metadata.len()),
        );
    }
}
