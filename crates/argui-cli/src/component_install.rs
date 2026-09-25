//! Atomic installation of versioned Argui widget sources.

use crate::components::{self, ComponentEntry, Registry};
use crate::project::{self, Framework};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

const MAX_FILE: u64 = 256 * 1024;

/// Installs all requested framework/name pairs from one checked-out release.
/// `requests` may mix adapters; sources and conflicts are validated as one batch.
///
/// # Errors
/// Returns an error for unknown names, unsafe sources, conflicts, or writes.
pub fn add_batch(cwd: &Path, requests: &[(Framework, String)]) -> Result<(), String> {
    if crate::standalone::is_project(cwd) {
        let app = crate::standalone::load(cwd)?;
        if app.distribution != "release" {
            return Err(format!(
                "Argui {} is an embedded development SDK; no matching published widget source is verified. `argui add` requires a release whose registry and sources exist at the exact tag.",
                app.argui_version
            ));
        }
        let registry_json = crate::source_cache::registry(&app.argui_version)?;
        let registry: Registry =
            serde_json::from_str(&registry_json).map_err(|error| error.to_string())?;
        if registry.argui_version() != Some(app.argui_version.as_str()) {
            return Err("release registry version does not match the application SDK".into());
        }
        let checksums = registry.files;
        return install_requests(cwd, requests, &registry_json, |path| {
            let expected = checksums
                .get(path)
                .ok_or_else(|| format!("missing release checksum for `{path}`"))?;
            crate::source_cache::file(&app.argui_version, path, expected)
        });
    }
    let root = project::find_root(cwd)?;
    let registry_json = read_bounded(&root.join("components/registry.json"))?;
    let source_root = root
        .join("packages/widgets/src")
        .canonicalize()
        .map_err(|error| format!("widget source directory: {error}"))?;
    install_requests(cwd, requests, &registry_json, |path| {
        let source = source_root.join(path);
        let resolved = source
            .canonicalize()
            .map_err(|error| format!("{}: {error}", source.display()))?;
        if !resolved.starts_with(&source_root) {
            return Err(format!("component source escapes widget directory: {path}"));
        }
        read_bounded(&resolved)
    })
}

/// Installs components from `registry_json` using `source` to obtain validated
/// repository-relative files. This also supports offline or mirrored registries.
/// `cwd` is the project directory, `framework` its adapter, and `names` are
/// requested components. Returns after recording dependencies in argui.json.
///
/// # Errors
/// Returns an error for invalid registry entries, conflicts, source failures,
/// or filesystem writes.
pub fn install(
    cwd: &Path,
    framework: Framework,
    names: &[String],
    registry_json: &str,
    source: impl FnMut(&str) -> Result<String, String>,
) -> Result<(), String> {
    let project = project::load(cwd)?;
    if project.framework != framework {
        return Err(format!(
            "project uses {}; requested {}",
            project.framework.name(),
            framework.name()
        ));
    }
    install_requests(
        cwd,
        &names
            .iter()
            .map(|name| (framework, name.clone()))
            .collect::<Vec<_>>(),
        registry_json,
        source,
    )
}

/// Applies one validated component batch, including mixed framework variants.
///
/// # Errors
/// Returns an error before mutation for invalid metadata, sources, or conflicts;
/// file writes are rolled back if an atomic install step fails.
fn install_requests(
    cwd: &Path,
    requests: &[(Framework, String)],
    registry_json: &str,
    mut source: impl FnMut(&str) -> Result<String, String>,
) -> Result<(), String> {
    let mut project = project::load(cwd)?;
    let package_update = package_dependencies(cwd, project.framework, requests)?;
    let registry: Registry =
        serde_json::from_str(registry_json).map_err(|error| error.to_string())?;
    let mut paths = std::collections::BTreeSet::new();
    for (framework, name) in requests {
        paths.extend(components::files(
            &registry,
            *framework,
            std::slice::from_ref(name),
        )?);
    }
    let paths: Vec<_> = paths.into_iter().collect();
    let standalone = crate::standalone::is_project(cwd);
    for path in &paths {
        let output = output_path(cwd, path, standalone)?;
        if output
            .ancestors()
            .take_while(|parent| *parent != cwd)
            .any(Path::is_symlink)
        {
            return Err(format!("symlink conflict: {}", output.display()));
        }
        if output.exists() && !project.component_files.contains(path) {
            return Err(format!(
                "untracked component file conflicts with {}",
                output.display()
            ));
        }
        if output.exists() && registry.version == 2 {
            let body = read_bounded(&output)?;
            let expected = registry
                .files
                .get(path)
                .expect("validated registry checksum");
            if hash_body(&body) != *expected {
                return Err(format!(
                    "locally modified component file conflicts with {}",
                    output.display()
                ));
            }
        }
    }
    let missing: Vec<_> = paths
        .iter()
        .filter(|path| output_path(cwd, path, standalone).is_ok_and(|output| !output.exists()))
        .cloned()
        .collect();
    let mut staged = Vec::new();
    for path in &missing {
        let body = source(path)?;
        if registry.version == 2
            && registry
                .files
                .get(path)
                .is_some_and(|expected| hash_body(&body) != *expected)
        {
            return Err(format!("source checksum mismatch for `{path}`"));
        }
        staged.push((path, body));
    }
    for (framework, name) in requests {
        let canonical = if name == "input" { "input-field" } else { name };
        let entry = format!("{}/{canonical}", framework.name());
        if !project.components.contains(&entry) {
            project.components.push(entry.clone());
        }
        if let Some(ComponentEntry::Versioned { version, .. }) = registry.components.get(canonical)
        {
            project.component_versions.insert(entry, version.clone());
        }
    }
    for path in paths {
        if !project.component_files.contains(&path) {
            project.component_files.push(path.clone());
        }
        if let Some(hash) = registry.files.get(&path) {
            project.component_checksums.insert(path, hash.clone());
        }
    }
    project.components.sort();
    project.component_files.sort();
    let original_manifest = fs::read(cwd.join("argui.json")).map_err(|error| error.to_string())?;
    let manifest = if standalone {
        let mut original: serde_json::Value =
            serde_json::from_slice(&original_manifest).map_err(|error| error.to_string())?;
        original["components"] = serde_json::json!(project.components);
        original["componentFiles"] = serde_json::json!(project.component_files);
        original["componentChecksums"] = serde_json::json!(project.component_checksums);
        original["componentVersions"] = serde_json::json!(project.component_versions);
        serde_json::to_string_pretty(&original).map_err(|error| error.to_string())?
    } else {
        serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?
    };
    let stage = cwd.join(format!(".argui-component-stage-{}", std::process::id()));
    fs::create_dir(&stage).map_err(|error| format!("{}: {error}", stage.display()))?;
    let result = (|| {
        for (path, body) in &staged {
            project::write(&stage.join(path), body)?;
        }
        project::write(&stage.join("argui.json"), &manifest)?;
        if let Some(package) = &package_update {
            project::write(&stage.join("package.json"), package)?;
        }
        for (path, _) in &staged {
            let output = output_path(cwd, path, standalone)?;
            if output.exists() {
                return Err(format!(
                    "{} appeared while reading sources; no files overwritten",
                    output.display()
                ));
            }
        }
        let mut installed = Vec::new();
        for (path, _) in &staged {
            let output = output_path(cwd, path, standalone)?;
            let write_result = output
                .parent()
                .map_or(Ok(()), fs::create_dir_all)
                .and_then(|()| fs::hard_link(stage.join(path), &output));
            if let Err(error) = write_result {
                for file in installed {
                    let _ = fs::remove_file(file);
                }
                return Err(format!("{}: {error}", output.display()));
            }
            installed.push(output);
        }
        if let Err(error) = fs::rename(stage.join("argui.json"), cwd.join("argui.json")) {
            for file in installed {
                let _ = fs::remove_file(file);
            }
            return Err(format!("argui.json: {error}"));
        }
        if package_update.is_some()
            && let Err(error) = fs::rename(stage.join("package.json"), cwd.join("package.json"))
        {
            let _ = fs::write(cwd.join("argui.json"), &original_manifest);
            for file in installed {
                let _ = fs::remove_file(file);
            }
            return Err(format!("package.json: {error}"));
        }
        for (path, _) in &staged {
            println!("Added {}", output_path(cwd, path, standalone)?.display());
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(stage);
    result?;
    if missing.is_empty() {
        println!("Components already present; manifest updated.");
    }
    Ok(())
}

/// Maps a versioned widget path into the app's editable component tree.
///
/// # Errors
/// Returns an error when a standalone registry uses an unknown source prefix.
fn output_path(cwd: &Path, path: &str, standalone: bool) -> Result<std::path::PathBuf, String> {
    if !standalone {
        return Ok(cwd.join("src/argui-ui").join(path));
    }
    let (adapter, file) = path
        .split_once('/')
        .ok_or("widget source has no adapter directory")?;
    let directory = match adapter {
        "solid" => "solid-components",
        "react" => "react-components",
        "shared" => "shared",
        _ => return Err(format!("unsupported widget source prefix `{adapter}`")),
    };
    Ok(cwd.join("ui").join(directory).join(file))
}

/// Prepares adapter dependencies when an install includes a second framework.
///
/// # Errors
/// Returns an error for a missing or malformed application package manifest.
fn package_dependencies(
    cwd: &Path,
    default: Framework,
    requests: &[(Framework, String)],
) -> Result<Option<String>, String> {
    if requests.iter().all(|(framework, _)| *framework == default) {
        return Ok(None);
    }
    let package_path = cwd.join("package.json");
    let body = fs::read_to_string(&package_path)
        .map_err(|error| format!("{}: {error}", package_path.display()))?;
    let mut package: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| format!("{}: {error}", package_path.display()))?;
    let standalone = crate::standalone::is_project(cwd);
    let pinned = if standalone {
        Some(crate::standalone::load(cwd)?.argui_version)
    } else {
        None
    };
    let dependencies = package
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("package.json lacks dependencies")?;
    for (framework, _) in requests {
        let (adapter, runtime, version) = match framework {
            Framework::Solid => ("@argui/solid", "solid-js", "1.9.15"),
            Framework::React => ("@argui/react", "react", "19.2.0"),
        };
        dependencies
            .entry(adapter)
            .or_insert_with(|| pinned.as_deref().unwrap_or("workspace:*").into());
        dependencies
            .entry(runtime)
            .or_insert_with(|| version.into());
        if *framework == Framework::React {
            dependencies
                .entry("react-reconciler")
                .or_insert_with(|| "0.33.0".into());
        }
    }
    if standalone
        && requests
            .iter()
            .any(|(framework, _)| *framework == Framework::React)
    {
        let dev = package
            .get_mut("devDependencies")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("package.json lacks devDependencies")?;
        dev.entry("@types/react").or_insert_with(|| "19.2.0".into());
        dev.entry("@types/react-reconciler")
            .or_insert_with(|| "0.33.0".into());
    }
    serde_json::to_string_pretty(&package)
        .map(|body| Some(format!("{body}\n")))
        .map_err(|error| error.to_string())
}

/// Returns the lowercase SHA-256 digest of one UTF-8 source body.
fn hash_body(body: &str) -> String {
    format!("{:x}", Sha256::digest(body.as_bytes()))
}

/// Reads a UTF-8 source file with a fixed byte limit.
/// `path` is a registry or source file within the versioned checkout.
/// Returns its contents after checking its size and encoding.
///
/// # Errors
/// Returns an error for an unreadable, oversized, or non-UTF-8 file.
pub(super) fn read_bounded(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if file
        .metadata()
        .map_err(|error| format!("{}: {error}", path.display()))?
        .len()
        > MAX_FILE
    {
        return Err(format!("{}: exceeds {MAX_FILE} bytes", path.display()));
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(format!("{}: exceeds {MAX_FILE} bytes", path.display()));
    }
    String::from_utf8(bytes).map_err(|error| format!("{}: invalid UTF-8: {error}", path.display()))
}
