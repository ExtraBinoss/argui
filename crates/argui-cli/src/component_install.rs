//! Atomic installation of versioned Argui widget sources.

use crate::components::{self, Registry};
use crate::project::{self, Framework};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

const MAX_FILE: u64 = 256 * 1024;

/// Installs all requested framework/name pairs into one standalone application.
/// `requests` may mix adapters; sources and conflicts are validated as one batch.
///
/// # Errors
/// Returns an error for unknown names, unsafe sources, conflicts, or writes.
pub fn add_batch(cwd: &Path, requests: &[(Framework, String)]) -> Result<(), String> {
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
    install_requests(cwd, requests, &registry_json, |path| {
        let expected = checksums
            .get(path)
            .ok_or_else(|| format!("missing release checksum for `{path}`"))?;
        crate::source_cache::file(&app.argui_version, path, expected)
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
    let project = crate::standalone::load(cwd)?;
    if project.framework != framework.name() {
        return Err(format!(
            "project uses {}; requested {}",
            project.framework,
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
    let mut project = crate::standalone::load(cwd)?;
    let package_update =
        package_dependencies(cwd, Framework::parse(&project.framework)?, requests)?;
    let registry: Registry =
        serde_json::from_str(registry_json).map_err(|error| error.to_string())?;
    if registry.argui_version() != Some(project.argui_version.as_str()) {
        return Err("component registry version does not match the application SDK".into());
    }
    let mut paths = std::collections::BTreeSet::new();
    for (framework, name) in requests {
        paths.extend(components::files(
            &registry,
            *framework,
            std::slice::from_ref(name),
        )?);
    }
    let paths: Vec<_> = paths.into_iter().collect();
    for path in &paths {
        let output = output_path(cwd, path)?;
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
        if output.exists() {
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
        .filter(|path| output_path(cwd, path).is_ok_and(|output| !output.exists()))
        .cloned()
        .collect();
    let mut staged = Vec::new();
    for path in &missing {
        let body = source(path)?;
        if registry
            .files
            .get(path)
            .is_some_and(|expected| hash_body(&body) != *expected)
        {
            return Err(format!("source checksum mismatch for `{path}`"));
        }
        staged.push((path, body));
    }
    for (framework, name) in requests {
        let entry = format!("{}/{name}", framework.name());
        if !project.components.contains(&entry) {
            project.components.push(entry.clone());
        }
        let component = registry.components.get(name).expect("validated component");
        project
            .component_versions
            .insert(entry, component.version.clone());
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
    let manifest = serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?;
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
            let output = output_path(cwd, path)?;
            if output.exists() {
                return Err(format!(
                    "{} appeared while reading sources; no files overwritten",
                    output.display()
                ));
            }
        }
        let mut installed = Vec::new();
        for (path, _) in &staged {
            let output = output_path(cwd, path)?;
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
            println!("Added {}", output_path(cwd, path)?.display());
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
/// Returns an error when the registry uses an unknown source prefix.
fn output_path(cwd: &Path, path: &str) -> Result<std::path::PathBuf, String> {
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
    let pinned = crate::standalone::load(cwd)?.argui_version;
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
            .or_insert_with(|| pinned.clone().into());
        dependencies
            .entry(runtime)
            .or_insert_with(|| version.into());
        if *framework == Framework::React {
            dependencies
                .entry("react-reconciler")
                .or_insert_with(|| "0.33.0".into());
        }
    }
    if requests
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
/// `path` is a downloaded registry or widget source file.
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
