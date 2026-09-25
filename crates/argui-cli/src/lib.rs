//! Project scaffolding and build commands for Argui's native TSX host.
#![cfg(not(target_arch = "wasm32"))]

mod components;
mod icons;
mod native;
mod project;
mod web;

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

pub use components::{component_files, install};
pub use project::{Framework, Project, Target};

const HELP: &str = "Argui TSX CLI\n\nUsage:\n  argui init counter-web|counter-native\n  argui init <solid|react|web|native> [name]\n  argui init <solid|react> <name>-web\n  argui check [path] [--json]\n  argui build [path] [dev|release]\n  argui dev [path]\n  argui run [path] <dev|release>\n  argui add <solid|react> <component>...\n  argui icon <source.png>\n  argui doctor [web]\n\nAvailable components are listed in the checkout's components/registry.json.\nInside an Argui checkout, init creates apps/<name>. Elsewhere, it creates a matching versioned checkout in <name>/ with the app in <name>/apps/<name>.\nProject commands accept a path relative to the current directory; omit it when already inside the app. The project manifest selects the native or browser target.\n";
#[cfg(windows)]
const C_COMPILER: (&str, &str) = ("cl", "https://rust-lang.org/tools/install/");
#[cfg(target_os = "macos")]
const C_COMPILER: (&str, &str) = ("clang", "https://developer.apple.com/xcode/resources/");
#[cfg(all(not(windows), not(target_os = "macos")))]
const C_COMPILER: (&str, &str) = (
    "cc",
    "https://github.com/ExtraBinoss/argui/blob/main/scripts/install-linux-ci.sh",
);

/// Runs one CLI command from the current directory.
/// `args` excludes the executable name. Returns an error for invalid input,
/// missing prerequisites, or failed child commands.
///
/// # Errors
/// Returns a human-readable error for invalid arguments or failed work.
pub fn run(args: Vec<String>) -> Result<(), String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    run_in(&cwd, &args)
}

/// Runs a CLI command relative to `cwd`; useful for embedding and integration tests.
/// `args` excludes the executable name. Returns an error for invalid input or work.
///
/// # Errors
/// Returns a human-readable error for invalid arguments or failed work.
pub fn run_in(cwd: &Path, args: &[String]) -> Result<(), String> {
    match args {
        [] | [_, ..]
            if matches!(
                args.first().map(String::as_str),
                Some("--help" | "-h" | "help")
            ) =>
        {
            print!("{HELP}");
            Ok(())
        }
        [] => overview(cwd),
        [command, preset]
            if command == "init" && (preset == "counter-web" || preset == "counter-native") =>
        {
            let target = if preset == "counter-web" {
                Target::Web
            } else {
                Target::Native
            };
            project::init_from_target(cwd, Framework::Solid, target, preset)
        }
        [command, framework, name] if command == "init" => {
            let (adapter, target) = match framework.as_str() {
                "web" => (Framework::Solid, Target::Web),
                "native" => (Framework::Solid, Target::Native),
                _ => (
                    Framework::parse(framework)?,
                    if name.ends_with("-web") {
                        Target::Web
                    } else {
                        Target::Native
                    },
                ),
            };
            project::init_from_target(cwd, adapter, target, name)
        }
        [command, framework] if command == "init" => {
            let (framework, target, name) = match framework.as_str() {
                "web" => (Framework::Solid, Target::Web, "argui-web-app".to_owned()),
                "native" => (
                    Framework::Solid,
                    Target::Native,
                    "argui-native-app".to_owned(),
                ),
                _ => {
                    let framework = Framework::parse(framework)?;
                    (
                        framework,
                        Target::Native,
                        format!("argui-{}-app", framework.name()),
                    )
                }
            };
            project::init_from_target(cwd, framework, target, &name)
        }
        [command] if command == "doctor" => doctor(),
        [command, target] if command == "doctor" && target == "web" => doctor_web(),
        [command, source] if command == "icon" => icons::generate(cwd, Path::new(source)),
        [command, options @ ..] if command == "check" => check_project(cwd, options),
        [command, options @ ..] if command == "build" || command == "run" || command == "dev" => {
            project_command(cwd, command, options)
        }
        [command, framework, names @ ..] if command == "add" && !names.is_empty() => {
            components::add(cwd, Framework::parse(framework)?, names)
        }
        _ => Err(format!("invalid command; run `argui --help`\n{HELP}")),
    }
}

/// Resolves an optional app path against `cwd`, leaving an in-app command local.
/// The returned path is later validated through its `argui.json` manifest.
fn app_path(cwd: &Path, path: Option<&str>) -> PathBuf {
    path.map_or_else(|| cwd.to_path_buf(), |path| cwd.join(path))
}

/// Checks a project selected by an optional path and `--json` flag.
///
/// # Errors
/// Returns an error for invalid arguments, missing manifests, or diagnostics.
fn check_project(cwd: &Path, options: &[String]) -> Result<(), String> {
    let (path, json) = match options {
        [] => (None, false),
        [flag] if flag == "--json" => (None, true),
        [path] => (Some(path.as_str()), false),
        [path, flag] if flag == "--json" => (Some(path.as_str()), true),
        _ => return Err("usage: argui check [path] [--json]".into()),
    };
    project::check(&app_path(cwd, path), json)
}

/// Builds or runs an app selected by `options`; a path may appear before or
/// after the mode. `dev` is an alias for `run dev`.
///
/// # Errors
/// Returns an error for invalid arguments or failed build/runtime tools.
fn project_command(cwd: &Path, command: &str, options: &[String]) -> Result<(), String> {
    let (path, release) = if command == "dev" {
        match options {
            [] => (None, false),
            [path] => (Some(path.as_str()), false),
            _ => return Err("usage: argui dev [path]".into()),
        }
    } else {
        match options {
            [] if command == "build" => (None, false),
            [mode] if mode == "dev" || mode == "release" => (None, mode == "release"),
            [path] if command == "build" => (Some(path.as_str()), false),
            [path, mode] if mode == "dev" || mode == "release" => {
                (Some(path.as_str()), mode == "release")
            }
            [mode, path] if mode == "dev" || mode == "release" => {
                (Some(path.as_str()), mode == "release")
            }
            _ => return Err(format!("usage: argui {command} [path] <dev|release>")),
        }
    };
    let app = app_path(cwd, path);
    project::build(&app, release)?;
    if command != "build" {
        if project::load(&app)?.target == Target::Web {
            web::run(&app, release)?;
        } else {
            native::run(&app, release)?;
        }
    }
    Ok(())
}

/// Checks browser build tools and prints installation steps for missing ones.
///
/// # Errors
/// Returns a combined error when a required executable or Rust target is absent.
pub fn doctor_web() -> Result<(), String> {
    let mut missing = Vec::new();
    for (tool, link) in [
        ("bun", "https://bun.sh/docs/installation"),
        ("cargo", "https://rust-lang.org/tools/install/"),
        (
            "wasm-pack",
            "https://rustwasm.github.io/wasm-pack/installer/",
        ),
        ("wasm-opt", "https://github.com/WebAssembly/binaryen"),
    ] {
        if available(tool) {
            println!("✓ {tool}");
        } else {
            missing.push(format!("{tool} missing — install: {link}"));
        }
    }
    let wasm_target = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .is_ok_and(|output| {
            String::from_utf8_lossy(&output.stdout).contains("wasm32-unknown-unknown")
        });
    if wasm_target {
        println!("✓ wasm32-unknown-unknown");
    } else {
        missing.push(
            "wasm32-unknown-unknown missing — run `rustup target add wasm32-unknown-unknown`"
                .into(),
        );
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing web prerequisites:\n  {}",
            missing.join("\n  ")
        ))
    }
}

/// Prints a concise start screen for the current operating system and project.
/// `cwd` is used to show the active Argui project when present.
///
/// # Errors
/// Returns an error only if standard output cannot be written.
pub fn overview(cwd: &Path) -> Result<(), String> {
    use std::io::Write;
    let project = project::load(cwd).ok();
    let context = project.map_or_else(
        || "Argui workspace".to_owned(),
        |app| format!("{} ({} / {:?})", app.name, app.framework.name(), app.target),
    );
    let js = if available("bun") { "ready" } else { "missing" };
    let rust = if available("cargo") && available("rustc") {
        "ready"
    } else {
        "missing"
    };
    let message = format!(
        "Argui {}  ·  TSX CLI\n\nSystem   {} / {}\nProject  {context}\nTools    Bun {js}  ·  Rust {rust}\n\nGet started\n  argui init counter-web      Create a browser canvas app\n  argui init counter-native   Create a desktop app\n  argui init react my-app     Create a React desktop app\n  argui doctor                Check build prerequisites and install links\n  argui --help                All commands\n\nIf Argui is useful to you or your app, give it a star or talk with us on Discord:\n  https://github.com/ExtraBinoss/argui  ·  https://discord.gg/66rjffMmD\n",
        env!("CARGO_PKG_VERSION"),
        env::consts::OS,
        env::consts::ARCH
    );
    std::io::stdout()
        .write_all(message.as_bytes())
        .map_err(|error| error.to_string())
}

/// Returns whether `tool` can be started from the current PATH.
fn available(tool: &str) -> bool {
    Command::new(tool).arg("--version").output().is_ok()
}

/// Checks native build prerequisites and prints official installation references.
/// Returns an error when a required executable is unavailable.
///
/// # Errors
/// Returns a combined list of missing prerequisites.
pub fn doctor() -> Result<(), String> {
    let mut missing = Vec::new();
    println!("System: {} / {}", env::consts::OS, env::consts::ARCH);
    for (tool, url) in [
        ("bun", "https://bun.sh/docs/installation"),
        ("cargo", "https://rust-lang.org/tools/install/"),
        ("rustc", "https://rust-lang.org/tools/install/"),
        ("git", "https://git-scm.com/downloads"),
    ] {
        if available(tool) {
            println!("✓ {tool}");
        } else {
            missing.push(format!("{tool} missing — install: {url}"));
        }
    }
    if available("node") {
        println!("✓ node (optional; Bun handles project builds)");
    }
    println!(
        "QuickJS is bundled. Its Rust bindings also need the libclang shared library: https://rust-lang.github.io/rust-bindgen/requirements.html"
    );
    if available("clang") {
        println!("✓ clang executable (verify libclang is installed if bindgen fails)");
    } else {
        missing.push("clang missing — install LLVM/Clang and libclang: https://rust-lang.github.io/rust-bindgen/requirements.html".into());
    }
    let (compiler, link) = C_COMPILER;
    if available(compiler) {
        println!("✓ C compiler ({compiler}); needed for embedded QuickJS");
    } else {
        missing.push(format!("C compiler ({compiler}) missing — install: {link}"));
    }
    #[cfg(target_os = "linux")]
    {
        if available("pkg-config") {
            println!("✓ pkg-config");
        } else {
            missing.push("pkg-config missing — install: https://github.com/ExtraBinoss/argui/blob/main/scripts/install-linux-ci.sh".into());
        }
        for library in ["wayland-client", "xkbcommon", "egl"] {
            if Command::new("pkg-config")
                .args(["--exists", library])
                .status()
                .is_ok_and(|status| status.success())
            {
                println!("✓ {library}");
            } else {
                missing.push(format!("{library} development library missing — see https://github.com/ExtraBinoss/argui/blob/main/scripts/install-linux-ci.sh"));
            }
        }
        println!("Optional WebView: GTK 3 + WebKitGTK 4.1 (https://github.com/tauri-apps/wry)");
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing prerequisites:\n  {}",
            missing.join("\n  ")
        ))
    }
}
