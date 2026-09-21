use std::{
    fs,
    path::{Path, PathBuf},
};

/// Creates a complete Cargo application whose public UI is authored in Argui DSL.
///
/// * `root` — directory relative to which `target` is resolved.
/// * `target` — new project directory, absolute or root-relative.
///
/// # Errors
///
/// Returns invalid-name, already-exists, directory, or file-write errors.
pub fn new_project(root: &Path, target: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let requested = Path::new(target);
    let destination = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    if destination.exists() {
        return Err(format!("destination `{}` already exists", destination.display()).into());
    }
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("project directory must have a UTF-8 name")?;
    if !valid_package_name(name) {
        return Err(
            "project name must contain only letters, digits, `-`, or `_` and start with a letter"
                .into(),
        );
    }
    fs::create_dir_all(destination.join("src"))?;
    fs::create_dir_all(destination.join("ui"))?;
    fs::write(destination.join("Cargo.toml"), manifest(name))?;
    fs::write(destination.join("build.rs"), BUILD_SCRIPT)?;
    fs::write(destination.join("src/main.rs"), MAIN_SOURCE)?;
    fs::write(destination.join("ui/main.argui"), UI_SOURCE)?;
    fs::write(destination.join(".gitignore"), "/target\n")?;
    Ok(destination)
}

/// Validates a Cargo-compatible, filesystem-safe package name.
fn valid_package_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|value| value.is_ascii_alphabetic())
        && characters.all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
}

/// Generates the version-matched Cargo manifest for a scaffold.
fn manifest(name: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[features]
argui-live = ["argui/tasks", "dep:argui-dsl-runtime"]

[dependencies]
argui = "{version}"
argui-dsl-runtime = {{ version = "{version}", optional = true }}

[build-dependencies]
argui-dsl-build = "{version}"
"#,
        version = env!("CARGO_PKG_VERSION")
    )
}

const BUILD_SCRIPT: &str = r#"fn main() {
    argui_dsl_build::compile("ui/main.argui").expect("Argui DSL compilation failed");
}
"#;

const MAIN_SOURCE: &str = r#"use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::RendererConfig,
    runtime::run_app,
};

argui::include_ui!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "argui-live")]
    let application = argui_dsl_runtime::LiveRuntime::connect(
        std::env::var("ARGUI_DEV_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4777".into()),
        std::time::Duration::from_secs(10),
    )?;
    #[cfg(not(feature = "argui-live"))]
    let application = Main::new();
    run_app(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui application"),
            WindowConfig::default(),
        ),
        RendererConfig::default(),
        application,
        |_| {},
    )?;
    Ok(())
}
"#;

const UI_SOURCE: &str = r#"import { Button, Column, Text } from "@argui/ui"

export component Main {
    private property count: int = 0

    Column {
        gap: 12.0
        Text { content: "Welcome to Argui" }
        Button {
            text: "Increment"
            on click { count = count + 1 }
        }
    }
}
"#;
