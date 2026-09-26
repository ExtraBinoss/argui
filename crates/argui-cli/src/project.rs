//! Standalone application metadata and component adapter names.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

/// A TSX adapter supported by the component registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    Solid,
    React,
}

impl Framework {
    /// Parses a component adapter name.
    ///
    /// # Errors
    /// Returns an error when `value` is not a supported adapter.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "solid" => Ok(Self::Solid),
            "react" => Ok(Self::React),
            _ => Err("framework must be solid or react".into()),
        }
    }

    /// Returns the package and registry suffix for this adapter.
    pub fn name(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::React => "react",
        }
    }
}

/// Manifest of one self-contained Argui v2 application.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Standalone schema version; only version 2 is supported.
    pub project_version: u8,
    /// Safe application name.
    pub name: String,
    /// Rust, Solid, or React presentation source.
    pub framework: String,
    /// Native and/or Web output targets.
    pub targets: Vec<String>,
    /// Build capabilities explicitly selected by the owner.
    pub features: Vec<String>,
    /// Exact Argui crate and SDK release version.
    pub argui_version: String,
    /// Hash of the CLI-owned SDK snapshot.
    pub sdk_sha256: String,
    /// Provenance of the SDK snapshot.
    pub distribution: String,
    /// Installed component names including adapter.
    #[serde(default)]
    pub components: Vec<String>,
    /// Installed widget source paths.
    #[serde(default)]
    pub component_files: Vec<String>,
    /// Widget source checksums.
    #[serde(default)]
    pub component_checksums: BTreeMap<String, String>,
    /// Widget component versions.
    #[serde(default)]
    pub component_versions: BTreeMap<String, String>,
}

/// Reports whether `name` is a safe package and directory name.
pub fn valid_name(name: &str) -> bool {
    name.len() <= 64
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Writes `contents` to `path`, creating parent directories.
///
/// # Errors
/// Returns an error for directory creation or file writes.
pub fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, contents).map_err(|error| format!("{}: {error}", path.display()))
}
