use std::{borrow::Cow, path::PathBuf};

use argui::ui::{TextEdit, TextEditError};

use crate::syntax::{self, HighlightedCode};

/// One editable text document in the active workspace.
#[derive(Clone, Debug)]
pub struct Document {
    /// Stable index used by tabs and search results.
    pub id: usize,
    /// Slash-separated path relative to the project root.
    pub path: String,
    /// Current controlled editor value.
    pub content: String,
    saved_content: String,
    dirty: bool,
    pub(super) search_content: String,
    revision: u64,
    search_revision: u64,
    highlight_revision: Option<u64>,
    highlighted: Option<HighlightedCode>,
    /// Native destination when this document came from a folder scan.
    pub absolute_path: Option<PathBuf>,
}

impl Document {
    /// Creates a document and optionally computes its initial syntax styling.
    pub(super) fn new(id: usize, path: String, content: String, highlight: bool) -> Self {
        let highlighted = highlight
            .then(|| syntax::highlight(&path, &content))
            .flatten();
        Self {
            id,
            path,
            search_content: content.to_lowercase(),
            saved_content: content.clone(),
            dirty: false,
            revision: 0,
            search_revision: 0,
            highlight_revision: highlight.then_some(0),
            highlighted,
            content,
            absolute_path: None,
        }
    }

    /// Returns the final path component displayed in tabs.
    #[must_use]
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// Returns the lowercase extension used for the editor language label.
    #[must_use]
    pub fn extension(&self) -> &str {
        self.name()
            .rsplit_once('.')
            .map_or("text", |(_, extension)| extension)
    }

    /// Returns a concise uppercase language label for the status bar.
    #[must_use]
    pub fn language_label(&self) -> &'static str {
        match self.extension() {
            "rs" => "RUST",
            "toml" => "TOML",
            "md" => "MARKDOWN",
            "json" => "JSON",
            "js" | "jsx" => "JAVASCRIPT",
            "ts" | "tsx" => "TYPESCRIPT",
            "css" => "CSS",
            "html" => "HTML",
            "yaml" | "yml" => "YAML",
            _ => "PLAIN TEXT",
        }
    }

    /// Returns the current number of logical lines, including one empty line for an empty file.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    /// Reports whether the controlled value differs from the last saved snapshot.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Applies one accepted editor replacement and invalidates derived data.
    ///
    /// Returns an error if `edit` does not address valid UTF-8 boundaries in the
    /// document's current revision. The document remains unchanged on error.
    pub fn apply_edit(&mut self, edit: &TextEdit) -> Result<(), TextEditError> {
        edit.apply_to(&mut self.content)?;
        self.revision = self.revision.wrapping_add(1);
        self.dirty =
            self.content.len() != self.saved_content.len() || self.content != self.saved_content;
        Ok(())
    }

    /// Returns an owned snapshot suitable for deferred derived-data work.
    pub(crate) fn derived_snapshot(&self) -> (u64, String, String) {
        (self.revision, self.path.clone(), self.content.clone())
    }

    /// Applies derived data only when `revision` still identifies the current text.
    pub(crate) fn apply_derived(
        &mut self,
        revision: u64,
        search_content: String,
        highlighted: Option<HighlightedCode>,
    ) -> bool {
        if revision != self.revision {
            return false;
        }
        self.search_content = search_content;
        self.search_revision = revision;
        self.highlighted = highlighted;
        self.highlight_revision = Some(revision);
        true
    }

    /// Reports whether search or syntax data still needs refreshing.
    pub(crate) fn needs_derived_refresh(&self) -> bool {
        self.search_revision != self.revision || self.highlight_revision != Some(self.revision)
    }

    /// Returns lowercase text from the cache or derives it for an outstanding edit.
    pub(super) fn searchable_content(&self) -> Cow<'_, str> {
        if self.search_revision == self.revision {
            Cow::Borrowed(&self.search_content)
        } else {
            Cow::Owned(self.content.to_lowercase())
        }
    }

    /// Returns cached rich source text for the requested editor theme when current.
    #[must_use]
    pub fn highlighted_content(&self, dark: bool) -> Option<argui::text::TextContent> {
        (self.highlight_revision == Some(self.revision))
            .then(|| {
                self.highlighted
                    .as_ref()
                    .map(|content| content.content(dark))
            })
            .flatten()
    }

    /// Marks the current value as the last saved snapshot.
    pub fn mark_saved(&mut self) {
        self.saved_content.clone_from(&self.content);
        self.dirty = false;
    }

    /// Marks `content` saved only when it is still the current value.
    ///
    /// This prevents an asynchronous native save from clearing the dirty marker after
    /// the user has already typed something newer.
    pub fn mark_snapshot_saved(&mut self, content: &str) {
        if self.content == content {
            self.saved_content = content.to_owned();
            self.dirty = false;
        }
    }
}
