use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{canonical_relative, is_relevant_path};

/// One real source-content change, not merely a filesystem notification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceChange {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub before: String,
    pub after: String,
}

/// Incremental source snapshots used to suppress duplicate watcher generations.
pub struct ChangeTracker {
    root: PathBuf,
    snapshots: HashMap<PathBuf, Vec<u8>>,
}

impl ChangeTracker {
    /// Snapshots relevant source files below `root` before watching begins.
    ///
    /// # Errors
    /// Returns a filesystem read or traversal failure.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        let mut snapshots = HashMap::new();
        collect(&root, &mut snapshots)?;
        Ok(Self { root, snapshots })
    }

    /// Compares notified paths with actual bytes and returns only changed files.
    ///
    /// `paths` are absolute or project-relative watcher paths. Added, modified,
    /// and removed files are reported in deterministic path order.
    ///
    /// # Errors
    /// Returns a source read failure other than a missing file.
    pub fn refresh(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> io::Result<Vec<SourceChange>> {
        let mut paths = paths
            .into_iter()
            .map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    self.root.join(path)
                }
            })
            .filter(|path| path.starts_with(&self.root) && is_relevant_path(path))
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        let mut changes = Vec::new();
        for path in paths {
            let relative = canonical_relative(&self.root, &path)?;
            let next = match fs::read(&path) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(error),
            };
            let previous = self.snapshots.get(&path);
            if previous.map(Vec::as_slice) == next.as_deref() {
                continue;
            }
            changes.push(describe(
                relative,
                previous.map(Vec::as_slice),
                next.as_deref(),
            ));
            if let Some(bytes) = next {
                self.snapshots.insert(path, bytes);
            } else {
                self.snapshots.remove(&path);
            }
        }
        Ok(changes)
    }
}

/// Reads relevant files while excluding generated and version-control trees.
fn collect(directory: &Path, snapshots: &mut HashMap<PathBuf, Vec<u8>>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if !matches!(name, ".git" | ".codex" | "target" | "node_modules") {
                collect(&path, snapshots)?;
            }
        } else if file_type.is_file() && is_relevant_path(&path) {
            snapshots.insert(path.clone(), fs::read(path)?);
        }
    }
    Ok(())
}

/// Summarizes the first changed Unicode scalar in text or byte counts in binary assets.
fn describe(path: String, previous: Option<&[u8]>, next: Option<&[u8]>) -> SourceChange {
    if let (Some(before), Some(after)) = (
        previous.map_or(Some(""), |bytes| std::str::from_utf8(bytes).ok()),
        next.map_or(Some(""), |bytes| std::str::from_utf8(bytes).ok()),
    ) {
        let mut line = 1;
        let mut column = 1;
        for (left, right) in before.chars().zip(after.chars()) {
            if left != right {
                break;
            }
            if left == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        return SourceChange {
            path,
            line: Some(line),
            column: Some(column),
            before: if previous.is_some() {
                changed_line(before, line)
            } else {
                "∅".into()
            },
            after: if next.is_some() {
                changed_line(after, line)
            } else {
                "∅".into()
            },
        };
    }
    SourceChange {
        path,
        line: None,
        column: None,
        before: bytes_summary(previous),
        after: bytes_summary(next),
    }
}

/// Returns a bounded display line without altering the stored source snapshot.
fn changed_line(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line - 1)
        .unwrap_or("∅")
        .chars()
        .take(120)
        .collect()
}

/// Formats an absent file or binary content size and stable fingerprint.
fn bytes_summary(bytes: Option<&[u8]>) -> String {
    let Some(bytes) = bytes else {
        return "∅".into();
    };
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{} bytes · {hash:016x}", bytes.len())
}
