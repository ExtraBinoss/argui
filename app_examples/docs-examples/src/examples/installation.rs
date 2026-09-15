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
        let text = |value, size, weight| {
            Element::text(value).text_style(TextStyle {
                font_size: size,
                line_height: size * 1.35,
                color: theme.foreground,
                weight,
                ..TextStyle::default()
            })
        };
        let card = Element::column([
            text("Argui is ready", 30.0, 700),
            text("Rust + WebAssembly + WebGPU", 16.0, 450),
        ])
        .width(length(460.0))
        .max_width(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(10.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(14.0));
        Element::container([card])
            .width(percent(1.0))
            .height(percent(1.0))
            .padding(Sides::length(20.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.background)
    }
}
