use crate::NodeId;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FocusTarget {
    Node(NodeId),
    Key(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FocusRequest {
    Focus(FocusTarget),
    Clear,
}

impl From<NodeId> for FocusTarget {
    fn from(value: NodeId) -> Self {
        Self::Node(value)
    }
}

impl From<String> for FocusTarget {
    fn from(value: String) -> Self {
        Self::Key(value)
    }
}

impl From<&str> for FocusTarget {
    fn from(value: &str) -> Self {
        Self::Key(value.to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InitialFocus {
    First,
    Target(FocusTarget),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusContainment {
    None,
    Trap,
    Modal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusScope {
    pub containment: FocusContainment,
    pub initial: Option<InitialFocus>,
    pub restore: bool,
}

impl FocusScope {
    /// Creates a non-trapping scope that restores the previously focused node.
    #[must_use]
    pub const fn restoring() -> Self {
        Self {
            containment: FocusContainment::None,
            initial: None,
            restore: true,
        }
    }

    /// Creates a focus-trapping scope with an initial focus target.
    ///
    /// * `initial` — target selected when focus enters the scope.
    #[must_use]
    pub const fn trapped(initial: InitialFocus) -> Self {
        Self {
            containment: FocusContainment::Trap,
            initial: Some(initial),
            restore: true,
        }
    }

    /// Creates a modal focus scope with an initial focus target.
    ///
    /// * `initial` — target selected when focus enters the modal scope.
    #[must_use]
    pub const fn modal(initial: InitialFocus) -> Self {
        Self {
            containment: FocusContainment::Modal,
            initial: Some(initial),
            restore: true,
        }
    }

    /// Sets whether focus returns to its previous node when leaving the scope.
    ///
    /// * `restore` — whether the previously focused node should be restored.
    #[must_use]
    pub const fn restore(mut self, restore: bool) -> Self {
        self.restore = restore;
        self
    }

    /// Returns whether this scope traps focus within its descendants.
    #[must_use]
    pub const fn traps(&self) -> bool {
        matches!(
            self.containment,
            FocusContainment::Trap | FocusContainment::Modal
        )
    }

    /// Returns whether this scope has modal containment.
    #[must_use]
    pub const fn is_modal(&self) -> bool {
        matches!(self.containment, FocusContainment::Modal)
    }
}
