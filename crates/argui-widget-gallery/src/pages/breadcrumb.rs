use argui::{
    runtime::Context,
    ui::{Element, EventType, length},
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
    );
    let custom = Breadcrumb::new(
        "project-path",
        [
            BreadcrumbLink::new("projects", "Projects"),
            BreadcrumbLink::new("library", "Component library"),
        ],
        "Navigation",
    )
    .separator("›")
    .label("Project location");
    Element::column([
        super::preview(
            "Know where you are",
            "Follow the trail back to an earlier page.",
            breadcrumb
                .build(theme)
                .on(cx.listener(EventType::Click, move |gallery, event, cx| {
                    if let Some(id) = breadcrumb.action(event) {
                        gallery.page = if id == "home" {
                            Page::Button
                        } else {
                            Page::Card
                        };
                        event.stop_propagation();
                        cx.notify();
                    }
                })),
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
            .footer(custom.build(theme).on(cx.listener(
                EventType::Click,
                move |gallery, event, cx| {
                    if let Some(id) = custom.action(event) {
                        gallery.page = if id == "projects" {
                            Page::Empty
                        } else {
                            Page::Card
                        };
                        event.stop_propagation();
                        cx.notify();
                    }
                },
            )))
            .build(theme)
            .max_width(length(560.0)),
            theme,
        ),
    ])
    .gap(24.0)
}
