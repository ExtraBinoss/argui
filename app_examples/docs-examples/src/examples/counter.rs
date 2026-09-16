use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Element, Sides, percent},
    widgets::{Button, default_theme},
};

#[derive(Default)]
pub struct Example {
    count: u32,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            Element::text(format!("Count: {}", self.count)).text_style(TextStyle {
                font_size: 34.0,
                line_height: 42.0,
                color: theme.foreground,
                weight: 700,
                ..TextStyle::default()
            }),
            Button::new("increment", "Increment", theme.button())
                .on_click(cx.callback(|app| app.count = app.count.saturating_add(1)))
                .build(),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .padding(Sides::length(28.0))
        .gap(18.0)
        .background(theme.background)
    }
}
