use argui::{
    core::Color,
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Element, FlexWrap, JustifyContent, Sides, length, percent},
    widgets::default_theme,
};

/// Exact-source example comparing identical glyphs over several solid backdrops.
#[derive(Default)]
pub struct Example;

/// Builds one color sample with identical typography and an explicit solid backdrop.
fn sample(label: &str, colors: &str, background: Color, foreground: Color) -> Element {
    Element::column([
        Element::text("Ag 0123 · Crisp type").text_style(TextStyle {
            color: foreground,
            font_size: 17.0,
            line_height: 23.0,
            weight: 560,
            ..TextStyle::default()
        }),
        Element::text(label).text_style(TextStyle {
            color: foreground.with_alpha(0.78),
            font_size: 12.0,
            line_height: 17.0,
            weight: 600,
            ..TextStyle::default()
        }),
        Element::text(colors).text_style(TextStyle {
            color: foreground.with_alpha(0.62),
            font_size: 10.0,
            line_height: 15.0,
            weight: 500,
            ..TextStyle::default()
        }),
    ])
    .grow(1.0)
    .min_width(length(190.0))
    .height(length(116.0))
    .padding(Sides::length(18.0))
    .gap(5.0)
    .align_items(AlignItems::START)
    .justify_content(JustifyContent::CENTER)
    .background(background)
    .radius(CornerRadii::all(14.0))
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let samples = Element::row([
            sample("Dark on light", "#18181B on #FFFFFF", Color::WHITE, Color::from_srgb8(24, 24, 27)),
            sample("Light on dark", "#FFFFFF on #18181B", Color::from_srgb8(24, 24, 27), Color::WHITE),
            sample("Light on blue", "#FFFFFF on #2563EB", Color::from_srgb8(37, 99, 235), Color::WHITE),
        ])
        .width(percent(1.0))
        .flex_wrap(FlexWrap::Wrap)
        .gap(12.0);

        Element::column([
            Element::text("Text fidelity across color").text_style(TextStyle {
                color: theme.foreground,
                font_size: 20.0,
                line_height: 27.0,
                weight: 720,
                ..TextStyle::default()
            }),
            Element::text(
                "The same size and weight are used in every sample. Argui resolves each solid backdrop so glyph edges keep a consistent optical weight.",
            )
            .text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 13.0,
                line_height: 19.0,
                ..TextStyle::default()
            }),
            samples,
            Element::text(
                "Tip: animate the highlight surface, not a fractional transform on the text itself.",
            )
            .padding(Sides::length(14.0))
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(10.0))
            .text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 12.0,
                line_height: 18.0,
                weight: 550,
                ..TextStyle::default()
            }),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(14.0)
        .background(theme.background)
    }
}
