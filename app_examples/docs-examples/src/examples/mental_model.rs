use argui::{
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, FlexWrap, Sides, length, percent},
    widgets::default_theme,
};

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let stage = |label| {
            Element::text(label)
                .text_style(TextStyle {
                    color: theme.primary_foreground,
                    weight: 650,
                    ..TextStyle::default()
                })
                .width(length(150.0))
                .grow(1.0)
                .padding(Sides::length(18.0))
                .background(theme.primary)
                .border(Border::all(1.0, theme.ring))
                .radius(CornerRadii::all(10.0))
        };
        Element::row([
            stage("Model state"),
            stage("Element tree"),
            stage("Layout + text"),
            stage("WGPU paint"),
        ])
        .width(percent(1.0))
        .padding(Sides::length(24.0))
        .gap(10.0)
        .flex_wrap(FlexWrap::Wrap)
        .background(theme.background)
    }
}
