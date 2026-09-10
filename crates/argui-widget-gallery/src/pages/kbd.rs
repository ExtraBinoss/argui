use crate::app::text;
use argui::{
    ui::{AlignItems, Element, JustifyContent, length},
    widgets::{Kbd, Separator, WidgetTheme},
};

pub(super) fn render(theme: &WidgetTheme) -> Element {
    super::preview(
        "Keyboard shortcuts",
        "Show a key or a complete combination alongside its action.",
        Element::column([
            row(
                "Search components",
                Kbd::new("kbd-search", ["Ctrl", "K"]).build(theme),
                theme,
            ),
            Separator::new("kbd-divider").build(theme),
            row(
                "Change theme",
                Kbd::new("kbd-theme", ["Ctrl", "Shift", "L"]).build(theme),
                theme,
            ),
            row(
                "Confirm",
                Kbd::new("kbd-enter", ["Enter"]).build(theme),
                theme,
            ),
            row(
                "Dismiss",
                Kbd::new("kbd-escape", ["Esc"]).label("Escape").build(theme),
                theme,
            ),
            row(
                "On macOS",
                Kbd::new("kbd-command", ["Cmd", "K"])
                    .label("Command plus K")
                    .build(theme),
                theme,
            ),
        ])
        .gap(16.0)
        .max_width(length(480.0)),
        theme,
    )
}

fn row(label: &str, shortcut: Element, theme: &WidgetTheme) -> Element {
    Element::row([text(label, 14.0, theme.foreground, 400).grow(1.0), shortcut])
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .gap(16.0)
}
