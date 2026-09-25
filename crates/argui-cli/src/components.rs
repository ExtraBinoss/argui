//! Bounded, conflict-aware component installation from the Argui repository.

use crate::project::{self, Framework};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path},
};

const MAX_FILE: u64 = 256 * 1024;

#[derive(Deserialize)]
struct Registry {
    version: u32,
    components: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

/// Resolves requested component names into fixed repository-relative source paths.
/// `framework` selects adapter-specific files; `names` contains user input.
/// Returns each required path once, or an error for unknown names.
///
/// # Errors
/// Returns an error for an unknown or malformed component name.
fn files(
    registry: &Registry,
    framework: Framework,
    names: &[String],
) -> Result<Vec<String>, String> {
    if registry.version != 1 {
        return Err(format!(
            "unsupported component registry version {}",
            registry.version
        ));
    }
    let mut paths = BTreeSet::new();
    for name in names {
        if !project::valid_name(name) {
            return Err(format!("invalid component name `{name}`"));
        }
        let component = registry.components.get(name).ok_or_else(|| {
            format!(
                "unknown component `{name}`; available: {}",
                registry
                    .components
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        let dependencies = component
            .get(framework.name())
            .ok_or_else(|| format!("component `{name}` has no {} variant", framework.name()))?;
        for path in dependencies {
            if !safe_path(path) {
                return Err(format!("unsafe registry path `{path}`"));
            }
            paths.insert(path.clone());
        }
    }
    Ok(paths.into_iter().collect())
}

/// Resolves `names` against a registry JSON string for `framework`.
/// Returns dependency paths from the registry after validating every path.
///
/// # Errors
/// Returns an error for malformed JSON, unsupported versions, missing names,
/// missing framework variants, or unsafe paths.
pub fn component_files(
    registry_json: &str,
    framework: Framework,
    names: &[String],
) -> Result<Vec<String>, String> {
    let registry: Registry =
        serde_json::from_str(registry_json).map_err(|error| error.to_string())?;
    files(&registry, framework, names)
}

/// Checks that a registry `path` stays below the widget source directory.
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 180
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'/' | b'-' | b'_' | b'.')
        })
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        && (path.ends_with(".ts") || path.ends_with(".tsx"))
}

/// Installs Argui components into `cwd/src/argui-ui` and records them in argui.json.
/// `framework` must match the project adapter; `names` must be known components.
/// Sources come from the versioned Argui checkout containing `cwd`.
///
/// # Errors
/// Returns an error for framework mismatch, missing or unsafe sources, conflicts, or writes.
pub fn add(cwd: &Path, framework: Framework, names: &[String]) -> Result<(), String> {
    let project = project::load(cwd)?;
    if project.framework != framework {
        return Err(format!(
            "project uses {}; requested {}",
            project.framework.name(),
            framework.name()
        ));
    }
    let root = project::find_root(cwd)?;
    let registry_json = read_bounded(&root.join("components/registry.json"))?;
    let source_root = root
        .join("packages/widgets/src")
        .canonicalize()
        .map_err(|error| format!("widget source directory: {error}"))?;
    install(cwd, framework, names, &registry_json, |path| {
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
    mut source: impl FnMut(&str) -> Result<String, String>,
) -> Result<(), String> {
    let mut project = project::load(cwd)?;
    if project.framework != framework {
        return Err(format!(
            "project uses {}; requested {}",
            project.framework.name(),
            framework.name()
        ));
    }
    let paths = component_files(registry_json, framework, names)?;
    let destination = cwd.join("src/argui-ui");
    for path in &paths {
        let output = destination.join(path);
        if output.is_symlink()
            || output.parent().is_some_and(|parent| parent.is_symlink())
            || destination.is_symlink()
            || cwd.join("src").is_symlink()
        {
            return Err(format!("symlink conflict: {}", output.display()));
        }
        if output.exists() && !project.component_files.contains(path) {
            return Err(format!(
                "untracked component file conflicts with {}",
                output.display()
            ));
        }
    }
    let missing: Vec<_> = paths
        .iter()
        .filter(|path| !destination.join(path).exists())
        .cloned()
        .collect();
    let mut staged = Vec::new();
    for path in &missing {
        staged.push((path, source(path)?));
    }
    for (path, body) in staged {
        let output = destination.join(path);
        if output.exists() {
            return Err(format!(
                "{} appeared while reading sources; no files overwritten",
                output.display()
            ));
        }
        project::write(&output, &body)?;
        println!("Added {}", output.display());
    }
    for name in names {
        let entry = format!("{}/{name}", framework.name());
        if !project.components.contains(&entry) {
            project.components.push(entry);
        }
    }
    for path in paths {
        if !project.component_files.contains(&path) {
            project.component_files.push(path);
        }
    }
    project.components.sort();
    project.component_files.sort();
    let manifest = serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?;
    fs::write(cwd.join("argui.json"), manifest).map_err(|error| error.to_string())?;
    if missing.is_empty() {
        println!("Components already present; manifest updated.");
    }
    Ok(())
}

/// Reads a UTF-8 source file with a fixed byte limit.
/// `path` is a registry or source file within the versioned checkout.
/// Returns its contents after checking its size and encoding.
///
/// # Errors
/// Returns an error for an unreadable, oversized, or non-UTF-8 file.
fn read_bounded(path: &Path) -> Result<String, String> {
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
