use std::sync::Arc;

use crate::AssetRegistryError;

/// Canonical, platform-independent identity of an imported source asset.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetKey(Arc<str>);

impl AssetKey {
    /// Canonicalizes a logical import path into a stable asset key.
    ///
    /// Backslashes become forward slashes and redundant `.` or empty segments
    /// are removed. Parent traversal is rejected so two project roots cannot
    /// accidentally alias the same key.
    ///
    /// * `source` — project-relative logical source path.
    ///
    /// # Errors
    ///
    /// Returns an error for empty paths, absolute paths, NUL bytes, or `..` traversal.
    pub fn new(source: impl AsRef<str>) -> Result<Self, AssetRegistryError> {
        let source = source.as_ref();
        if source.is_empty()
            || source.starts_with(['/', '\\'])
            || source.contains('\0')
            || has_windows_prefix(source)
        {
            return Err(AssetRegistryError::InvalidKey(source.into()));
        }
        let normalized = source.replace('\\', "/");
        let mut segments = Vec::new();
        for segment in normalized.split('/') {
            match segment {
                "" | "." => {}
                ".." => return Err(AssetRegistryError::InvalidKey(source.into())),
                value => segments.push(value),
            }
        }
        if segments.is_empty() {
            return Err(AssetRegistryError::InvalidKey(source.into()));
        }
        Ok(Self(segments.join("/").into()))
    }

    /// Returns the canonical logical source path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AssetKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Monotonic content revision of one stable asset key.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetRevision(u64);

impl AssetRevision {
    pub(crate) const INITIAL: Self = Self(1);

    /// Returns the numeric revision used by development protocols and caches.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self, key: &AssetKey) -> Result<Self, AssetRegistryError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| AssetRegistryError::RevisionExhausted(key.clone()))
    }
}

fn has_windows_prefix(source: &str) -> bool {
    let bytes = source.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
