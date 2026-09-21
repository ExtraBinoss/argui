use std::{borrow::Borrow, fmt, hash::Hash, sync::Arc};

/// An immutable engine-level name that accepts static and dynamically loaded text.
///
/// Static names do not allocate. Owned names use shared immutable storage, so
/// cloning a name is constant-time and does not require a process-global
/// interner. Equality and hashing depend only on the string contents.
#[derive(Clone)]
pub struct Name(Storage);

#[derive(Clone)]
enum Storage {
    Static(&'static str),
    Shared(Arc<str>),
}

impl Name {
    /// Creates a name backed directly by a static string.
    ///
    /// * `value` — immutable name text embedded in the program.
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Storage::Static(value))
    }

    /// Creates a name that owns dynamically loaded text.
    ///
    /// * `value` — name text whose allocation becomes shared immutable storage.
    #[must_use]
    pub fn from_owned(value: String) -> Self {
        Self(Storage::Shared(Arc::from(value)))
    }

    /// Returns the name text independently of its storage origin.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Storage::Static(value) => value,
            Storage::Shared(value) => value,
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Name").field(&self.as_str()).finish()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&'static str> for Name {
    fn from(value: &'static str) -> Self {
        Self::from_static(value)
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self::from_owned(value)
    }
}

impl From<Arc<str>> for Name {
    fn from(value: Arc<str>) -> Self {
        Self(Storage::Shared(value))
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Name {}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Name {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
