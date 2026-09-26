//! Standalone application manifests and CLI routing.

mod build;
mod format;
mod init;
mod menu;

pub(crate) use crate::project::Project as Manifest;
use std::{
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
};

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
    let mut install_dependencies = true;
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
            "--no-install" => install_dependencies = false,
            unknown => {
                return Err(format!(
                    "unknown init argument `{unknown}`; use --dir, --targets, --feature, --yes, or --no-install"
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
    let framework = framework
        .or_else(|| yes.then(|| "solid".to_owned()))
        .ok_or(
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
        install_dependencies,
    };
    init::create(&options)
}

/// Routes a check, format, build, dev, or run command for a standalone project.
///
/// # Errors
/// Returns an error for invalid flags or failed build processes.
pub(crate) fn command(cwd: &Path, command: &str, args: &[String]) -> Result<(), String> {
    let mut directory = cwd.to_path_buf();
    let mut target = None;
    let mut release = false;
    let mut json = false;
    let mut check_format = false;
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
            "--check" if command == "format" => check_format = true,
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
        "format" if target.is_none() => format::run(&directory, &manifest, check_format),
        "format" => Err("format does not accept --target".into()),
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
