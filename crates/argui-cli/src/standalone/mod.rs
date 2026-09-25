//! Standalone application manifests and CLI routing.

mod build;
mod init;
mod menu;

use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
};

/// Manifest format emitted into the application directory.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    /// Distinguishes self-contained projects from old checkout projects.
    pub project_version: u8,
    /// Safe package and application name.
    pub name: String,
    /// Rust, Solid, or React presentation source.
    pub framework: String,
    /// Native and/or Web output targets.
    pub targets: Vec<String>,
    /// Build capabilities explicitly selected by the app owner.
    pub features: Vec<String>,
    /// Exact Argui crate and SDK release version.
    pub argui_version: String,
    /// Hash of the CLI-owned SDK snapshot.
    pub sdk_sha256: String,
    /// Provenance of the SDK snapshot until a matching release is published.
    pub distribution: String,
    /// Installed component names including their adapter.
    #[serde(default)]
    pub components: Vec<String>,
    /// Installed widget source paths.
    #[serde(default)]
    pub component_files: Vec<String>,
    /// Widget source hashes.
    #[serde(default)]
    pub component_checksums: std::collections::BTreeMap<String, String>,
    /// Widget component versions.
    #[serde(default)]
    pub component_versions: std::collections::BTreeMap<String, String>,
}

/// Returns whether `directory` contains a standalone Argui manifest.
pub(crate) fn is_project(directory: &Path) -> bool {
    fs::read(directory.join("argui.json"))
        .ok()
        .and_then(|body| serde_json::from_slice::<serde_json::Value>(&body).ok())
        .is_some_and(|value| value["projectVersion"] == 2)
}

/// Loads and validates a standalone project in `directory`.
///
/// # Errors
/// Returns an error for malformed, unsupported, or inconsistent metadata.
pub(crate) fn load(directory: &Path) -> Result<Manifest, String> {
    let path = directory.join("argui.json");
    let body = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let manifest: Manifest =
        serde_json::from_slice(&body).map_err(|error| format!("{}: {error}", path.display()))?;
    if manifest.project_version != 2 || !crate::project::valid_name(&manifest.name) {
        return Err(format!(
            "{}: unsupported or invalid application manifest",
            path.display()
        ));
    }
    if !matches!(manifest.framework.as_str(), "rust" | "solid" | "react") {
        return Err(format!("{}: unknown framework", path.display()));
    }
    if manifest.targets.is_empty()
        || manifest
            .targets
            .iter()
            .any(|target| !matches!(target.as_str(), "native" | "web"))
        || manifest.targets.len()
            != manifest
                .targets
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
    {
        return Err(format!("{}: invalid target set", path.display()));
    }
    if manifest.framework == "rust" && manifest.targets.iter().any(|target| target == "web") {
        return Err(
            "the Rust scaffold currently supports native only; use --targets native".into(),
        );
    }
    if manifest.argui_version != env!("CARGO_PKG_VERSION")
        || manifest.sdk_sha256 != crate::sdk::digest()
    {
        return Err(format!(
            "{} needs Argui CLI {} and its exact SDK snapshot",
            path.display(),
            manifest.argui_version
        ));
    }
    Ok(manifest)
}

/// Parses a standalone `init` invocation and creates its application.
///
/// # Errors
/// Returns a clear usage, capability, or filesystem error.
pub(crate) fn init(cwd: &Path, args: &[String]) -> Result<(), String> {
    let mut framework = None;
    let mut directory = cwd.to_path_buf();
    let mut name = None;
    let mut targets = None;
    let mut features = Vec::new();
    let mut yes = false;
    let mut position = 0;
    while position < args.len() {
        match args[position].as_str() {
            "rust" | "solid" | "react" if framework.is_none() => {
                framework = Some(args[position].clone())
            }
            "--dir" | "--name" | "--targets" | "--feature" => {
                let flag = args[position].as_str();
                position += 1;
                let value = args
                    .get(position)
                    .ok_or_else(|| format!("{flag} needs a value"))?;
                match flag {
                    "--dir" => directory = cwd.join(value),
                    "--name" => name = Some(value.clone()),
                    "--targets" => targets = Some(value.split(',').map(str::to_owned).collect()),
                    _ => features.push(value.clone()),
                }
            }
            "--yes" => yes = true,
            unknown => {
                return Err(format!(
                    "unknown init argument `{unknown}`; use --dir, --targets, --feature, or --yes"
                ));
            }
        }
        position += 1;
    }
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal() && !yes;
    if interactive {
        let choice = menu::choose(framework, targets, features)?;
        framework = Some(choice.0);
        targets = Some(choice.1);
        features = choice.2;
    }
    let framework = framework.ok_or(
        "non-interactive init needs rust, solid, or react; or pass --yes for Solid defaults",
    )?;
    let targets = targets.unwrap_or_else(|| {
        if framework == "rust" {
            vec!["native".into()]
        } else {
            vec!["native".into(), "web".into()]
        }
    });
    let name = name.unwrap_or_else(|| {
        directory
            .file_name()
            .and_then(|part| part.to_str())
            .unwrap_or("")
            .to_owned()
    });
    let options = init::Options {
        directory,
        name,
        framework,
        targets,
        features,
    };
    init::create(&options)
}

/// Routes a check, build, dev, or run command for a standalone project.
///
/// # Errors
/// Returns an error for invalid flags or failed build processes.
pub(crate) fn command(cwd: &Path, command: &str, args: &[String]) -> Result<(), String> {
    let mut directory = cwd.to_path_buf();
    let mut target = None;
    let mut release = false;
    let mut json = false;
    let mut position = 0;
    while position < args.len() {
        match args[position].as_str() {
            "--target" => {
                position += 1;
                target = Some(
                    args.get(position)
                        .ok_or("--target needs native or web")?
                        .clone(),
                );
            }
            "dev" if command == "build" || command == "run" => release = false,
            "release" if command == "build" || command == "run" => release = true,
            "--json" if command == "check" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown {command} option `{value}`"));
            }
            value if directory == cwd => directory = cwd.join(value),
            value => return Err(format!("unexpected {command} argument `{value}`")),
        }
        position += 1;
    }
    let manifest = load(&directory)?;
    match command {
        "check" => build::check(&directory, &manifest, json),
        "build" => build::build(&directory, &manifest, release, target.as_deref()),
        "dev" | "run" => {
            let target =
                target.unwrap_or_else(|| manifest.targets.first().cloned().unwrap_or_default());
            build::build(&directory, &manifest, release, Some(&target))?;
            build::run(&directory, &manifest, release, &target)
        }
        _ => Err(format!("unsupported standalone command `{command}`")),
    }
}

/// Builds a standalone native TSX host with the explicit automation feature.
///
/// # Errors
/// Returns an error for a missing feature, target, or failed build.
pub(crate) fn automation_binary(directory: &Path) -> Result<PathBuf, String> {
    let manifest = load(directory)?;
    if manifest.framework == "rust" || !manifest.targets.iter().any(|target| target == "native") {
        return Err("argui test requires a native Solid or React target".into());
    }
    if !manifest
        .features
        .iter()
        .any(|feature| feature == "automation")
    {
        return Err("argui test requires `--feature automation` at init".into());
    }
    build::native(directory, &manifest, false, true)?;
    Ok(build::automation_binary_path(directory, &manifest))
}

/// Resolves a project path from a command's positional arguments.
pub(crate) fn selected_path(cwd: &Path, args: &[String]) -> PathBuf {
    let mut skip = false;
    for argument in args {
        if skip {
            skip = false;
            continue;
        }
        if argument == "--target" {
            skip = true;
            continue;
        }
        if !argument.starts_with('-') && !matches!(argument.as_str(), "dev" | "release") {
            return cwd.join(argument);
        }
    }
    cwd.to_path_buf()
}
