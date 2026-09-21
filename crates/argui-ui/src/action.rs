use crate::{Element, EventListener, NodeId, SelectionCommand};
use argui_core::{Key, KeyInput, KeyState, Name};

/// Stable application-defined command identity. No process-global registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActionId {
    name: Name,
}

/// Creates an action identifier from a static name.
///
/// This compatibility constructor keeps literal `ActionId("name")` call sites
/// concise while [`ActionId::from_owned`] accepts live-compiled names.
///
/// * `name` — stable application-defined action name.
#[allow(non_snake_case)]
#[must_use]
pub const fn ActionId(name: &'static str) -> ActionId {
    ActionId::new(name)
}

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
    pub const COPY: Self = Self::new("argui.copy");
    pub const CUT: Self = Self::new("argui.cut");
    pub const PASTE: Self = Self::new("argui.paste");
    pub const SELECT_ALL: Self = Self::new("argui.select-all");
    pub const UNDO: Self = Self::new("argui.undo");
    pub const REDO: Self = Self::new("argui.redo");

    /// Creates an action identifier from a static name.
    ///
    /// * `name` — stable application-defined action name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: Name::from_static(name),
        }
    }

    /// Creates an action identifier from dynamically loaded owned text.
    ///
    /// * `name` — action name whose allocation becomes shared immutable storage.
    #[must_use]
    pub fn from_owned(name: String) -> Self {
        Self {
            name: Name::from_owned(name),
        }
    }

    /// Returns the action name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the built-in text-selection command represented by this identifier.
    #[must_use]
    pub fn selection_command(&self) -> Option<SelectionCommand> {
        match self.as_str() {
            "argui.copy" => Some(SelectionCommand::Copy),
            "argui.cut" => Some(SelectionCommand::Cut),
            "argui.paste" => Some(SelectionCommand::Paste),
            "argui.select-all" => Some(SelectionCommand::SelectAll),
            "argui.undo" => Some(SelectionCommand::Undo),
            "argui.redo" => Some(SelectionCommand::Redo),
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
    /// Creates a shortcut using the platform's primary modifier and the given key.
    #[must_use]
    pub fn primary(key: impl Into<String>) -> Self {
        Self {
            key: Key::Character(key.into().to_lowercase()),
            primary: true,
            shift: false,
            alt: false,
        }
    }
    /// Adds Shift to this shortcut.
    #[must_use]
    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    /// Reports whether a key input activates this shortcut.
    ///
    /// Repeated key presses and releases do not match.
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
    /// Formats this shortcut using the platform's primary-modifier label.
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
    /// Creates an enabled action with the supplied display label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            shortcut: None,
        }
    }
    /// Sets whether this action is currently enabled.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Assigns the keyboard shortcut shown for this action.
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
            Self::DuplicateId(id) => {
                write!(formatter, "duplicate action {} in one scope", id.as_str())
            }
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
    /// Creates a scope after checking that action identifiers and shortcuts are unique.
    ///
    /// # Arguments
    ///
    /// * `bindings` — action definitions to include in this scope.
    ///
    /// # Errors
    ///
    /// Returns [`ActionError::DuplicateId`] or [`ActionError::ShortcutConflict`] when
    /// a duplicate identifier or shortcut is present.
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionInvocation {
    pub id: ActionId,
    pub origin: Option<NodeId>,
}

impl ActionInvocation {
    /// Creates an invocation without a captured origin node.
    ///
    /// * `id` — identity of the action to invoke.
    #[must_use]
    pub const fn new(id: ActionId) -> Self {
        Self { id, origin: None }
    }
    /// Captures the node from which this action was invoked.
    #[must_use]
    pub const fn at(mut self, node: NodeId) -> Self {
        self.origin = Some(node);
        self
    }
}

impl Element {
    /// Attaches a validated action scope to this element.
    ///
    /// * `scope` — action definitions available within this element's scope.
    #[must_use]
    pub fn action_scope(mut self, scope: ActionScope) -> Self {
        self.action_scope = Some(scope);
        self
    }
    /// Assigns a built-in or application-defined action to this element.
    ///
    /// * `id` — identity of the action to invoke.
    #[must_use]
    pub fn action(self, id: ActionId) -> Self {
        self.action_from(ActionInvocation::new(id))
    }
    /// Assigns an action invocation, optionally retaining its origin node.
    ///
    /// * `invocation` — action identity and captured origin, if any.
    #[must_use]
    pub fn action_from(mut self, invocation: ActionInvocation) -> Self {
        self.action = Some(invocation);
        self
    }
}
