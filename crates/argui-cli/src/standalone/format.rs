//! Formats source files in an initialized Solid or React application.

use super::Manifest;
use std::{path::Path, process::Command};

/// Runs the application's pinned Oxfmt on its TSX sources and Vite config.
/// `directory` is the selected application root, `manifest` determines whether
/// the app supports TSX, and `check` reports differences without writing files.
///
/// # Errors
/// Returns an error for a Rust-only app, missing Bun dependencies, or Oxfmt
/// failure.
pub(super) fn run(directory: &Path, manifest: &Manifest, check: bool) -> Result<(), String> {
    if manifest.framework == "rust" {
        return Err("argui format requires a Solid or React application".into());
    }
    let formatter = directory.join("node_modules/oxfmt/bin/oxfmt");
    if !formatter.is_file() {
        return Err(format!(
            "Oxfmt is missing in {}; run `bun install` there",
            directory.display()
        ));
    }
    let mut command = Command::new("bun");
    command.arg(formatter).args(["--config", ".oxfmtrc.json"]);
    if check {
        command.arg("--check");
    }
    let output = command
        .args(["src", "vite.config.ts"])
        .current_dir(directory)
        .output()
        .map_err(|error| format!("bun unavailable: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    if output.status.success() {
        Ok(())
    } else if check {
        Err("format check failed".into())
    } else {
        Err("format failed".into())
    }
}
