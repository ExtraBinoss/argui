use argui::{
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Element, JustifyContent, Sides, length, percent},
    widgets::default_theme,
};

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let heading = Element::text("Hello from Argui").text_style(TextStyle {
            font_size: 34.0,
            line_height: 42.0,
            color: theme.foreground,
            weight: 720,
            ..TextStyle::default()
        });
        let copy = Element::text(
            "The same retained Rust model runs in this browser and on native desktop.",
        )
        .text_style(TextStyle {
            color: theme.muted_foreground,
            ..TextStyle::default()
        });
        let card = Element::column([heading, copy])
            .width(length(560.0))
            .max_width(percent(1.0))
            .padding(Sides::length(30.0))
            .gap(14.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(16.0));
        Element::container([card])
            .width(percent(1.0))
            .height(percent(1.0))
            .padding(Sides::length(20.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.background)
    }
}
