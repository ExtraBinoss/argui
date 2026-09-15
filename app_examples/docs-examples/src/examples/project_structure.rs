use argui::{
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::{FontFamily, TextStyle},
    ui::{Element, Sides, percent},
    widgets::default_theme,
};

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let file = |name, purpose| {
            Element::row([
                Element::text(name).text_style(TextStyle {
                    family: FontFamily::Monospace,
                    color: theme.primary,
                    weight: 650,
                    ..TextStyle::default()
                }),
                Element::text(purpose)
                    .text_style(TextStyle {
                        color: theme.muted_foreground,
                        ..TextStyle::default()
                    })
                    .grow(1.0),
            ])
            .gap(16.0)
            .padding(Sides::length(14.0))
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0))
        };
        Element::column([
            file("main.rs", "target entry point"),
            file("app.rs", "state and event routing"),
            file("app/view.rs", "Element composition"),
            file("services/", "domain I/O"),
        ])
        .width(percent(1.0))
        .padding(Sides::length(24.0))
        .gap(9.0)
        .background(theme.background)
    }
}
