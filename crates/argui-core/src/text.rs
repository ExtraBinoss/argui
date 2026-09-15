/// Side of a text boundary to which a caret is associated.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CaretAffinity {
    /// Associate with the text immediately before the boundary.
    #[default]
    Before,
    /// Associate with the text immediately after the boundary.
    After,
}

/// Text index together with its visual caret affinity.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextPosition {
    /// Character or byte index as defined by the consuming text model.
    pub index: usize,
    /// Visual side of the indexed boundary.
    pub affinity: CaretAffinity,
}

impl TextPosition {
    /// Creates a text position from an index and affinity.
    #[must_use]
    pub const fn new(index: usize, affinity: CaretAffinity) -> Self {
        Self { index, affinity }
    }
}
