use crate::{WidgetGallery, navigation::Page};
use argui::{
    paint::CornerRadii,
    runtime::Context,
    ui::{Element, Sides, length},
    widgets::{Button, Empty, TablerIcon, WidgetAssets, WidgetTheme},
};

pub(super) fn render(
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut Context<WidgetGallery>,
) -> Element {
    let action = Button::new("empty-browse", "Browse components", theme.button())
        .on_click(cx.callback(|gallery| {
            gallery.page = Page::Card;
        }))
        .build();
    let media = Element::row([assets
        .icon(TablerIcon::Copy, 24.0)
        .vector_color(theme.foreground)])
    .padding(Sides::length(12.0))
    .width(length(48.0))
    .height(length(48.0))
    .background(theme.muted)
    .radius(CornerRadii::all(10.0));
    super::preview(
        "A useful starting point",
        "Explain what belongs here and offer a way forward.",
        Empty::new("empty-projects", "No projects yet")
            .description("Your projects will appear here. Explore the component library to start building your first interface.")
            .media(media)
            .content(action)
            .build(theme)
            .max_width(length(580.0)),
        theme,
    )
}
