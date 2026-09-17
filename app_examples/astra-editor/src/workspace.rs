use std::{collections::BTreeMap, path::PathBuf};

mod document;
pub use document::Document;

#[cfg(not(target_arch = "wasm32"))]
const MAX_PROJECT_FILES: usize = 8_000;
#[cfg(not(target_arch = "wasm32"))]
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

/// Kind and optional document target represented by one project-tree row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A collapsible directory row.
    Directory,
    /// An editable file with its document index.
    File(usize),
}

/// One preorder row in the project navigator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectEntry {
    /// Stable path key shared with the tree widget.
    pub key: String,
    /// Final path component displayed to the user.
    pub label: String,
    /// Zero-based nesting level.
    pub depth: usize,
    /// Directory or document target represented by this row.
    pub kind: EntryKind,
}

/// A single line match returned by the in-memory project index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextMatch {
    /// Matching document index.
    pub document: usize,
    /// One-based source line.
    pub line: usize,
    /// Trimmed line preview.
    pub preview: String,
}

/// Searchable project state shared by the explorer, tabs, and editor.
#[derive(Clone, Debug)]
pub struct Project {
    /// Short root name shown in the toolbar.
    pub name: String,
    /// Native project root, absent for the bundled and browser workspaces.
    pub root: Option<PathBuf>,
    /// Preorder tree rows.
    pub entries: Vec<ProjectEntry>,
    /// Editable text documents.
    pub documents: Vec<Document>,
}

impl Project {
    /// Creates the instant-start bundled Rust workspace used on every platform.
    #[must_use]
    pub fn demo() -> Self {
        Self::from_documents(
            "hello-argui",
            [
                (
                    "src/main.rs".into(),
                    r#"use argui::{prelude::*, widgets::{Button, shadcn}};

#[derive(Default)]
struct Counter {
    value: u32,
}

impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            Element::text(format!("Count: {}", self.value)),
            Button::new("increment", "Increment", theme.button())
                .on_click(cx.callback(|app| app.value += 1))
                .build(),
        ])
        .gap(12.0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    argui::runtime::run_app(Counter::default())?;
    Ok(())
}
"#
                    .into(),
                ),
                (
                    "src/theme.rs".into(),
                    r#"use argui::core::Color;

pub const ACCENT: Color = Color::from_srgb8(43, 110, 242);
pub const EDITOR_RADIUS: f32 = 12.0;
"#
                    .into(),
                ),
                (
                    "tests/counter.rs".into(),
                    r#"use argui_testing::TestApp;

#[test]
fn increments_from_the_real_button() {
    let mut app = TestApp::new(Counter::default());
    app.click("increment").unwrap();
    app.assert_text("Count: 1");
}
"#
                    .into(),
                ),
                (
                    "Cargo.toml".into(),
                    r#"[package]
name = "hello-argui"
version = "0.1.0"
edition = "2024"

[dependencies]
argui = { version = "0.3", features = ["widget-button"] }
"#
                    .into(),
                ),
                (
                    "README.md".into(),
                    "# Hello Argui\n\nA tiny, native and WebAssembly counter application.\n".into(),
                ),
            ],
        )
    }

    /// Builds a project from slash-separated in-memory files.
    ///
    /// `name` is the project label and `files` contains relative paths with UTF-8 text.
    #[must_use]
    pub fn from_documents(
        name: impl Into<String>,
        files: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self::from_documents_inner(name, files, true)
    }

    /// Builds imported project state and defers syntax work until documents are opened.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_imported_documents(
        name: impl Into<String>,
        files: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self::from_documents_inner(name, files, false)
    }

    /// Builds project state while deferring syntax work until a file is opened.
    fn from_documents_inner(
        name: impl Into<String>,
        files: impl IntoIterator<Item = (String, String)>,
        highlight: bool,
    ) -> Self {
        let mut root = VirtualNode::default();
        let mut documents = Vec::new();
        for (path, content) in files {
            let path = normalize_path(&path);
            if path.is_empty() {
                continue;
            }
            let id = documents.len();
            documents.push(Document::new(id, path.clone(), content, highlight));
            root.insert(&path, id);
        }
        let mut entries = Vec::new();
        root.emit("", 0, &mut entries);
        Self {
            name: name.into(),
            root: None,
            entries,
            documents,
        }
    }

    /// Returns the document represented by a tree key, if the row is a file.
    #[must_use]
    pub fn document_for_key(&self, key: &str) -> Option<usize> {
        self.entries.iter().find_map(|entry| {
            (entry.key == key)
                .then_some(entry.kind)
                .and_then(|kind| match kind {
                    EntryKind::File(document) => Some(document),
                    EntryKind::Directory => None,
                })
        })
    }

    /// Reports whether a tree key represents a directory.
    #[must_use]
    pub fn is_directory(&self, key: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.key == key && entry.kind == EntryKind::Directory)
    }

    /// Searches cached lowercase document text and returns at most `limit` line matches.
    #[must_use]
    pub fn search(&self, query: &str, limit: usize) -> Vec<TextMatch> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let mut matches = Vec::new();
        for document in &self.documents {
            let searchable_content = document.searchable_content();
            for (index, (line, searchable)) in document
                .content
                .lines()
                .zip(searchable_content.lines())
                .enumerate()
            {
                if searchable.contains(&query) {
                    matches.push(TextMatch {
                        document: document.id,
                        line: index + 1,
                        preview: line.trim().to_owned(),
                    });
                    if matches.len() == limit {
                        return matches;
                    }
                }
            }
        }
        matches
    }

    /// Returns document indices whose paths contain `query`, capped at `limit`.
    #[must_use]
    pub fn matching_files(&self, query: &str, limit: usize) -> Vec<usize> {
        let query = query.trim().to_lowercase();
        self.documents
            .iter()
            .filter(|document| query.is_empty() || document.path.to_lowercase().contains(&query))
            .take(limit)
            .map(|document| document.id)
            .collect()
    }

    /// Loads all supported UTF-8 text files below a native directory.
    ///
    /// `root` is the selected project folder and `cancelled` is checked between entries.
    ///
    /// # Errors
    ///
    /// Returns an error when the root cannot be read or does not contain readable text files.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path(
        root: PathBuf,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, ProjectLoadError> {
        let metadata = std::fs::metadata(&root).map_err(ProjectLoadError::ReadRoot)?;
        if !metadata.is_dir() {
            return Err(ProjectLoadError::NotDirectory(root));
        }
        let mut files = Vec::new();
        collect_files(&root, &root, &cancelled, &mut files)?;
        if files.is_empty() {
            return Err(ProjectLoadError::NoTextFiles);
        }
        let name = root.file_name().map_or_else(
            || root.display().to_string(),
            |name| name.to_string_lossy().into(),
        );
        let mut project = Self::from_documents_inner(name, files, false);
        for document in &mut project.documents {
            document.absolute_path = Some(root.join(&document.path));
        }
        project.root = Some(root);
        Ok(project)
    }
}

/// Failure to load a selected native project folder.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum ProjectLoadError {
    /// The selected path could not be inspected.
    ReadRoot(std::io::Error),
    /// A directory in the selected tree could not be enumerated.
    ReadDirectory(std::io::Error),
    /// The selected path is not a directory.
    NotDirectory(PathBuf),
    /// No supported UTF-8 text file was found.
    NoTextFiles,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for ProjectLoadError {
    /// Formats a user-facing project-loading diagnostic.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadRoot(error) => write!(formatter, "could not inspect the folder: {error}"),
            Self::ReadDirectory(error) => {
                write!(formatter, "could not read part of the project: {error}")
            }
            Self::NotDirectory(path) => write!(formatter, "{} is not a folder", path.display()),
            Self::NoTextFiles => formatter.write_str("no supported UTF-8 text files were found"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for ProjectLoadError {}

#[derive(Default)]
struct VirtualNode {
    document: Option<usize>,
    children: BTreeMap<String, VirtualNode>,
}

impl VirtualNode {
    /// Inserts one slash-separated document path into the temporary tree.
    fn insert(&mut self, path: &str, document: usize) {
        let mut node = self;
        for part in path.split('/').filter(|part| !part.is_empty()) {
            node = node.children.entry(part.to_owned()).or_default();
        }
        node.document = Some(document);
    }

    /// Emits preorder directory and file rows below `prefix` at `depth`.
    fn emit(&self, prefix: &str, depth: usize, output: &mut Vec<ProjectEntry>) {
        for (name, node) in &self.children {
            if node.document.is_some() && node.children.is_empty() {
                continue;
            }
            let path = join_path(prefix, name);
            output.push(ProjectEntry {
                key: path.clone(),
                label: name.clone(),
                depth,
                kind: EntryKind::Directory,
            });
            node.emit(&path, depth + 1, output);
        }
        for (name, node) in &self.children {
            let Some(document) = node.document else {
                continue;
            };
            let path = join_path(prefix, name);
            output.push(ProjectEntry {
                key: path,
                label: name.clone(),
                depth,
                kind: EntryKind::File(document),
            });
        }
    }
}

/// Normalizes a user- or browser-supplied relative path to slash separators.
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/")
}

/// Joins a normalized project prefix and one child name.
fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}/{name}")
    }
}

/// Recursively collects supported UTF-8 files while honoring project limits.
#[cfg(not(target_arch = "wasm32"))]
fn collect_files(
    root: &std::path::Path,
    directory: &std::path::Path,
    cancelled: &impl Fn() -> bool,
    output: &mut Vec<(String, String)>,
) -> Result<(), ProjectLoadError> {
    if cancelled() || output.len() >= MAX_PROJECT_FILES {
        return Ok(());
    }
    let mut entries = std::fs::read_dir(directory)
        .map_err(ProjectLoadError::ReadDirectory)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in entries {
        if cancelled() || output.len() >= MAX_PROJECT_FILES {
            break;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() || ignored_name(&name) {
            continue;
        }
        if kind.is_dir() {
            collect_files(root, &path, cancelled, output)?;
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !kind.is_file() || metadata.len() > MAX_FILE_BYTES || !supported_file(&path) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(content) = String::from_utf8(bytes) else {
            continue;
        };
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        output.push((normalize_path(&relative.to_string_lossy()), content));
    }
    Ok(())
}

/// Returns whether a directory name should be pruned from project discovery.
#[cfg(not(target_arch = "wasm32"))]
fn ignored_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".idea" | ".vscode" | "target" | "node_modules" | "dist" | "build"
    )
}

/// Returns whether a path is expected to contain editable project text.
#[cfg(not(target_arch = "wasm32"))]
fn supported_file(path: &std::path::Path) -> bool {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        extension.as_str(),
        "rs" | "toml"
            | "md"
            | "txt"
            | "json"
            | "yaml"
            | "yml"
            | "ron"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "css"
            | "html"
            | "htm"
            | "wgsl"
            | "sh"
            | "bash"
            | "zsh"
            | "py"
            | "go"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
    ) || path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, "Makefile" | "Dockerfile" | "LICENSE" | "NOTICE"))
}
