//! Browser application scaffold and WebAssembly build lifecycle.

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

use super::project::{self, Framework};

/// Writes the browser entry point and HTML page for `name` into `app`.
/// `framework` chooses the TSX adapter in the generated application.
///
/// # Errors
/// Returns an error if a generated file cannot be written.
pub(crate) fn scaffold(app: &Path, name: &str, framework: Framework) -> Result<(), String> {
    project::write(
        &app.join("index.html"),
        &format!(
            "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{name} · Argui</title><style>html,body,#argui-root{{margin:0;width:100%;height:100%;min-height:100vh;background:#0f172a}}canvas{{display:block;width:100%;height:100%}}</style></head><body><main id=\"argui-root\"></main><script type=\"module\" src=\"/src/web.ts\"></script></body></html>\n"
        ),
    )?;
    project::write(
        &app.join("src/mount.ts"),
        "import init, { ArguiWebHost } from '../../web-host/pkg/argui_web_host.js'\nimport type { NativeBridge } from '@argui/host'\nimport { mountGallery } from './main'\n\n/** Mounts the Argui WASM renderer and TSX application in an existing element. */\nexport async function mountArgui(elementId: string): Promise<() => void> {\n  const root = document.getElementById(elementId)\n  if (!root) throw new Error(`Missing Argui mount element #${elementId}`)\n  root.addEventListener('argui:error', event => {\n    root.textContent = `Argui could not render: ${String((event as CustomEvent).detail)}`\n  })\n  await init()\n  const bridge = new ArguiWebHost(elementId)\n  const adapter: NativeBridge = {\n    contract: () => bridge.contract(),\n    commit: operations => bridge.commit(operations),\n    subscribe: callback => {\n      const unsubscribe = bridge.subscribe(callback)\n      return () => unsubscribe()\n    },\n    theme: {\n      create: definition => bridge.themeCreate(definition),\n      update: (id, patch) => bridge.themeUpdate(id, patch),\n      subscribe: (id, callback) => bridge.themeSubscribe(id, callback),\n      dispose: id => bridge.themeDispose(id),\n    },\n  }\n  return mountGallery(adapter, adapter.contract().abiHash)\n}\n",
    )?;
    project::write(
        &app.join("src/web.ts"),
        "import { mountArgui } from './mount'\n\nmountArgui('argui-root').catch(error => {\n  const root = document.getElementById('argui-root')\n  if (root) root.textContent = `Argui could not start: ${String(error)}`\n  console.error(error)\n})\n",
    )?;
    if framework == Framework::React {
        project::write(&app.join("src/react-env.d.ts"), "import 'react'\n")?;
    }
    Ok(())
}

/// Returns the Vite configuration for a browser app using `framework`.
pub(crate) fn vite_config(framework: Framework) -> String {
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
        "import {{ defineConfig }} from 'vite'\n{plugin}export default defineConfig({{ root: import.meta.dirname, {plugins} build: {{ outDir: 'dist/web', target: 'es2022' }} }})\n"
    )
}

/// Builds the browser bridge and Vite page in `cwd`; `release` enables
/// optimized Rust compilation and mandatory `wasm-opt` compression.
///
/// # Errors
/// Returns a useful error for missing tools or failed build stages.
pub(crate) fn build(cwd: &Path, release: bool) -> Result<(), String> {
    let root = project::find_root(cwd)?;
    prepare(cwd, release)?;
    status(
        Command::new("bun")
            .arg(root.join("node_modules/vite/bin/vite.js"))
            .args(["build", "--config", "vite.config.ts"])
            .current_dir(cwd),
        "vite build",
    )?;
    println!("Web app: {}", cwd.join("dist/web").display());
    Ok(())
}

/// Compiles the Rust browser host before TypeScript checking or page bundling.
/// `release` enables optimized Rust and the required `wasm-opt` pass.
///
/// # Errors
/// Returns an error for missing JavaScript/Rust tools or failed WASM output.
pub(crate) fn prepare(cwd: &Path, release: bool) -> Result<(), String> {
    project::ensure_js_dependencies(cwd)?;
    let root = project::find_root(cwd)?;
    require_tool("bun", "https://bun.sh/docs/installation")?;
    require_tool(
        "wasm-pack",
        "https://rustwasm.github.io/wasm-pack/installer/",
    )?;
    if release {
        require_tool("wasm-opt", "https://github.com/WebAssembly/binaryen")?;
    }
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|error| {
            format!("rustup is required for web builds: {error}; https://rustup.rs")
        })?;
    if !String::from_utf8_lossy(&installed.stdout).contains("wasm32-unknown-unknown") {
        return Err(
            "Rust WebAssembly target missing. Run `rustup target add wasm32-unknown-unknown`."
                .into(),
        );
    }
    let output = cwd.join("../web-host/pkg");
    let mut wasm_pack = Command::new("wasm-pack");
    wasm_pack
        .arg("build")
        .arg(root.join("apps/web-host"))
        .args(["--target", "web", "--out-dir"])
        .arg(&output)
        .arg(if release { "--release" } else { "--dev" })
        .env("CARGO_TARGET_DIR", root.join("target/argui-web"))
        .env("WASM_PACK_CACHE", root.join("target/argui-wasm-pack-cache"));
    status(&mut wasm_pack, "wasm-pack build")?;
    if release {
        let wasm = output.join("argui_web_host_bg.wasm");
        let optimized = output.join("argui_web_host_bg.opt.wasm");
        status(
            Command::new("wasm-opt")
                .args(["-Oz", "--enable-bulk-memory", "-o"])
                .arg(&optimized)
                .arg(&wasm),
            "wasm-opt",
        )?;
        fs::copy(&optimized, &wasm).map_err(|error| format!("{}: {error}", optimized.display()))?;
        fs::remove_file(&optimized).map_err(|error| format!("{}: {error}", optimized.display()))?;
    }
    Ok(())
}

/// Serves `cwd` with Vite. `release` previews the built static site; dev watches
/// Rust sources, rebuilds WASM, and lets Vite reload the page after changes.
///
/// # Errors
/// Returns an error when Bun or Vite cannot start or exits unsuccessfully.
pub(crate) fn run(cwd: &Path, release: bool) -> Result<(), String> {
    let root = project::find_root(cwd)?;
    let mut vite = Command::new("bun");
    vite.arg(root.join("node_modules/vite/bin/vite.js"));
    if release {
        vite.args(["preview", "--config", "vite.config.ts"]);
    } else {
        vite.args(["--config", "vite.config.ts"]);
    }
    if release {
        return status(vite.current_dir(cwd), "vite server");
    }
    let stopped = Arc::new(AtomicBool::new(false));
    let watching = Arc::clone(&stopped);
    let app = cwd.to_path_buf();
    let watcher = thread::spawn(move || watch_rust(&root, &app, &watching));
    let result = status(vite.current_dir(cwd), "vite server");
    stopped.store(true, Ordering::Relaxed);
    if watcher.join().is_err() {
        return Err("Argui WASM source watcher stopped unexpectedly".into());
    }
    result
}

/// Rebuilds the browser host when Rust sources or their Cargo manifests change.
/// `root` is the Argui checkout, `app` is the active web project, and `stopped`
/// signals that the Vite process has ended. A failed rebuild leaves Vite alive.
fn watch_rust(root: &Path, app: &Path, stopped: &AtomicBool) {
    let mut previous = source_snapshot(root);
    while !stopped.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
        let current = source_snapshot(root);
        if current == previous {
            continue;
        }
        previous = current;
        thread::sleep(Duration::from_millis(300));
        let stable = source_snapshot(root);
        if stable != previous {
            previous = stable;
        }
        eprintln!("Rust sources changed; rebuilding Argui WASM…");
        match prepare(app, false) {
            Ok(()) => eprintln!("Argui WASM rebuilt; Vite will reload the page."),
            Err(error) => eprintln!("Argui WASM rebuild failed: {error}"),
        }
    }
}

/// Captures Rust source and Cargo file metadata that affects the browser host.
/// The result maps absolute file paths to their modification time and size.
fn source_snapshot(root: &Path) -> BTreeMap<PathBuf, (Option<SystemTime>, u64)> {
    let mut files = BTreeMap::new();
    for path in [root.join("crates"), root.join("apps/web-host/src")] {
        visit_source(&path, &mut files);
    }
    for path in [
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("apps/web-host/Cargo.toml"),
        root.join("apps/web-host/Cargo.lock"),
    ] {
        record_source(&path, &mut files);
    }
    files
}

/// Visits source files below `path`, adding relevant metadata to `files`.
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
            Some("rs" | "toml" | "wgsl")
        ) {
            record_source(&path, files);
        }
    }
}

/// Records `path` in `files` when its metadata can be read.
fn record_source(path: &Path, files: &mut BTreeMap<PathBuf, (Option<SystemTime>, u64)>) {
    if let Ok(metadata) = fs::metadata(path) {
        files.insert(
            path.to_path_buf(),
            (metadata.modified().ok(), metadata.len()),
        );
    }
}

/// Checks whether `tool` starts and includes `url` in a missing-tool error.
///
/// # Errors
/// Returns an error when the executable is unavailable.
fn require_tool(tool: &str, url: &str) -> Result<(), String> {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|_| ())
        .map_err(|error| format!("{tool} is required for web builds: {error}. Install: {url}"))
}

/// Waits for `command` and includes `label` in any failure.
///
/// # Errors
/// Returns an error when the child cannot run or exits unsuccessfully.
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

/// Returns generated documentation for `name` and `framework`.
pub(crate) fn readme(name: &str, framework: Framework) -> String {
    format!(
        "# {name}\n\nArgui {} browser application. The Rust renderer runs in WebAssembly and draws into a canvas inside `#argui-root`.\n\nFrom the repository root, run `bun install`, then `argui dev apps/{name}` or `argui build apps/{name} release`. From this directory, omit the path. Dev rebuilds WASM after Rust changes and Vite reloads the page; TSX changes use Vite's normal reload. Open the URL printed by Vite. Release files are in `dist/web/`; serve that directory with any static HTTPS host.\n\nTo embed this app in an existing page, add `<div id=\"argui-root\"></div>` and import `mountArgui` from `src/mount.ts`, then call `await mountArgui('argui-root')`. Change the element ID to match your page. `mountArgui` returns a TSX cleanup function; mount one Argui host per page. The host bundle is built by `argui build dev` or `argui build release`.\n",
        framework.name()
    )
}

/// Solid counter application shared with the native target.
pub(crate) const SOLID_APP: &str = project::SOLID_APP;

/// React counter application shared with the native target.
pub(crate) const REACT_APP: &str = project::REACT_APP;
