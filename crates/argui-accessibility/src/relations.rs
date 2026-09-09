use crate::SemanticNodeId;

/// References resolved in one semantic tree; no renderer or widget keys escape here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticRelations {
    pub labelled_by: Vec<SemanticNodeId>,
    pub described_by: Vec<SemanticNodeId>,
    pub controls: Vec<SemanticNodeId>,
    pub active_descendant: Option<SemanticNodeId>,
}

impl SemanticRelations {
    pub const fn new() -> Self {
        Self {
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            active_descendant: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridPosition {
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
    /// One-based logical position, including unmounted rows and columns.
    pub row_index: Option<u32>,
    pub column_index: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupKind {
    Menu,
    ListBox,
    Tree,
    Grid,
    Dialog,
}
