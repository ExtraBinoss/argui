use argui::{
    ui::{Element, LiveRegion, length},
    widgets::{Alert, AlertVariant, TablerIcon, WidgetAssets, WidgetTheme},
};

pub(super) fn render(theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
    super::preview(
        "Keep people informed",
        "Inline messages stay alongside the content they describe.",
        Element::column([
            Alert::new("alert-info", "Your changes have been saved")
                .description("Everyone in your workspace can now see the latest version.")
                .icon(
                    assets
                        .icon(TablerIcon::Success, 18.0)
                        .vector_color(theme.foreground),
                )
                .live(LiveRegion::Off)
                .build(theme),
            Alert::new("alert-error", "We couldn't sync your changes")
                .description(
                    "Your work is saved on this device. Check your connection and try again.",
                )
                .variant(AlertVariant::Destructive)
                .icon(assets.icon(TablerIcon::Warning, 18.0))
                .live(LiveRegion::Off)
                .build(theme),
            Alert::new("alert-simple", "A new version is available.")
                .live(LiveRegion::Off)
                .build(theme),
        ])
        .gap(16.0)
        .max_width(length(580.0)),
        theme,
    )
}
