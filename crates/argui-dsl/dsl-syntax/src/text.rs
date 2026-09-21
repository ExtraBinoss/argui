use rowan::TextRange;

/// Stable source-file identity assigned by the semantic database.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileId(u32);

impl FileId {
    /// Creates a file identity from its database-local numeric value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the database-local numeric value.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Exact byte range associated with syntax or semantic data in one source file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub file: FileId,
    pub range: TextRange,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Span {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(
            &(
                self.file,
                u32::from(self.range.start()),
                u32::from(self.range.end()),
            ),
            serializer,
        )
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Span {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (file, start, end) = <(FileId, u32, u32)>::deserialize(deserializer)?;
        Ok(Self::new(file, TextRange::new(start.into(), end.into())))
    }
}

impl Span {
    /// Creates a source span.
    ///
    /// * `file` — owning source file.
    /// * `range` — UTF-8 byte range inside the file.
    #[must_use]
    pub const fn new(file: FileId, range: TextRange) -> Self {
        Self { file, range }
    }
}
