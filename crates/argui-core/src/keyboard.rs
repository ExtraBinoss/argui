#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Character(String),
    Function(u8),
    ContextMenu,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
    Other,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    #[must_use]
    pub const fn command(self) -> bool {
        self.control || self.super_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyInput {
    pub key: Key,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImeInput {
    Enabled,
    Disabled,
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit(String),
}
