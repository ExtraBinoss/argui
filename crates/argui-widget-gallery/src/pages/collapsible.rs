use argui::{
    paint::{Border, CornerRadii},
    runtime::Context,
    ui::{Element, EventType, Sides, length},
    widgets::{Collapsible, TablerIcon, WidgetAssets, WidgetTheme},
};

use crate::{WidgetGallery, app::text};

pub(super) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
    cx: &mut Context<WidgetGallery>,
) -> Element {
    let open = gallery.collapsible_open;
    let disclosure = Collapsible::new(
        "project-files",
        "Project files",
        open,
        Element::column([
            file("components/button.rs", theme),
            file("components/card.rs", theme),
            file("components/badge.rs", theme),
        ])
        .gap(8.0),
    )
    .trigger(text(
        "3 components in this project",
        14.0,
        theme.foreground,
        600,
    ))
    .indicator(
        assets
            .icon(
                if open {
                    TablerIcon::ChevronDown
                } else {
                    TablerIcon::ChevronRight
                },
                16.0,
            )
            .vector_color(theme.foreground),
    )
    .build(theme)
    .on(cx.listener(EventType::Click, |gallery, event, cx| {
        if let Some(open) = Collapsible::new(
            "project-files",
            "Project files",
            gallery.collapsible_open,
            Element::container([]),
        )
        .action(event)
        {
            gallery.collapsible_open = open;
            event.stop_propagation();
            cx.notify();
        }
    }));
    super::preview(
        "A closer look",
        "Expand the project or archive to browse its components.",
        Element::column([
            disclosure,
            Collapsible::new(
                "archived-files",
                "Archived files",
                gallery.archived_open,
                Element::column([
                    file("archive/button-v1.rs", theme),
                    file("archive/theme-v1.rs", theme),
                ])
                .gap(8.0),
            )
            .indicator(
                assets
                    .icon(
                        if gallery.archived_open {
                            TablerIcon::ChevronDown
                        } else {
                            TablerIcon::ChevronRight
                        },
                        16.0,
                    )
                    .vector_color(theme.foreground),
            )
            .build(theme)
            .on(cx.listener(EventType::Click, |gallery, event, cx| {
                if let Some(open) = Collapsible::new(
                    "archived-files",
                    "Archived files",
                    gallery.archived_open,
                    Element::container([]),
                )
                .action(event)
                {
                    gallery.archived_open = open;
                    event.stop_propagation();
                    cx.notify();
                }
            })),
        ])
        .gap(24.0)
        .max_width(length(480.0)),
        theme,
    )
}

fn file(name: &str, theme: &WidgetTheme) -> Element {
    text(name, 14.0, theme.foreground, 400)
        .padding(Sides::length(12.0))
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(7.0))
}
