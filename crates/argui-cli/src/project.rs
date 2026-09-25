//! Generated workspace projects and their build lifecycle.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Supported TSX adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    Solid,
    React,
}

/// Rendering target selected by an application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    #[default]
    Native,
    Web,
}

impl Framework {
    /// Parses `value` as a supported adapter, returning its framework.
    ///
    /// # Errors
    /// Returns an error for any other adapter.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "solid" => Ok(Self::Solid),
            "react" => Ok(Self::React),
            _ => Err("framework must be solid or react".into()),
        }
    }

    /// Returns the adapter's package suffix.
    pub fn name(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::React => "react",
        }
    }
}

/// Manifest for one workspace application.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// Application name used for the output directory and package.
    pub name: String,
    /// TSX framework used by the application.
    pub framework: Framework,
    /// Runtime and renderer target for this application.
    #[serde(default)]
    pub target: Target,
    /// Installed component names for each adapter.
    #[serde(default)]
    pub components: Vec<String>,
    /// Repository files installed as component dependencies.
    #[serde(default)]
    pub component_files: Vec<String>,
}

/// Finds the Argui checkout containing `cwd`, returning its root path.
///
/// # Errors
/// Returns an error if no ancestor contains the expected workspace packages.
pub fn find_root(cwd: &Path) -> Result<PathBuf, String> {
    cwd.ancestors()
        .find(|path| {
            path.join("packages/host/package.json").is_file()
                && path.join("apps/gallery/quickjs-host/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| "run this command inside an Argui checkout".into())
}

/// Creates a project for `target` in a local checkout or matching release clone.
/// `cwd` is the starting directory, `framework` selects the TSX adapter, and
/// `name` is the application directory. Returns an error for clone or file failures.
///
/// # Errors
/// Returns an error for invalid names, existing paths, Git failures, or writes.
pub fn init_from_target(
    cwd: &Path,
    framework: Framework,
    target: Target,
    name: &str,
) -> Result<(), String> {
    if let Ok(root) = find_root(cwd) {
        return init_target(&root, framework, target, name);
    }
    if !valid_name(name) {
        return Err("project name must use lowercase ASCII letters, digits, and hyphens, beginning with a letter".into());
    }
    let root = cwd.join(name);
    if root.exists() {
        return Err(format!("{} already exists", root.display()));
    }
    let tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--single-branch", "--branch", &tag,
            "https://github.com/ExtraBinoss/argui.git"])
        .arg(&root).status().map_err(|error| format!("Git is required to create a project outside an Argui checkout: {error}; https://git-scm.com/downloads"))?;
    if !status.success() {
        return Err(format!(
            "Git could not clone Argui {tag}: {status}. Check network access and https://github.com/ExtraBinoss/argui/releases"
        ));
    }
    let host_source = root.join("apps/gallery/quickjs-host/src/runner.rs");
    if !fs::read_to_string(&host_source).is_ok_and(|source| source.contains("ARGUI_APP_BUNDLE")) {
        return Err(format!(
            "Argui {tag} does not include the project bundle host. Install a newer Argui CLI release, or initialize inside a current Argui checkout."
        ));
    }
    init_target(&root, framework, target, name)?;
    println!("Project: {}", root.join("apps").join(name).display());
    Ok(())
}

/// Creates a project for `target` beneath `root/apps`.
/// `framework` selects the TSX adapter and `name` the directory. Returns an
/// error for unsafe names, existing paths, or failed writes.
///
/// # Errors
/// Returns an error for invalid names, conflicts, or filesystem failures.
pub fn init_target(
    root: &Path,
    framework: Framework,
    runtime: Target,
    name: &str,
) -> Result<(), String> {
    if !valid_name(name) {
        return Err("project name must use lowercase ASCII letters, digits, and hyphens, beginning with a letter".into());
    }
    let target = root.join("apps").join(name);
    if target.exists() {
        return Err(format!("{} already exists", target.display()));
    }
    fs::create_dir_all(target.join("src")).map_err(|error| error.to_string())?;
    let project = Project {
        name: name.into(),
        framework,
        target: runtime,
        components: Vec::new(),
        component_files: Vec::new(),
    };
    write(
        &target.join("argui.json"),
        &serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?,
    )?;
    write(
        &target.join("package.json"),
        &format!(
            r#"{{
  "name": "@argui/{name}", "private": true, "type": "module",
  "dependencies": {{ "@argui/host": "workspace:*", "@argui/{adapter}": "workspace:*", "@argui/widgets": "workspace:*", "{runtime}": "{version}" }}
}}
"#,
            adapter = framework.name(),
            runtime = if framework == Framework::Solid {
                "solid-js"
            } else {
                "react"
            },
            version = if framework == Framework::Solid {
                "1.9.15"
            } else {
                "19.2.0"
            }
        ),
    )?;
    write(
        &target.join("tsconfig.json"),
        &format!(
            r#"{{"extends":"../../tsconfig.json","compilerOptions":{{"jsxImportSource":"@argui/{}"}},"include":["src/**/*.ts","src/**/*.tsx"]}}"#,
            framework.name()
        ),
    )?;
    write(
        &target.join("vite.config.ts"),
        &if runtime == Target::Web {
            super::web::vite_config(framework)
        } else {
            vite_config(framework)
        },
    )?;
    write(
        &target.join("src/main.tsx"),
        match (framework, runtime) {
            (Framework::Solid, Target::Web) => super::web::SOLID_APP,
            (Framework::React, Target::Web) => super::web::REACT_APP,
            (Framework::Solid, Target::Native) => SOLID_APP,
            (Framework::React, Target::Native) => REACT_APP,
        },
    )?;
    if runtime == Target::Web {
        super::web::scaffold(&target, name, framework)?;
    }
    write(
        &target.join("README.md"),
        &if runtime == Target::Web {
            super::web::readme(name, framework)
        } else {
            format!(
                "# {name}\n\nNative Argui {} application. From this directory run `argui check` and `argui dev` to launch with TSX reload. `argui build release` copies a distributable binary, TSX bundle, and launcher to `dist/desktop/`. From the checkout root, pass `apps/{name}` to `argui dev` or `argui build`.\n",
                framework.name()
            )
        },
    )?;
    println!("Created {}", target.display());
    println!(
        "Next: run `bun install` from {}, then `cd apps/{name}` and `argui check`.",
        root.display()
    );
    Ok(())
}

/// Reports whether `name` is a safe package and directory name.
pub fn valid_name(name: &str) -> bool {
    name.len() <= 64
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Loads the application manifest in `cwd`, returning its project.
///
/// # Errors
/// Returns an error if the manifest cannot be read or parsed.
pub fn load(cwd: &Path) -> Result<Project, String> {
    let path = cwd.join("argui.json");
    let data = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let project: Project =
        serde_json::from_str(&data).map_err(|error| format!("{}: {error}", path.display()))?;
    if !valid_name(&project.name) {
        return Err(format!("{}: invalid project name", path.display()));
    }
    Ok(project)
}

/// Writes `contents` to `path`, creating parent directories as needed.
///
/// # Errors
/// Returns an error for directory creation or file writes.
pub fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, contents).map_err(|error| format!("{}: {error}", path.display()))
}

/// Checks project TSX with TypeScript, reporting readable diagnostics or JSON.
/// `cwd` is the project directory and `json` requests structured output.
///
/// # Errors
/// Returns an error when TypeScript reports diagnostics or cannot start.
pub fn check(cwd: &Path, json: bool) -> Result<(), String> {
    if load(cwd)?.target == Target::Web {
        super::web::prepare(cwd, false)?;
    }
    ensure_js_dependencies(cwd)?;
    let root = find_root(cwd)?;
    let output = Command::new("bun")
        .arg(root.join("node_modules/typescript/bin/tsc"))
        .args(["--noEmit", "--pretty", "false", "-p", "tsconfig.json"])
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("bun unavailable: {error}; https://bun.sh/docs/installation"))?;
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
        Err("TypeScript check failed".into())
    }
}

/// Builds the TSX bundle and native QuickJS host for the project in `cwd`.
/// `release` selects the optimized Cargo profile. Returns an error on failed tools.
///
/// # Errors
/// Returns an error for missing prerequisites, failed builds, or output copies.
pub fn build(cwd: &Path, release: bool) -> Result<(), String> {
    if load(cwd)?.target == Target::Web {
        return super::web::build(cwd, release);
    }
    let root = find_root(cwd)?;
    ensure_js_dependencies(cwd)?;
    status(
        Command::new("bun")
            .arg(root.join("node_modules/vite/bin/vite.js"))
            .args(["build", "--config", "vite.config.ts"])
            .current_dir(cwd),
        "vite build",
    )?;
    if !root.join("apps/gallery/dist/gallery-core.mjs").is_file() {
        command("bun", &["run", "build:gallery"], &root)?;
    }
    let manifest = root.join("apps/gallery/quickjs-host/Cargo.toml");
    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--target-dir")
        .arg(native_target(&root))
        .arg("--locked");
    if release {
        cargo.arg("--release");
    }
    status(cargo.current_dir(&root), "cargo build")?;
    if release {
        package(cwd, &root)?;
    }
    Ok(())
}

/// Checks that Bun-installed workspace packages and compiler tools are present.
/// `cwd` points to a generated application. Returns an actionable install hint.
///
/// # Errors
/// Returns an error when workspace dependencies are missing.
pub(crate) fn ensure_js_dependencies(cwd: &Path) -> Result<(), String> {
    if Command::new("bun").arg("--version").output().is_err() {
        return Err("Bun is missing. Install it from https://bun.sh/docs/installation".into());
    }
    let root = find_root(cwd)?;
    let project = load(cwd)?;
    if root.join("node_modules/typescript").exists()
        && root.join("node_modules/vite").exists()
        && cwd
            .join(format!("node_modules/@argui/{}", project.framework.name()))
            .exists()
    {
        Ok(())
    } else {
        Err(format!(
            "JavaScript workspace dependencies are missing. From {} run `bun install`, then retry. Install Bun: https://bun.sh/docs/installation",
            root.display()
        ))
    }
}

/// Returns the checkout's shared target directory for generated native applications.
pub(crate) fn native_target(root: &Path) -> PathBuf {
    root.join("target/argui-native")
}

/// Returns the target platform's native host executable name.
pub fn binary_name() -> &'static str {
    #[cfg(windows)]
    {
        "argui-gallery-quickjs.exe"
    }
    #[cfg(not(windows))]
    {
        "argui-gallery-quickjs"
    }
}

/// Runs `program` with `args` in `cwd`, returning an error on nonzero exit.
///
/// # Errors
/// Returns an error if the process cannot start or fails.
fn command(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    status(Command::new(program).args(args).current_dir(cwd), program)
}

/// Waits for `command` and reports a failure with `label`.
///
/// # Errors
/// Returns an error if the process fails to start or exits unsuccessfully.
fn status(command: &mut Command, label: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("{label}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} exited with {status}"))
    }
}

/// Copies a release binary and bundle into a desktop directory with a launcher.
///
/// # Errors
/// Returns an error if any output cannot be copied or written.
fn package(cwd: &Path, root: &Path) -> Result<(), String> {
    let project = load(cwd)?;
    let target = cwd.join("dist/desktop");
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let binary = native_target(root).join("release").join(binary_name());
    fs::copy(&binary, target.join(binary_name()))
        .map_err(|error| format!("{}: {error}", binary.display()))?;
    fs::copy(cwd.join("dist/app.mjs"), target.join("app.mjs"))
        .map_err(|error| error.to_string())?;
    for icon in ["icon-256.png", "app.ico", "app.icns"] {
        let source = cwd.join("icons").join(icon);
        if source.is_file() {
            fs::copy(&source, target.join(icon))
                .map_err(|error| format!("{}: {error}", source.display()))?;
        }
    }
    #[cfg(windows)]
    {
        write(
            &target.join("run.cmd"),
            &format!(
                "@echo off\r\nset ARGUI_APP_BUNDLE=%~dp0app.mjs\r\nset ARGUI_APP_TITLE={}\r\n\"%~dp0argui-gallery-quickjs.exe\"\r\n",
                project.name
            ),
        )?;
    }
    #[cfg(not(windows))]
    {
        let path = target.join("run.sh");
        write(
            &path,
            &format!(
                "#!/bin/sh\nset -eu\nDIR=$(CDPATH= cd \"$(dirname \"$0\")\" && pwd)\nARGUI_APP_TITLE='{}' ARGUI_APP_BUNDLE=\"$DIR/app.mjs\" exec \"$DIR/argui-gallery-quickjs\"\n",
                project.name
            ),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))
                .map_err(|error| error.to_string())?;
        }
    }
    println!("Desktop package: {}", target.display());
    Ok(())
}

/// Returns a Vite config for the selected adapter and single QuickJS module.
fn vite_config(framework: Framework) -> String {
    let plugin = if framework == Framework::Solid {
        "import solid from 'vite-plugin-solid'\n"
    } else {
        ""
    };
    let plugins = if framework == Framework::Solid {
        "plugins: [solid({ solid: { moduleName: '@argui/solid', generate: 'universal' }, hot: false })],"
    } else {
        "oxc: { jsx: { runtime: 'automatic', importSource: '@argui/react' } }, define: { 'process.env.NODE_ENV': JSON.stringify('production') },"
    };
    format!(
        "import {{ defineConfig }} from 'vite'\n{plugin}export default defineConfig({{ root: import.meta.dirname, {plugins} ssr: {{ noExternal: true, resolve: {{ conditions: ['browser'] }} }}, build: {{ ssr: 'src/main.tsx', outDir: 'dist', target: 'es2022', rollupOptions: {{ output: {{ entryFileNames: 'app.mjs' }} }} }} }})\n"
    )
}

const SOLID_APP: &str = r##"import { NativeHost, type NativeBridge, type NativeNode } from '@argui/host'
import { createSignal, render, useNativeHost } from '@argui/solid'
import { Button, palette } from '@argui/widgets/solid'

const theme = palette('dark', 'blue')

function App() {
  const [count, setCount] = createSignal(0)
  return <column width="fill" height="fill" padding={32} gap={20} background={theme.background}>
    <text text="Argui counter" color={theme.foreground} font_size={32} />
    <rectangle width="fill" height={140} radius={18} background="#334155dd" backdrop_filter="blur(12px)">
      <text text="Argui backdrop blur with a translucent fallback" color="#ffffff" font_size={18} />
    </rectangle>
    <text text={`Count: ${count()}`} color={theme.foreground} font_size={20} />
    <Button id="increment" label="Increment" theme={theme} kind="primary" onClick={() => setCount(count() + 1)} />
  </column>
}

/** Mounts the Solid application in the native QuickJS host. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  useNativeHost(host)
  const root = host.createElement('Column')
  host.setProperty(root, 'width', 'fill')
  host.setProperty(root, 'height', 'fill')
  const dispose = render(() => <App /> as NativeNode, root)
  host.setRoot(root)
  return () => { dispose(); host.dispose() }
}
"##;

const REACT_APP: &str = r##"/** @jsxImportSource @argui/react */
import { NativeHost, type NativeBridge } from '@argui/host'
import { createRoot } from '@argui/react'
import { useState } from 'react'
import { Button, palette } from '@argui/widgets/react'

const theme = palette('dark', 'blue')

function App() {
  const [count, setCount] = useState(0)
  return <column width="fill" height="fill" padding={32} gap={20} background={theme.background}>
    <text text="Argui counter" color={theme.foreground} font_size={32} />
    <rectangle width="fill" height={140} radius={18} background="#334155dd" backdrop_filter="blur(12px)">
      <text text="Argui backdrop blur with a translucent fallback" color="#ffffff" font_size={18} />
    </rectangle>
    <text text={`Count: ${count}`} color={theme.foreground} font_size={20} />
    <Button id="increment" label="Increment" theme={theme} kind="primary" onClick={() => setCount(value => value + 1)} />
  </column>
}

/** Mounts the React application in the native QuickJS host. */
export function mountGallery(bridge: NativeBridge, expectedAbiHash: string): () => void {
  const host = new NativeHost(bridge, expectedAbiHash)
  const root = createRoot(host, 'Column', { width: 'fill', height: 'fill' })
  root.render(<App />)
  return () => root.unmount()
}
"##;
