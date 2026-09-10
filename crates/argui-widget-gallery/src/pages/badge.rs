use argui::{
    ui::{Element, FlexWrap},
    widgets::{Badge, BadgeVariant, TablerIcon, WidgetAssets, WidgetTheme},
};

pub(super) fn render(theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    super::preview(
        "A little context",
        "Use badges for categories, counts and short status labels.",
        Element::column([
            Element::row([
                Badge::new("badge-primary", "Default").build(theme),
                Badge::new("badge-secondary", "Secondary")
                    .variant(BadgeVariant::Secondary)
                    .build(theme),
                Badge::new("badge-destructive", "Destructive")
                    .variant(BadgeVariant::Destructive)
                    .build(theme),
                Badge::new("badge-outline", "Outline")
                    .variant(BadgeVariant::Outline)
                    .build(theme),
                Badge::new("badge-ghost", "Ghost")
                    .variant(BadgeVariant::Ghost)
                    .build(theme),
            ])
            .gap(8.0)
            .flex_wrap(FlexWrap::Wrap),
            Element::row([
                Badge::new("badge-verified", "Verified")
                    .leading(
                        assets
                            .icon(TablerIcon::Check, 12.0)
                            .vector_color(theme.primary_foreground),
                    )
                    .build(theme),
                Badge::new("badge-review", "In review")
                    .variant(BadgeVariant::Secondary)
                    .leading(
                        assets
                            .icon(TablerIcon::Information, 12.0)
                            .vector_color(theme.foreground),
                    )
                    .build(theme),
                Badge::new("badge-version", "Version 1.0")
                    .variant(BadgeVariant::Outline)
                    .trailing(
                        assets
                            .icon(TablerIcon::ChevronRight, 12.0)
                            .vector_color(theme.foreground),
                    )
                    .build(theme),
            ])
            .gap(8.0)
            .flex_wrap(FlexWrap::Wrap),
        ])
        .gap(24.0),
        theme,
    )
}
