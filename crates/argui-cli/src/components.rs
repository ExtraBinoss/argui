//! Versioned Argui widget registry and catalog.

use crate::project::{self, Framework};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

#[derive(Deserialize)]
pub(super) struct Registry {
    pub(super) version: u32,
    #[serde(rename = "arguiVersion")]
    argui_version: Option<String>,
    #[serde(default)]
    pub(super) files: BTreeMap<String, String>,
    pub(super) components: BTreeMap<String, ComponentEntry>,
}

impl Registry {
    /// Returns the Argui release version recorded in this registry.
    pub(super) fn argui_version(&self) -> Option<&str> {
        self.argui_version.as_deref()
    }
}

#[derive(Deserialize)]
pub(super) struct ComponentEntry {
    pub(super) version: String,
    pub(super) source: BTreeMap<String, String>,
    pub(super) solid: Vec<String>,
    pub(super) react: Vec<String>,
}

impl ComponentEntry {
    /// Returns paths for the selected adapter and validates immutable source metadata.
    ///
    /// # Errors
    /// Returns an error if the source URL or component version is absent or malformed.
    fn paths(&self, framework: Framework) -> Result<&[String], String> {
        if self.version.is_empty()
            || !self.source.get(framework.name()).is_some_and(|url| {
                url.starts_with("https://github.com/") && url.contains("/blob/v")
            })
        {
            return Err("component has no immutable source URL or version".into());
        }
        Ok(if framework == Framework::Solid {
            &self.solid
        } else {
            &self.react
        })
    }
}

/// Resolves requested component names into fixed repository-relative source paths.
/// `framework` selects adapter-specific files; `names` contains user input.
/// Returns each required path once, or an error for unknown names.
///
/// # Errors
/// Returns an error for an unknown or malformed component name.
pub(super) fn files(
    registry: &Registry,
    framework: Framework,
    names: &[String],
) -> Result<Vec<String>, String> {
    if registry.version != 2 {
        return Err(format!(
            "unsupported component registry version {}",
            registry.version
        ));
    }
    if registry.argui_version.as_deref().is_none_or(str::is_empty) {
        return Err("versioned registry lacks arguiVersion".into());
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
        let release = registry
            .argui_version
            .as_deref()
            .ok_or("versioned registry lacks arguiVersion")?;
        let expected =
            format!("https://github.com/ExtraBinoss/argui/blob/v{release}/packages/widgets/src/");
        if !component
            .source
            .get(framework.name())
            .is_some_and(|url| url.starts_with(&expected))
        {
            return Err(format!(
                "component `{name}` source does not match Argui v{release}"
            ));
        }
        let dependencies = component
            .paths(framework)
            .map_err(|error| format!("component `{name}`: {error}"))?;
        for path in dependencies {
            if !safe_path(path) {
                return Err(format!("unsafe registry path `{path}`"));
            }
            if !registry
                .files
                .get(path)
                .is_some_and(|hash| valid_hash(hash))
            {
                return Err(format!(
                    "component `{name}` lacks a SHA-256 checksum for `{path}`"
                ));
            }
            paths.insert(path.clone());
        }
    }
    Ok(paths.into_iter().collect())
}

/// Reports whether `hash` is exactly one lowercase SHA-256 digest.
fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Serialize)]
struct CatalogRow {
    name: String,
    source: String,
    framework: String,
    version: String,
    installed: bool,
    files: Vec<String>,
}

/// Formats installable component variants from a registry and optional project.
/// `framework` filters variants; `json` selects structured or columnar output.
///
/// # Errors
/// Returns an error for malformed registry data or unsupported registry versions.
pub fn catalog(
    registry_json: &str,
    project: Option<&project::Project>,
    framework: Option<Framework>,
    json: bool,
) -> Result<String, String> {
    catalog_with_sources(registry_json, project, framework, json, true)
}

/// Formats the catalog while withholding unverified release links.
///
/// # Errors
/// Returns an error for malformed registry data or component metadata.
fn catalog_with_sources(
    registry_json: &str,
    project: Option<&project::Project>,
    framework: Option<Framework>,
    json: bool,
    published: bool,
) -> Result<String, String> {
    let registry: Registry =
        serde_json::from_str(registry_json).map_err(|error| error.to_string())?;
    if registry.version != 2 {
        return Err(format!(
            "unsupported component registry version {}",
            registry.version
        ));
    }
    let mut rows = Vec::new();
    for (name, component) in &registry.components {
        for adapter in [Framework::Solid, Framework::React] {
            if framework.is_some_and(|selected| selected != adapter) {
                continue;
            }
            let paths = files(&registry, adapter, std::slice::from_ref(name))?;
            rows.push(CatalogRow {
                name: name.clone(),
                source: if published {
                    component
                        .source
                        .get(adapter.name())
                        .cloned()
                        .unwrap_or_default()
                } else {
                    "unpublished development snapshot".into()
                },
                framework: adapter.name().to_owned(),
                version: component.version.clone(),
                installed: project.is_some_and(|app| {
                    app.components
                        .contains(&format!("{}/{name}", adapter.name()))
                }),
                files: paths,
            });
        }
    }
    if json {
        serde_json::to_string_pretty(&rows).map_err(|error| error.to_string())
    } else {
        let mut lines = vec!["NAME  SOURCE  FRAMEWORK  VERSION  INSTALLED".to_owned()];
        lines.extend(rows.into_iter().map(|row| {
            format!(
                "{}  {}  {}  {}  {}",
                row.name,
                row.source,
                row.framework,
                row.version,
                if row.installed { "yes" } else { "no" }
            )
        }));
        Ok(lines.join("\n"))
    }
}

/// Lists components from the release registry or CLI's embedded catalog.
/// `cwd` may be a project directory, while `project_path` explicitly selects one.
///
/// # Errors
/// Returns an error for an invalid project or catalog.
pub fn list(
    cwd: &Path,
    project_path: Option<&Path>,
    framework: Option<Framework>,
    json: bool,
) -> Result<(), String> {
    let app = project_path.or_else(|| {
        cwd.ancestors()
            .find(|ancestor| ancestor.join("argui.json").is_file())
    });
    let project = app.map(crate::standalone::load).transpose()?;
    let registry = if let Some(app) = &project
        && app.distribution == "release"
    {
        crate::source_cache::registry(&app.argui_version)?
    } else {
        include_str!("../../../components/registry.json").to_owned()
    };
    let published = project
        .as_ref()
        .is_some_and(|app| app.distribution == "release")
        || project.is_none() && crate::sdk::release_available();
    println!(
        "{}",
        catalog_with_sources(&registry, project.as_ref(), framework, json, published)?
    );
    Ok(())
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
