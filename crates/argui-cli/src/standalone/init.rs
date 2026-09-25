//! Transactional creation of one self-contained application directory.

use super::Manifest;
use crate::sdk;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Validated inputs for one init operation.
pub(super) struct Options {
    /// Final application directory.
    pub directory: PathBuf,
    /// Package and display name.
    pub name: String,
    /// Rust, Solid, or React source type.
    pub framework: String,
    /// Native and/or Web output targets.
    pub targets: Vec<String>,
    /// Supported optional build features.
    pub features: Vec<String>,
}

/// Creates the complete application or removes every newly written file.
///
/// # Errors
/// Returns an error before mutation for invalid choices and path conflicts, or
/// after rolling back filesystem writes when staging or installation fails.
pub(super) fn create(options: &Options) -> Result<(), String> {
    validate(options)?;
    let mut files = source_files(options)?;
    let manifest = Manifest {
        project_version: 2,
        name: options.name.clone(),
        framework: options.framework.clone(),
        targets: options.targets.clone(),
        features: options.features.clone(),
        argui_version: env!("CARGO_PKG_VERSION").into(),
        sdk_sha256: sdk::digest(),
        distribution: if sdk::release_available() {
            "release"
        } else {
            "embedded-development"
        }
        .into(),
        components: Vec::new(),
        component_files: Vec::new(),
        component_checksums: BTreeMap::new(),
        component_versions: BTreeMap::new(),
    };
    files.insert(
        "argui.json".into(),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?
        )
        .into_bytes(),
    );
    preflight(&options.directory, &files)?;
    install(&options.directory, &files)?;
    println!("Created {}", options.directory.display());
    if options.framework == "rust" {
        println!(
            "Next: cd {} && argui check && argui build release",
            options.directory.display()
        );
    } else {
        println!(
            "Next: cd {} && bun install && argui check && argui build release",
            options.directory.display()
        );
    }
    Ok(())
}

/// Validates framework, targets, and selectable capabilities.
///
/// # Errors
/// Returns an error naming an unsupported or contradictory choice.
fn validate(options: &Options) -> Result<(), String> {
    if !crate::project::valid_name(&options.name) {
        return Err("project name must use lowercase ASCII letters, digits, and hyphens, beginning with a letter; use --name".into());
    }
    if !matches!(options.framework.as_str(), "rust" | "solid" | "react") {
        return Err("framework must be rust, solid, or react".into());
    }
    let targets = options
        .targets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if targets.len() != options.targets.len() || targets.is_empty() {
        return Err("--targets needs each target once; choose native, web, or native,web".into());
    }
    if targets.contains("mobile") {
        return Err("Mobile means Android and iOS together; iOS app shell and packaging are unavailable, so use --targets native,web or native".into());
    }
    if targets
        .iter()
        .any(|target| !matches!(*target, "native" | "web"))
    {
        return Err("--targets accepts native, web, or native,web".into());
    }
    if options.framework == "rust" && targets.contains("web") {
        return Err(
            "the Rust scaffold currently supports native only; use --targets native".into(),
        );
    }
    let features = options
        .features
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if features.len() != options.features.len() {
        return Err("--feature values may appear only once".into());
    }
    for feature in features {
        match feature {
            "tasks" => {}
            "automation" if options.framework != "rust" && targets.contains("native") => {}
            "automation" => {
                return Err(
                    "--feature automation needs a Solid or React app with native target".into(),
                );
            }
            _ => {
                return Err(format!(
                    "--feature {feature} is not wired to the generated builds; selectable: tasks, automation (native TSX)"
                ));
            }
        }
    }
    Ok(())
}

/// Builds the app's file map, including only selected host targets.
///
/// # Errors
/// Returns an error for a missing or malformed CLI-owned SDK asset.
fn source_files(options: &Options) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    let web = options.targets.iter().any(|target| target == "web");
    if options.framework == "rust" {
        files.insert("Cargo.toml".into(), rust_manifest(options).into_bytes());
        files.insert("src/main.rs".into(), rust_main().as_bytes().to_vec());
    } else {
        for asset in sdk::assets()? {
            if asset.path == format!("templates/{}-main.tsx", options.framework) {
                files.insert("src/main.tsx".into(), asset.body.to_vec());
            }
            if web && asset.path == "templates/mount.ts" {
                files.insert("src/mount.ts".into(), asset.body.to_vec());
            }
            if web && asset.path == "templates/web.ts" {
                files.insert("src/web.ts".into(), asset.body.to_vec());
            }
        }
        if !files.contains_key(Path::new("src/main.tsx")) {
            return Err("CLI SDK snapshot lacks the selected TSX template".into());
        }
        files.insert("package.json".into(), js_manifest(options).into_bytes());
        files.insert("tsconfig.json".into(), tsconfig(options).into_bytes());
        files.insert("vite.config.ts".into(), vite_config(options).into_bytes());
        if web {
            files.insert("index.html".into(), html(&options.name).into_bytes());
        }
    }
    files.insert("README.md".into(), readme(options).into_bytes());
    files.insert(
        ".gitignore".into(),
        b"node_modules/\ntarget/\ndist/\n*.log\n".to_vec(),
    );
    Ok(files)
}

/// Checks every output path before creating the staging directory.
///
/// # Errors
/// Returns an error if any output would overwrite a file or traverse a symlink.
fn preflight(directory: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<(), String> {
    if directory.is_symlink() {
        return Err(format!("{} is a symlink", directory.display()));
    }
    if directory.exists() && !directory.is_dir() {
        return Err(format!("{} is not a directory", directory.display()));
    }
    for relative in files.keys() {
        if !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(format!("unsafe generated path {}", relative.display()));
        }
        let path = directory.join(relative);
        for parent in path.ancestors().take_while(|parent| *parent != directory) {
            if parent.is_symlink() {
                return Err(format!("symlink conflict: {}", parent.display()));
            }
            if parent.exists() && !parent.is_dir() {
                return Err(format!(
                    "generated file already exists: {}",
                    parent.display()
                ));
            }
        }
        if path.exists() || path.is_symlink() {
            return Err(format!("generated file already exists: {}", path.display()));
        }
    }
    Ok(())
}

/// Stages generated files and moves them into the final directory with rollback.
///
/// # Errors
/// Returns an error after removing files created by this call.
fn install(directory: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<(), String> {
    let parent = directory
        .parent()
        .ok_or("application directory has no parent")?;
    if !parent.is_dir() {
        return Err(format!(
            "parent directory does not exist: {}",
            parent.display()
        ));
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let stage = parent.join(format!(".argui-init-{}-{stamp}", std::process::id()));
    fs::create_dir(&stage).map_err(|error| format!("{}: {error}", stage.display()))?;
    let staged = (|| -> Result<(), String> {
        for (relative, body) in files {
            let path = stage.join(relative);
            fs::create_dir_all(path.parent().ok_or("generated path lacks parent")?)
                .map_err(|error| error.to_string())?;
            fs::write(&path, body).map_err(|error| format!("{}: {error}", path.display()))?;
        }
        Ok(())
    })();
    if let Err(error) = staged {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    if !directory.exists() {
        return fs::rename(&stage, directory).map_err(|error| {
            let _ = fs::remove_dir_all(&stage);
            format!("{}: {error}", directory.display())
        });
    }
    let mut installed = Vec::new();
    let result = (|| -> Result<(), String> {
        for relative in files.keys() {
            let output = directory.join(relative);
            fs::create_dir_all(output.parent().ok_or("generated path lacks parent")?)
                .map_err(|error| error.to_string())?;
            fs::rename(stage.join(relative), &output)
                .map_err(|error| format!("{}: {error}", output.display()))?;
            installed.push(output);
        }
        Ok(())
    })();
    if result.is_err() {
        for path in installed {
            let _ = fs::remove_file(path);
        }
    }
    let _ = fs::remove_dir_all(&stage);
    result
}

/// Returns the app's pinned Rust manifest, including selected runtime features.
fn rust_manifest(options: &Options) -> String {
    let tasks = if options.features.iter().any(|feature| feature == "tasks") {
        ", features = [\"tasks\"]"
    } else {
        ""
    };
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.89\"\n\n[dependencies]\nargui-runtime = {{ version = \"={}\"{} }}\n",
        options.name,
        env!("CARGO_PKG_VERSION"),
        tasks
    )
}

/// Returns the first pure Rust application source.
fn rust_main() -> &'static str {
    include_str!("../../assets/templates/rust-main.rs")
}

/// Returns a standalone Bun package with app-local SDK adapters.
fn js_manifest(options: &Options) -> String {
    let framework = &options.framework;
    let runtime = if framework == "solid" {
        "\"solid-js\": \"1.9.15\""
    } else {
        "\"react\": \"19.2.0\", \"react-reconciler\": \"0.33.0\""
    };
    let plugin = if framework == "solid" {
        ", \"vite-plugin-solid\": \"2.11.14\""
    } else {
        ", \"@types/react\": \"19.2.0\", \"@types/react-reconciler\": \"0.33.0\""
    };
    format!(
        "{{\n  \"name\": \"{}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"arguiSdk\": {{ \"version\": \"{}\", \"delivery\": \"cli-cache-development\" }},\n  \"dependencies\": {{ {runtime} }},\n  \"devDependencies\": {{ \"typescript\": \"5.9.3\", \"vite\": \"8.2.2\", \"@types/bun\": \"1.3.11\"{plugin} }}\n}}\n",
        options.name,
        env!("CARGO_PKG_VERSION")
    )
}

/// Returns the local SDK-aware TypeScript configuration.
fn tsconfig(options: &Options) -> String {
    format!(
        "{{\n  \"compilerOptions\": {{\n    \"target\": \"ES2022\", \"module\": \"ESNext\", \"moduleResolution\": \"Bundler\",\n    \"lib\": [\"ES2022\", \"DOM\"], \"jsx\": \"preserve\",\n    \"jsxImportSource\": \"@argui/{}\", \"strict\": true, \"noEmit\": true,\n    \"skipLibCheck\": true, \"types\": [\"bun\"],\n    \"allowImportingTsExtensions\": true, \"resolveJsonModule\": true\n  }},\n  \"include\": [\"src/**/*.ts\", \"src/**/*.tsx\"]\n}}\n",
        options.framework
    )
}

/// Returns one Vite configuration with distinct native and Web outputs.
fn vite_config(options: &Options) -> String {
    let plugin = if options.framework == "solid" {
        "import solid from 'vite-plugin-solid'\n"
    } else {
        ""
    };
    let adapter = if options.framework == "solid" {
        "plugins: [solid({ solid: { moduleName: '@argui/solid', generate: 'universal' }, hot: false })],"
    } else {
        "oxc: { jsx: { runtime: 'automatic', importSource: '@argui/react' } }, define: { 'process.env.NODE_ENV': JSON.stringify('production') },"
    };
    format!(
        "import {{ defineConfig }} from 'vite'\n{plugin}export default defineConfig(({{ mode }}) => ({{ root: import.meta.dirname, {adapter} ssr: {{ noExternal: true, resolve: {{ conditions: ['browser'] }} }}, build: mode === 'native' ? {{ ssr: 'src/main.tsx', outDir: 'dist/native', target: 'es2022', rollupOptions: {{ output: {{ entryFileNames: 'app.mjs' }} }} }} : {{ outDir: 'dist/web', target: 'es2022' }} }}))\n"
    )
}

/// Returns the browser entry page.
fn html(name: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{name} · Argui</title><style>html,body,#argui-root{{margin:0;width:100%;height:100%;min-height:100vh;background:#0f172a}}canvas{{display:block;width:100%;height:100%}}</style></head><body><main id=\"argui-root\"></main><script type=\"module\" src=\"/src/web.ts\"></script></body></html>\n"
    )
}

/// Returns concise generated project instructions.
fn readme(options: &Options) -> String {
    let install = if options.framework == "rust" {
        ""
    } else {
        "bun install\n"
    };
    format!(
        "# {}\n\nArgui {} application. Targets: {}.\n\nThe CLI supplies Argui SDK and host sources from its verified local cache outside this project. This is an explicit development distribution until Argui {} crates and JS SDK are published together. The application tracks the exact SDK version and hash in `argui.json`.\n\n```sh\n{install}argui check\nargui build release\n```\n\nSelect one output with `argui build release --target native` or `--target web`. Web artifacts go to `dist/web/`; native artifacts go to `dist/desktop/`. `argui dev --target native` starts the native host; `argui dev --target web` starts Vite.\n",
        options.name,
        options.framework,
        options.targets.join(", "),
        env!("CARGO_PKG_VERSION")
    )
}
