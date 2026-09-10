use crate::app::text;
use argui::{
    paint::ImageId,
    ui::{AlignItems, Element, FlexWrap},
    widgets::{Avatar, WidgetTheme},
};

pub(super) fn render(theme: &WidgetTheme, image: ImageId) -> Element {
    super::preview(
        "People and teams",
        "Use a loaded image or initials while a photo is unavailable.",
        Element::column([
            Element::row([
                Avatar::new("avatar-photo", "Argui workspace", "AR")
                    .image(Some(image))
                    .size(64.0)
                    .build(theme),
                Element::column([
                    text("Argui workspace", 17.0, theme.foreground, 600),
                    text("Design and development", 14.0, theme.muted_foreground, 400),
                ])
                .gap(4.0),
            ])
            .gap(16.0)
            .align_items(AlignItems::CENTER),
            Element::row([
                Avatar::new("avatar-small", "Ada Lovelace", "AL")
                    .size(24.0)
                    .build(theme),
                Avatar::new("avatar-medium", "Grace Hopper", "GH").build(theme),
                Avatar::new("avatar-large", "Linus Torvalds", "LT")
                    .size(64.0)
                    .build(theme),
                Avatar::new("avatar-unavailable", "Photo unavailable", "?")
                    .image(None)
                    .build(theme),
            ])
            .gap(16.0)
            .align_items(AlignItems::CENTER)
            .flex_wrap(FlexWrap::Wrap),
        ])
        .gap(28.0),
        theme,
    )
}
