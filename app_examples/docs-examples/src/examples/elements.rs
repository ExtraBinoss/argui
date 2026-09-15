use argui::{
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Element, FlexWrap, JustifyContent, Sides, length, percent},
    widgets::default_theme,
};

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let item = |label: &'static str| {
            Element::text(label)
                .text_style(TextStyle {
                    color: theme.primary_foreground,
                    weight: 700,
                    ..TextStyle::default()
                })
                .width(length(56.0))
                .height(length(40.0))
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::CENTER)
                .background(theme.primary)
                .radius(CornerRadii::all(8.0))
        };
        let panel = |title: &'static str, detail: &'static str, composition: Element| {
            Element::column([
                Element::text(title).text_style(TextStyle {
                    color: theme.foreground,
                    weight: 700,
                    ..TextStyle::default()
                }),
                Element::text(detail).text_style(TextStyle {
                    color: theme.muted_foreground,
                    font_size: 13.0,
                    ..TextStyle::default()
                }),
                Element::container([composition])
                    .height(length(150.0))
                    .padding(Sides::length(14.0))
                    .align_items(AlignItems::CENTER)
                    .justify_content(JustifyContent::CENTER)
                    .background(theme.muted)
                    .radius(CornerRadii::all(9.0)),
            ])
            .width(length(280.0))
            .grow(1.0)
            .padding(Sides::length(18.0))
            .gap(10.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(12.0))
        };

        let horizontal = Element::row([item("A"), item("B"), item("C")]).gap(10.0);
        let vertical = Element::column([item("A"), item("B"), item("C")]).gap(8.0);

        Element::row([
            panel(
                "Element::row",
                "Children flow from left to right.",
                horizontal,
            ),
            panel(
                "Element::column",
                "Children flow from top to bottom.",
                vertical,
            ),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(24.0))
        .gap(12.0)
        .flex_wrap(FlexWrap::Wrap)
        .background(theme.background)
    }
}
