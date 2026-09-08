use crate::{Element, EventListener, NodeId, SelectionCommand};
use argui_core::{Key, KeyInput, KeyState};

/// Stable application-defined command identity. No process-global registry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ActionId(pub &'static str);

impl From<SelectionCommand> for ActionId {
    fn from(command: SelectionCommand) -> Self {
        match command {
            SelectionCommand::Copy => Self::COPY,
            SelectionCommand::Cut => Self::CUT,
            SelectionCommand::Paste => Self::PASTE,
            SelectionCommand::SelectAll => Self::SELECT_ALL,
            SelectionCommand::Undo => Self::UNDO,
            SelectionCommand::Redo => Self::REDO,
        }
    }
}

impl ActionId {
    pub const COPY: Self = Self("argui.copy");
    pub const CUT: Self = Self("argui.cut");
    pub const PASTE: Self = Self("argui.paste");
    pub const SELECT_ALL: Self = Self("argui.select-all");
    pub const UNDO: Self = Self("argui.undo");
    pub const REDO: Self = Self("argui.redo");
    #[must_use]
    pub fn selection_command(self) -> Option<SelectionCommand> {
        match self {
            Self::COPY => Some(SelectionCommand::Copy),
            Self::CUT => Some(SelectionCommand::Cut),
            Self::PASTE => Some(SelectionCommand::Paste),
            Self::SELECT_ALL => Some(SelectionCommand::SelectAll),
            Self::UNDO => Some(SelectionCommand::Undo),
            Self::REDO => Some(SelectionCommand::Redo),
            _ => None,
        }
    }
}

/// One key combination; `primary` means Command on macOS and Control elsewhere.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shortcut {
    pub key: Key,
    pub primary: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Shortcut {
    #[must_use]
    pub fn primary(key: impl Into<String>) -> Self {
        Self {
            key: Key::Character(key.into().to_lowercase()),
            primary: true,
            shift: false,
            alt: false,
        }
    }
    #[must_use]
    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    #[must_use]
    pub fn matches(&self, input: &KeyInput) -> bool {
        let key = match &input.key {
            Key::Character(value) => Key::Character(value.to_lowercase()),
            key => key.clone(),
        };
        input.state == KeyState::Pressed
            && !input.repeat
            && self.key == key
            && self.primary == input.modifiers.command()
            && self.shift == input.modifiers.shift
            && self.alt == input.modifiers.alt
    }
    #[must_use]
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.primary {
            parts.push(if cfg!(target_os = "macos") {
                "⌘".into()
            } else {
                "Ctrl".into()
            });
        }
        if self.shift {
            parts.push("Shift".into());
        }
        if self.alt {
            parts.push("Alt".into());
        }
        parts.push(match &self.key {
            Key::Character(key) => key.to_uppercase(),
            key => format!("{key:?}"),
        });
        parts.join("+")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionState {
    pub label: String,
    pub enabled: bool,
    pub shortcut: Option<Shortcut>,
}

impl ActionState {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            shortcut: None,
        }
    }
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    #[must_use]
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionBinding {
    pub id: ActionId,
    pub state: ActionState,
    pub listener: EventListener,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionError {
    DuplicateId(ActionId),
    ShortcutConflict(Shortcut),
    Unavailable,
    StaleOrigin,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(formatter, "duplicate action {} in one scope", id.0),
            Self::ShortcutConflict(shortcut) => write!(
                formatter,
                "conflicting shortcut {} in one scope",
                shortcut.label()
            ),
            Self::Unavailable => formatter.write_str("action is unavailable in this scope"),
            Self::StaleOrigin => formatter.write_str("action origin is no longer mounted"),
        }
    }
}
impl std::error::Error for ActionError {}

/// A validated scope. Duplicate commands/shortcuts are rejected, not ordered arbitrarily.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionScope {
    pub(crate) bindings: Vec<ActionBinding>,
}

impl ActionScope {
    pub fn new(bindings: impl IntoIterator<Item = ActionBinding>) -> Result<Self, ActionError> {
        let mut scope = Self::default();
        for binding in bindings {
            if scope.bindings.iter().any(|old| old.id == binding.id) {
                return Err(ActionError::DuplicateId(binding.id));
            }
            if let Some(shortcut) = &binding.state.shortcut
                && scope
                    .bindings
                    .iter()
                    .any(|old| old.state.shortcut.as_ref() == Some(shortcut))
            {
                return Err(ActionError::ShortcutConflict(shortcut.clone()));
            }
            scope.bindings.push(binding);
        }
        Ok(scope)
    }
}

/// Capture `origin` before opening a menu/palette. Removed nodes fail closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionInvocation {
    pub id: ActionId,
    pub origin: Option<NodeId>,
}

impl ActionInvocation {
    #[must_use]
    pub const fn new(id: ActionId) -> Self {
        Self { id, origin: None }
    }
    #[must_use]
    pub const fn at(mut self, node: NodeId) -> Self {
        self.origin = Some(node);
        self
    }
}

impl Element {
    #[must_use]
    pub fn action_scope(mut self, scope: ActionScope) -> Self {
        self.action_scope = Some(scope);
        self
    }
    #[must_use]
    pub fn action(self, id: ActionId) -> Self {
        self.action_from(ActionInvocation::new(id))
    }
    #[must_use]
    pub fn action_from(mut self, invocation: ActionInvocation) -> Self {
        self.action = Some(invocation);
        self
    }
}
