//! Builds and launches an application entirely from its own directory.

use super::Manifest;
use crate::sdk;
use std::{fs, path::Path, process::Command};

/// Checks the generated Rust or TSX sources without using an Argui checkout.
///
/// # Errors
/// Returns compiler diagnostics or a missing dependency hint.
pub(super) fn check(directory: &Path, manifest: &Manifest, json: bool) -> Result<(), String> {
    if manifest.framework == "rust" {
        let output = Command::new("cargo")
            .args(["check", "--manifest-path"])
            .arg(directory.join("Cargo.toml"))
            .output()
            .map_err(|error| format!("cargo check: {error}"))?;
        return report(output, json, "Rust check failed");
    }
    js_dependencies(directory, manifest)?;
    generate_assets(directory)?;
    if manifest.targets.iter().any(|target| target == "web") {
        web_host(directory, manifest, false)?;
    }
    let output = Command::new("bun")
        .arg(directory.join("node_modules/typescript/bin/tsc"))
        .args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"])
        .current_dir(directory)
        .output()
        .map_err(|error| format!("bun unavailable: {error}"))?;
    report(output, json, "TypeScript check failed")
}

/// Reports child compiler output as text or one diagnostics JSON object.
///
/// # Errors
/// Returns `failure` when the compiler exits unsuccessfully.
fn report(output: std::process::Output, json: bool, failure: &str) -> Result<(), String> {
    let diagnostics = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    if json {
        println!(
            "{}",
            serde_json::json!({"ok": output.status.success(), "diagnostics": diagnostics.lines().collect::<Vec<_>>() })
        );
    } else if !diagnostics.trim().is_empty() {
        print!("{diagnostics}");
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(failure.into())
    }
}

/// Builds every selected target or one selected output.
///
/// # Errors
/// Returns an error for an absent target, missing tool, or failed compiler.
pub(super) fn build(
    directory: &Path,
    manifest: &Manifest,
    release: bool,
    target: Option<&str>,
) -> Result<(), String> {
    let targets = if let Some(target) = target {
        if !manifest.targets.iter().any(|selected| selected == target) {
            return Err(format!(
                "target `{target}` is not selected; available: {}",
                manifest.targets.join(", ")
            ));
        }
        vec![target]
    } else {
        manifest.targets.iter().map(String::as_str).collect()
    };
    for target in targets {
        if target == "native" {
            native(directory, manifest, release, false)?;
        } else if target == "web" {
            web(directory, manifest, release)?;
        }
    }
    Ok(())
}

/// Compiles the native Rust app or TSX host, and packages release output.
///
/// # Errors
/// Returns an error for missing JS dependencies, Cargo failure, or copying.
pub(super) fn native(
    directory: &Path,
    manifest: &Manifest,
    release: bool,
    automation: bool,
) -> Result<(), String> {
    let rust = manifest.framework == "rust";
    if !rust {
        js_dependencies(directory, manifest)?;
        generate_assets(directory)?;
        status(
            Command::new("bun")
                .arg(directory.join("node_modules/vite/bin/vite.js"))
                .args(["build", "--mode", "native", "--config", "vite.config.ts"])
                .env("ARGUI_APP_RELEASE", if release { "1" } else { "0" })
                .current_dir(directory),
            "vite native build",
        )?;
    }
    let cargo_manifest = if rust {
        directory.join("Cargo.toml")
    } else {
        sdk::materialize()?.join("hosts/native/Cargo.toml")
    };
    let mut command = Command::new("cargo");
    command
        .args(["build", "--manifest-path"])
        .arg(&cargo_manifest)
        .args(["--target-dir"])
        .arg(target_dir(directory, "native"));
    if release {
        command.arg("--release");
    }
    if !rust {
        let mut features = Vec::new();
        if manifest.features.iter().any(|feature| feature == "tasks") {
            features.push("tasks");
        }
        if automation {
            features.push("automation");
        }
        if !features.is_empty() {
            command.args(["--features", &features.join(",")]);
        }
    }
    status(command.current_dir(directory), "cargo native build")?;
    if release {
        package_native(directory, manifest)?;
    }
    Ok(())
}

/// Compiles the app-owned WebAssembly host and browser bundle.
///
/// # Errors
/// Returns an error for missing WASM tools, Cargo failure, or Vite failure.
fn web(directory: &Path, manifest: &Manifest, release: bool) -> Result<(), String> {
    js_dependencies(directory, manifest)?;
    generate_assets(directory)?;
    web_host(directory, manifest, release)?;
    status(
        Command::new("bun")
            .arg(directory.join("node_modules/vite/bin/vite.js"))
            .args(["build", "--mode", "web", "--config", "vite.config.ts"])
            .env("ARGUI_APP_RELEASE", if release { "1" } else { "0" })
            .current_dir(directory),
        "vite web build",
    )?;
    if release {
        package_assets(directory, &directory.join("dist/web"))?;
    }
    println!("Web app: {}", directory.join("dist/web").display());
    Ok(())
}

/// Compiles the app-owned WASM crate into its local `pkg` directory.
///
/// # Errors
/// Returns a prerequisite or wasm-pack error.
fn web_host(directory: &Path, manifest: &Manifest, release: bool) -> Result<(), String> {
    if Command::new("wasm-pack").arg("--version").output().is_err() {
        return Err("wasm-pack is required for Web; install it from https://rustwasm.github.io/wasm-pack/installer/".into());
    }
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|error| format!("rustup target list: {error}"))?;
    if !String::from_utf8_lossy(&installed.stdout).contains("wasm32-unknown-unknown") {
        return Err("Web requires `rustup target add wasm32-unknown-unknown`".into());
    }
    let cache = sdk::materialize()?;
    let mut command = Command::new("wasm-pack");
    command
        .arg("build")
        .arg(cache.join("hosts/web"))
        .args(["--target", "web", "--out-dir"])
        .arg(cache.join("hosts/web/pkg"))
        .arg(if release { "--release" } else { "--dev" })
        .env("CARGO_TARGET_DIR", target_dir(directory, "web"))
        .env("WASM_PACK_CACHE", target_dir(directory, "wasm-pack-cache"));
    if manifest.features.iter().any(|feature| feature == "tasks") {
        command.args(["--", "--features", "tasks"]);
    }
    status(&mut command, "wasm-pack build")?;
    let package = directory.join("node_modules/@argui/web-host");
    fs::create_dir_all(&package).map_err(|error| format!("{}: {error}", package.display()))?;
    let pkg = cache.join("hosts/web/pkg");
    for entry in fs::read_dir(&pkg).map_err(|error| format!("{}: {error}", pkg.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.path().is_file() {
            let destination = package.join(entry.file_name());
            fs::copy(entry.path(), &destination)
                .map_err(|error| format!("{}: {error}", destination.display()))?;
        }
    }
    let package_manifest = package.join("package.json");
    fs::write(
        &package_manifest,
        "{\"name\":\"@argui/web-host\",\"type\":\"module\"}\n",
    )
    .map_err(|error| format!("{}: {error}", package_manifest.display()))?;
    Ok(())
}

/// Confirms local Bun packages were installed in the application directory.
///
/// # Errors
/// Returns a `bun install` hint when a required package is absent.
fn js_dependencies(directory: &Path, manifest: &Manifest) -> Result<(), String> {
    if manifest.sdk_sha256 != sdk::digest() {
        return Err(
            "application SDK hash differs from this CLI; install the matching Argui CLI".into(),
        );
    }
    if Command::new("bun").arg("--version").output().is_err() {
        return Err("Bun is missing; install it from https://bun.sh/docs/installation".into());
    }
    for path in ["node_modules/typescript", "node_modules/vite"] {
        if !directory.join(path).exists() {
            return Err(format!(
                "JavaScript dependencies are missing in {}; run `bun install` there",
                directory.display()
            ));
        }
    }
    let cache = sdk::materialize()?;
    let scope = directory.join("node_modules/@argui");
    fs::create_dir_all(&scope).map_err(|error| format!("{}: {error}", scope.display()))?;
    for adapter in ["host", "solid", "react", "test"] {
        let source = cache.join("sdk").join(adapter);
        let link = scope.join(adapter);
        if link.exists() {
            if link.canonicalize().ok().as_deref() != source.canonicalize().ok().as_deref() {
                return Err(format!(
                    "{} conflicts with the CLI SDK snapshot",
                    link.display()
                ));
            }
            continue;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&source, &link)
            .map_err(|error| format!("{}: {error}", link.display()))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&source, &link).map_err(|error| {
            format!(
                "{}: {error}; enable Developer Mode for SDK cache links",
                link.display()
            )
        })?;
    }
    Ok(())
}

/// Generates app-owned media references and release/debug manifests before TSX compilation.
///
/// `directory` is the application root containing its assets and source entries.
///
/// # Errors
/// Returns a generator diagnostic for unsafe sources, missing references, or dynamic asset keys.
fn generate_assets(directory: &Path) -> Result<(), String> {
    status(
        Command::new("bun")
            .arg("scripts/generate-assets.mjs")
            .arg("assets.config.json")
            .current_dir(directory),
        "app asset generation",
    )
}

/// Copies a release executable, bundle, and launcher to `dist/desktop`.
///
/// # Errors
/// Returns an error for a missing build product or output write failure.
fn package_native(directory: &Path, manifest: &Manifest) -> Result<(), String> {
    let output = directory.join("dist/desktop");
    fs::create_dir_all(&output).map_err(|error| error.to_string())?;
    let binary = binary_name(manifest);
    let source = target_dir(directory, "native")
        .join("release")
        .join(&binary);
    fs::copy(&source, output.join(&binary))
        .map_err(|error| format!("{}: {error}", source.display()))?;
    if manifest.framework != "rust" {
        let bundle = directory.join("dist/native/app.mjs");
        fs::copy(&bundle, output.join("app.mjs"))
            .map_err(|error| format!("{}: {error}", bundle.display()))?;
        package_assets(directory, &output)?;
    }
    #[cfg(windows)]
    {
        let launcher = if manifest.framework == "rust" {
            format!("@echo off\r\n\"%~dp0{binary}\"\r\n")
        } else {
            format!(
                "@echo off\r\nset ARGUI_APP_BUNDLE=%~dp0app.mjs\r\nset ARGUI_APP_ASSETS=%~dp0assets.generated.json\r\nset ARGUI_APP_TITLE={}\r\n\"%~dp0{binary}\"\r\n",
                manifest.name
            )
        };
        fs::write(output.join("run.cmd"), launcher).map_err(|error| error.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let prefix = if manifest.framework == "rust" {
            String::new()
        } else {
            format!(
                "ARGUI_APP_TITLE='{}' ARGUI_APP_BUNDLE=\"$DIR/app.mjs\" ARGUI_APP_ASSETS=\"$DIR/assets.generated.json\" ",
                manifest.name
            )
        };
        let launcher = format!(
            "#!/bin/sh\nset -eu\nDIR=$(CDPATH= cd \"$(dirname \"$0\")\" && pwd)\n{prefix}exec \"$DIR/{binary}\"\n"
        );
        let path = output.join("run.sh");
        fs::write(&path, launcher).map_err(|error| format!("{}: {error}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                .map_err(|error| error.to_string())?;
        }
    }
    println!("Desktop package: {}", output.display());
    Ok(())
}

/// Copies only the media files named by the generated release manifest.
///
/// `directory` is the app root and `output` is its portable desktop package.
///
/// # Errors
/// Returns an error if the manifest is invalid, a source escapes `assets/`, or copying fails.
fn package_assets(directory: &Path, output: &Path) -> Result<(), String> {
    let manifest = directory.join("assets.generated.json");
    let document: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest).map_err(|error| format!("{}: {error}", manifest.display()))?,
    )
    .map_err(|error| format!("{}: {error}", manifest.display()))?;
    let entries = document
        .get("assets")
        .and_then(serde_json::Value::as_array)
        .ok_or("asset manifest must contain an assets array")?;
    let source_root =
        fs::canonicalize(directory.join("assets")).map_err(|error| error.to_string())?;
    let asset_output = output.join("assets");
    if asset_output.exists() {
        fs::remove_dir_all(&asset_output)
            .map_err(|error| format!("{}: {error}", asset_output.display()))?;
    }
    fs::create_dir_all(&asset_output)
        .map_err(|error| format!("{}: {error}", asset_output.display()))?;
    for entry in entries {
        let relative = entry
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or("asset manifest entry lacks path")?;
        let path = Path::new(relative);
        if !path
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            return Err(format!("unsafe release asset path: {relative}"));
        }
        let source = fs::canonicalize(source_root.join(path))
            .map_err(|error| format!("{relative}: {error}"))?;
        if !source.starts_with(&source_root) {
            return Err(format!("release asset escapes app root: {relative}"));
        }
        let destination = asset_output.join(path);
        fs::create_dir_all(destination.parent().ok_or("asset output has no parent")?)
            .map_err(|error| error.to_string())?;
        fs::copy(&source, &destination)
            .map_err(|error| format!("{}: {error}", destination.display()))?;
    }
    fs::copy(&manifest, output.join("assets.generated.json"))
        .map_err(|error| format!("{}: {error}", manifest.display()))?;
    Ok(())
}

/// Returns the host binary name for this framework and operating system.
fn binary_name(manifest: &Manifest) -> String {
    let stem = if manifest.framework == "rust" {
        manifest.name.as_str()
    } else {
        "argui-app-native"
    };
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.into()
    }
}

/// Starts a selected built target from its own application directory.
///
/// # Errors
/// Returns an error for an absent output or failed child process.
pub(super) fn run(
    directory: &Path,
    manifest: &Manifest,
    release: bool,
    target: &str,
) -> Result<(), String> {
    if target == "web" {
        let mut command = Command::new("bun");
        command.arg(directory.join("node_modules/vite/bin/vite.js"));
        if release {
            command.arg("preview");
        } else {
            command.args(["--mode", "web"]);
        }
        return status(
            command
                .args(["--config", "vite.config.ts"])
                .current_dir(directory),
            "vite server",
        );
    }
    let binary = target_dir(directory, "native")
        .join(if release { "release" } else { "debug" })
        .join(binary_name(manifest));
    if !binary.is_file() {
        return Err(format!("native executable missing: {}", binary.display()));
    }
    let mut command = Command::new(binary);
    if manifest.framework != "rust" {
        command
            .env("ARGUI_APP_BUNDLE", directory.join("dist/native/app.mjs"))
            .env(
                "ARGUI_APP_ASSETS",
                directory.join(if release {
                    "assets.generated.json"
                } else {
                    "assets.dev.generated.json"
                }),
            )
            .env("ARGUI_APP_TITLE", &manifest.name);
    }
    status(command.current_dir(directory), "native app")
}

/// Runs one child process and names any process failure.
///
/// # Errors
/// Returns an error when launch fails or exit status is unsuccessful.
fn status(command: &mut Command, label: &str) -> Result<(), String> {
    let result = command
        .status()
        .map_err(|error| format!("{label}: {error}"))?;
    if result.success() {
        Ok(())
    } else {
        Err(format!("{label} exited with {result}"))
    }
}

/// Returns one target-specific Cargo directory, honoring the caller's target root.
fn target_dir(directory: &Path, target: &str) -> std::path::PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| directory.join("target"))
        .join(target)
}

/// Returns the debug native host produced by the automation build.
pub(super) fn automation_binary_path(directory: &Path, manifest: &Manifest) -> std::path::PathBuf {
    target_dir(directory, "native")
        .join("debug")
        .join(binary_name(manifest))
}
