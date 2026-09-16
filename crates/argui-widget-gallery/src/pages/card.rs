use argui::{
    runtime::Context,
    ui::{AlignItems, Element, FlexWrap, JustifyContent, length},
    widgets::{
        Badge, BadgeVariant, Button, Card, Separator, TablerIcon, WidgetAssets, WidgetTheme,
    },
};

use crate::{WidgetGallery, app::text, navigation::Page};

pub(super) fn render(
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut Context<WidgetGallery>,
) -> Element {
    let details = Button::new(
        "card-details",
        "View project details",
        theme.outline_button(),
    )
    .trailing(
        assets
            .icon(TablerIcon::ChevronRight, 16.0)
            .vector_color(theme.foreground),
    )
    .on_click(cx.callback(|gallery| {
        gallery.page = Page::Collapsible;
    }))
    .build();
    super::preview(
        "Project overview",
        "Compose a header, an optional action, body content and a footer.",
        Card::new(
            "project-card",
            Element::column([
                text("Design system", 26.0, theme.foreground, 650),
                text(
                    "A shared home for the components your team builds with.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
                Separator::new("card-divider").build(theme),
                Element::row([
                    text("Components", 14.0, theme.muted_foreground, 400),
                    text("34", 14.0, theme.foreground, 600),
                ])
                .justify_content(JustifyContent::SPACE_BETWEEN),
            ])
            .gap(16.0),
        )
        .title("Your workspace")
        .description("Everything you need to keep moving.")
        .action(
            Badge::new("card-plan", "Personal")
                .variant(BadgeVariant::Secondary)
                .build(theme),
        )
        .footer(
            Element::row([
                Badge::new("card-status", "Up to date")
                    .variant(BadgeVariant::Outline)
                    .leading(
                        assets
                            .icon(TablerIcon::Check, 12.0)
                            .vector_color(theme.foreground),
                    )
                    .build(theme),
                details,
            ])
            .gap(12.0)
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN),
        )
        .build(theme)
        .max_width(length(520.0)),
        theme,
    )
}
