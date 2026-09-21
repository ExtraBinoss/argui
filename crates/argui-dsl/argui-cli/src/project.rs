use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use argui_dsl_compiler::SourceModule;

/// Complete deterministic `.argui` source snapshot below one project root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectFiles {
    pub root: PathBuf,
    pub modules: Vec<SourceModule>,
}

impl ProjectFiles {
    /// Reads every project module while excluding build and VCS directories.
    ///
    /// # Errors
    ///
    /// Returns the first directory traversal or UTF-8 source read failure.
    pub fn read(root: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let root = root.into();
        let mut paths = Vec::new();
        collect_modules(&root, &root, &mut paths)?;
        paths.sort();
        let modules = paths
            .into_iter()
            .map(|path| {
                let module = canonical_relative(&root, &path)?;
                let source = fs::read_to_string(&path)?;
                Ok(SourceModule::new(module, source))
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        Ok(Self { root, modules })
    }

    /// Loads this immutable filesystem snapshot into a semantic query database.
    ///
    /// # Errors
    ///
    /// Returns an error if Argui's built-in native schema is internally invalid.
    pub fn semantic_database(
        &self,
    ) -> Result<argui_dsl_semantic::CompilerDatabase, argui_schema::SchemaError> {
        let mut database = argui_dsl_semantic::CompilerDatabase::with_builtins()?;
        for module in &self.modules {
            database.set_file(&module.path, module.source.clone());
        }
        Ok(database)
    }
}

/// Converts a path below `root` to a slash-separated lexical module path.
///
/// # Errors
///
/// Returns invalid-input when the path escapes the project or is not UTF-8.
pub fn canonical_relative(root: &Path, path: &Path) -> Result<String, std::io::Error> {
    let relative = path.strip_prefix(root).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is outside project root",
        )
    })?;
    let mut output = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => output.push(value.to_str().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "path is not UTF-8")
            })?),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "path is not project-relative",
                ));
            }
        }
    }
    Ok(output.join("/"))
}

/// Recursively collects modules without entering generated or VCS directories.
fn collect_modules(
    root: &Path,
    directory: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if !matches!(name, ".git" | ".codex" | "target" | "node_modules") {
                collect_modules(root, &path, output)?;
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "argui")
        {
            let _ = path.strip_prefix(root).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "module escaped project")
            })?;
            output.push(path);
        }
    }
    Ok(())
}
