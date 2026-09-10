use argui::{
    ui::{AlignItems, Element, length},
    widgets::{Separator, WidgetTheme},
};

use crate::app::text;

pub(super) fn render(theme: &WidgetTheme) -> Element {
    super::preview(
        "Room between ideas",
        "One component for horizontal rules, vertical dividers and centered labels.",
        Element::column([
            Element::column([
                text("Argui", 18.0, theme.foreground, 600),
                text(
                    "A toolkit for building your next interface.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(6.0),
            Separator::new("section-separator")
                .decorative(false)
                .build(theme),
            Element::row([
                text("Components", 14.0, theme.foreground, 500),
                Separator::new("separator-first")
                    .orientation(argui::ui::Orientation::Vertical)
                    .build(theme),
                text("Examples", 14.0, theme.foreground, 500),
                Separator::new("separator-second")
                    .orientation(argui::ui::Orientation::Vertical)
                    .build(theme),
                text("Resources", 14.0, theme.foreground, 500),
            ])
            .height(length(20.0))
            .gap(16.0)
            .align_items(AlignItems::CENTER),
            Separator::new("separator-label")
                .label("Or continue with")
                .build(theme),
            text("Email · GitHub · Google", 14.0, theme.foreground, 500)
                .text_align(argui::text::TextAlign::Center),
            Separator::new("separator-section")
                .label("Preferences")
                .decorative(false)
                .build(theme),
        ])
        .gap(20.0)
        .max_width(length(480.0)),
        theme,
    )
}
