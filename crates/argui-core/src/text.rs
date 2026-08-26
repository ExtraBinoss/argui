#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CaretAffinity {
    #[default]
    Before,
    After,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextPosition {
    pub index: usize,
    pub affinity: CaretAffinity,
}

impl TextPosition {
    #[must_use]
    pub const fn new(index: usize, affinity: CaretAffinity) -> Self {
        Self { index, affinity }
    }
}
