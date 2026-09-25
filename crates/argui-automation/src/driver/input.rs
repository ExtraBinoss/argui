use argui_core::Key;

/// Parses one common key spelling into an Argui keyboard key.
/// `value` is a key name or one printable character.
///
/// # Errors
/// Returns an error for unsupported key syntax.
pub(super) fn parse_key(value: &str) -> Result<Key, String> {
    Ok(match value {
        "Enter" => Key::Enter,
        "Tab" => Key::Tab,
        "Escape" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "ArrowUp" => Key::ArrowUp,
        "ArrowDown" => Key::ArrowDown,
        "ArrowLeft" => Key::ArrowLeft,
        "ArrowRight" => Key::ArrowRight,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        other if other.chars().count() == 1 => Key::Character(other.to_owned()),
        _ => {
            return Err(format!(
                "unsupported key {value:?}; use a named key or one character"
            ));
        }
    })
}
