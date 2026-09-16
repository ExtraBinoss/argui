use argui::{
    runtime::Context,
    ui::{Element, length},
    widgets::{Breadcrumb, BreadcrumbLink, Card, WidgetTheme},
};

use crate::{WidgetGallery, app::text, navigation::Page};

pub(super) fn render(theme: &WidgetTheme, cx: &mut Context<WidgetGallery>) -> Element {
    let breadcrumb = Breadcrumb::new(
        "component-path",
        [
            BreadcrumbLink::new("home", "Home"),
            BreadcrumbLink::new("components", "Components"),
        ],
        "Breadcrumb",
    )
    .on_activate(cx.value_callback(|gallery, id| {
        gallery.page = if id == "home" {
            Page::Button
        } else {
            Page::Card
        };
    }));
    let custom = Breadcrumb::new(
        "project-path",
        [
            BreadcrumbLink::new("projects", "Projects"),
            BreadcrumbLink::new("library", "Component library"),
        ],
        "Navigation",
    )
    .separator("›")
    .label("Project location")
    .on_activate(cx.value_callback(|gallery, id| {
        gallery.page = if id == "projects" {
            Page::Empty
        } else {
            Page::Card
        };
    }));
    Element::column([
        super::preview(
            "Know where you are",
            "Follow the trail back to an earlier page.",
            breadcrumb.build(theme),
            theme,
        ),
        super::preview(
            "A different separator",
            "Keep the current page quiet and let the path do the work.",
            Card::new(
                "breadcrumb-project",
                text(
                    "Build a familiar route through your workspace.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
            )
            .title("Component library")
            .footer(custom.build(theme))
            .build(theme)
            .max_width(length(560.0)),
            theme,
        ),
    ])
    .gap(24.0)
}
