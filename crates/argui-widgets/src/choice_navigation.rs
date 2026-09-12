#[cfg(any(
    feature = "accordion",
    feature = "toggle-group",
    feature = "combobox",
    feature = "navigation-menu"
))]
use argui_core::{Key, KeyState};
#[cfg(any(
    feature = "accordion",
    feature = "toggle-group",
    feature = "combobox",
    feature = "navigation-menu"
))]
use argui_ui::{Orientation, UiEvent, UiEventKind};

/// Selection cardinality for controlled groups.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChoiceMode {
    #[default]
    Single,
    Multiple,
}

#[cfg(any(
    feature = "accordion",
    feature = "toggle-group",
    feature = "combobox",
    feature = "navigation-menu"
))]
pub(crate) fn navigate(
    event: &UiEvent,
    current: usize,
    count: usize,
    orientation: Orientation,
    rtl: bool,
    enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    let UiEventKind::KeyInput(input) = &event.kind else {
        return None;
    };
    if input.state != KeyState::Pressed
        || input.modifiers.command()
        || input.modifiers.alt
        || count == 0
    {
        return None;
    }
    let backwards = match orientation {
        Orientation::Vertical => Key::ArrowUp,
        Orientation::Horizontal if rtl => Key::ArrowRight,
        Orientation::Horizontal => Key::ArrowLeft,
    };
    let forwards = match orientation {
        Orientation::Vertical => Key::ArrowDown,
        Orientation::Horizontal if rtl => Key::ArrowLeft,
        Orientation::Horizontal => Key::ArrowRight,
    };
    match &input.key {
        Key::Home => (0..count).find(|index| enabled(*index)),
        Key::End => (0..count).rev().find(|index| enabled(*index)),
        key if *key == forwards || *key == backwards => (1..=count)
            .map(|offset| {
                if *key == forwards {
                    (current + offset) % count
                } else {
                    (current + count - offset) % count
                }
            })
            .find(|index| enabled(*index)),
        _ => None,
    }
}
