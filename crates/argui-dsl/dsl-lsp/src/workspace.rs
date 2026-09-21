use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use argui_dsl_semantic::CompilerDatabase;

use crate::convert::{module_path, uri_to_path};

/// Open and on-disk DSL sources synchronized with one semantic database.
pub(crate) struct Workspace {
    root: PathBuf,
    sources: HashMap<String, String>,
    database: CompilerDatabase,
}

impl Workspace {
    /// Loads a workspace rooted at `root` and all of its `.argui` modules.
    pub(crate) fn load(root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut workspace = Self {
            root,
            sources: HashMap::new(),
            database: CompilerDatabase::with_builtins()?,
        };
        workspace.reload_disk()?;
        Ok(workspace)
    }

    /// Replaces the workspace root and reloads its source graph.
    pub(crate) fn set_root(&mut self, root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        *self = Self::load(root)?;
        Ok(())
    }

    /// Applies a full-document open/change notification.
    pub(crate) fn set_document(&mut self, uri: &str, source: String) -> Result<String, String> {
        let module = self.module_for_uri(uri)?;
        self.sources.insert(module.clone(), source.clone());
        self.database.set_file(&module, source);
        Ok(module)
    }

    /// Closes an overlay and restores its disk contents when available.
    pub(crate) fn close_document(&mut self, uri: &str) -> Result<String, String> {
        let path = uri_to_path(uri)?;
        let module = module_path(&self.root, &path)?;
        if let Ok(source) = fs::read_to_string(path) {
            self.sources.insert(module.clone(), source.clone());
            self.database.set_file(&module, source);
        } else {
            self.sources.remove(&module);
            self.database.remove_path(&module);
        }
        Ok(module)
    }

    /// Returns the canonical module key for one document URI.
    pub(crate) fn module_for_uri(&self, uri: &str) -> Result<String, String> {
        module_path(&self.root, &uri_to_path(uri)?)
    }

    /// Returns current source text for a canonical module key.
    pub(crate) fn source(&self, module: &str) -> Option<&str> {
        self.sources.get(module).map(String::as_str)
    }

    /// Returns the mutable semantic query database.
    pub(crate) const fn database(&mut self) -> &mut CompilerDatabase {
        &mut self.database
    }

    /// Returns the immutable workspace root.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Recursively reloads every disk module into a fresh semantic database.
    fn reload_disk(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        collect(&self.root, &mut files)?;
        files.sort();
        for path in files {
            let module = module_path(&self.root, &path)?;
            let source = fs::read_to_string(path)?;
            self.sources.insert(module.clone(), source.clone());
            self.database.set_file(&module, source);
        }
        Ok(())
    }
}

/// Collects project DSL files without traversing generated dependency trees.
fn collect(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if !matches!(name, ".git" | ".codex" | "target" | "node_modules") {
                collect(&path, output)?;
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "argui")
        {
            output.push(path);
        }
    }
    Ok(())
}
