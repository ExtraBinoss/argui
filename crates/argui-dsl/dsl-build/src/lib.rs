//! Cargo build-script integration for release-only Argui DSL Rust generation.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use argui_dsl_compiler::{Compiler, SourceModule};

/// Build-script compilation failure with its actionable path context.
#[derive(Debug)]
pub enum BuildError {
    MissingEnvironment(&'static str),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Compiler(argui_dsl_compiler::CompilerError),
    EntryOutsideManifest(PathBuf),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnvironment(name) => write!(formatter, "build environment lacks `{name}`"),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Compiler(error) => error.fmt(formatter),
            Self::EntryOutsideManifest(path) => write!(
                formatter,
                "entry `{}` must be inside CARGO_MANIFEST_DIR",
                path.display()
            ),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<argui_dsl_compiler::CompilerError> for BuildError {
    fn from(error: argui_dsl_compiler::CompilerError) -> Self {
        Self::Compiler(error)
    }
}

/// Compiles a project entry to `OUT_DIR/argui_ui.rs` and emits Cargo dependencies.
///
/// * `entry` — manifest-relative `.argui` entry module.
///
/// # Errors
///
/// Returns filesystem, semantic, shader, asset, or Rust-generation failures.
pub fn compile(entry: impl AsRef<Path>) -> Result<(), BuildError> {
    let manifest = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironment("CARGO_MANIFEST_DIR"))?;
    let output = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironment("OUT_DIR"))?
        .join("argui_ui.rs");
    compile_to(&manifest, entry.as_ref(), &output)
}

/// Compiles an app with the same versioned native registry installed at runtime.
///
/// * `entry` — manifest-relative DSL entry module.
/// * `registry` — built-ins plus application native adapters.
///
/// # Errors
///
/// Returns environment, source, semantic, ABI, asset, or output errors.
pub fn compile_with_registry(
    entry: impl AsRef<Path>,
    registry: argui_schema::SchemaRegistry,
) -> Result<(), BuildError> {
    let manifest = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironment("CARGO_MANIFEST_DIR"))?;
    let output = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(BuildError::MissingEnvironment("OUT_DIR"))?
        .join("argui_ui.rs");
    compile_to_with_registry(&manifest, entry.as_ref(), &output, registry)
}

/// Compiles to an explicit path, primarily for tooling and deterministic tests.
///
/// * `manifest` — project root used for canonical module and asset paths.
/// * `entry` — manifest-relative or absolute entry module.
/// * `output` — generated Rust destination.
///
/// # Errors
///
/// Returns when source discovery, compilation, validation, or atomic output fails.
pub fn compile_to(manifest: &Path, entry: &Path, output: &Path) -> Result<(), BuildError> {
    let registry =
        argui_schema::builtin::registry().map_err(argui_dsl_compiler::CompilerError::from)?;
    compile_to_with_registry(manifest, entry, output, registry)
}

/// Compiles one project with a caller-owned registry into a deterministic Rust file.
///
/// * `manifest` — project root for module and asset paths.
/// * `entry` — manifest-relative or absolute DSL entry module.
/// * `output` — generated Rust destination.
/// * `registry` — exact native contract used by AOT and live runtimes.
///
/// # Errors
///
/// Returns source, semantic, ABI, asset, or atomic output errors.
pub fn compile_to_with_registry(
    manifest: &Path,
    entry: &Path,
    output: &Path,
    registry: argui_schema::SchemaRegistry,
) -> Result<(), BuildError> {
    let entry = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        manifest.join(entry)
    };
    let entry_relative = entry
        .strip_prefix(manifest)
        .map_err(|_| BuildError::EntryOutsideManifest(entry.clone()))?;
    let source_root = entry.parent().unwrap_or(manifest);
    let mut paths = Vec::new();
    collect_argui(
        source_root,
        entry_relative.parent().unwrap_or(Path::new("")),
        &mut paths,
    )?;
    paths.sort();
    let modules = paths
        .iter()
        .map(|(path, relative)| {
            let source = fs::read_to_string(path).map_err(|source| BuildError::Io {
                path: path.clone(),
                source,
            })?;
            println!("cargo:rerun-if-changed={}", path.display());
            Ok(SourceModule::new(normalize(relative), source))
        })
        .collect::<Result<Vec<_>, BuildError>>()?;
    let compiled =
        Compiler::compile_with_registry(modules, &normalize(entry_relative), registry, |asset| {
            fs::read(manifest.join(asset)).map_err(|error| error.to_string())
        })?;
    for dependency in &compiled.dependencies {
        if dependency.starts_with('@') {
            continue;
        }
        println!(
            "cargo:rerun-if-changed={}",
            manifest.join(dependency).display()
        );
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|source| BuildError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let temporary = output.with_extension("rs.tmp");
    fs::write(&temporary, compiled.rust).map_err(|source| BuildError::Io {
        path: temporary.clone(),
        source,
    })?;
    if let Err(source) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(BuildError::Io {
            path: output.to_path_buf(),
            source,
        });
    }
    Ok(())
}

/// Discovers `.argui` modules and their manifest-relative keys without following symlink directories.
///
/// `directory` is the current absolute directory, `relative_directory` is its
/// manifest-relative counterpart, and `output` receives matching path pairs.
///
/// # Errors
///
/// Returns a filesystem error when a directory entry cannot be read or typed.
fn collect_argui(
    directory: &Path,
    relative_directory: &Path,
    output: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), BuildError> {
    let entries = fs::read_dir(directory).map_err(|source| BuildError::Io {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| BuildError::Io {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| BuildError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() && !file_type.is_symlink() {
            collect_argui(&path, &relative_directory.join(entry.file_name()), output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "argui") {
            output.push((path, relative_directory.join(entry.file_name())));
        }
    }
    Ok(())
}

/// Converts a platform path to the compiler's slash-separated module key.
fn normalize(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
