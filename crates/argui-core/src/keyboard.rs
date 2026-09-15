/// Whether a key is being pressed or released.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    /// Key transitioned to down.
    Pressed,
    /// Key transitioned to up.
    Released,
}

/// Normalized keyboard key, preserving text for character keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    /// Text-producing key value.
    Character(String),
    /// Function key number, such as F1.
    Function(u8),
    /// Context-menu key.
    ContextMenu,
    /// Left arrow key.
    ArrowLeft,
    /// Right arrow key.
    ArrowRight,
    /// Up arrow key.
    ArrowUp,
    /// Down arrow key.
    ArrowDown,
    /// Page-up navigation key.
    PageUp,
    /// Page-down navigation key.
    PageDown,
    /// Home navigation key.
    Home,
    /// End navigation key.
    End,
    /// Backspace editing key.
    Backspace,
    /// Delete editing key.
    Delete,
    /// Enter/return key.
    Enter,
    /// Tab key.
    Tab,
    /// Escape key.
    Escape,
    /// Key not represented by another variant.
    Other,
}

/// Modifier keys active for a keyboard event.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    /// Shift modifier is active.
    pub shift: bool,
    /// Control modifier is active.
    pub control: bool,
    /// Alt/option modifier is active.
    pub alt: bool,
    /// Super/command/windows modifier is active.
    pub super_key: bool,
}

impl Modifiers {
    /// Returns whether the platform's command modifier is active.
    #[must_use]
    pub const fn command(self) -> bool {
        self.control || self.super_key
    }
}

/// Normalized keyboard event and any associated text input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyInput {
    /// Key that changed state.
    pub key: Key,
    /// Press or release transition.
    pub state: KeyState,
    /// Modifiers active during the event.
    pub modifiers: Modifiers,
    /// Whether this event is an auto-repeat.
    pub repeat: bool,
    /// Text produced by the event, when available.
    pub text: Option<String>,
}

/// Input method editor state or text composition update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImeInput {
    /// The input method editor became active.
    Enabled,
    /// The input method editor became inactive.
    Disabled,
    /// Current uncommitted composition and optional selection range.
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    /// Final committed text from the input method editor.
    Commit(String),
}
