use argui::{
    runtime::Context,
    ui::{Element, length},
    widgets::{Card, Pagination, WidgetTheme},
};

use crate::{WidgetGallery, app::text};

pub(super) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    cx: &mut Context<WidgetGallery>,
) -> Element {
    let page = gallery.result_page;
    let pagination = Pagination::new("results", page, 12)
        .on_select(cx.value_callback(|gallery, page| gallery.result_page = page));
    let titles = [
        "Navigation patterns",
        "Designing with contrast",
        "A shared component library",
    ];
    let rows = titles.iter().enumerate().map(|(index, title)| {
        Element::column([
            text(
                format!("{:02}  {title}", (page - 1) * 3 + index + 1),
                15.0,
                theme.foreground,
                600,
            ),
            text(
                "Notes from the design system",
                13.0,
                theme.muted_foreground,
                400,
            ),
        ])
        .gap(5.0)
    });
    let content = Card::new("result-list", Element::column(rows).gap(24.0))
        .title("Library notes")
        .description(format!("Page {page} of 12 · 36 notes"))
        .footer(pagination.build(theme))
        .build(theme)
        .max_width(length(720.0));
    Element::column([
        super::preview(
            "One page at a time",
            "Browse the notes with page numbers, Previous and Next.",
            content,
            theme,
        ),
        super::preview(
            "Only one page",
            "Navigation stays visible when there is nowhere else to go.",
            Pagination::new("single-page", 1, 1).build(theme),
            theme,
        ),
    ])
    .gap(24.0)
}
