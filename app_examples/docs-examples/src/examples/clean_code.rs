use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, Sides, percent},
    widgets::{Button, default_theme},
};

#[derive(Default)]
pub struct Example {
    completed: usize,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let checks = [
            "State is explicit",
            "Views stay pure",
            "Effects have owners",
        ];
        let rows = checks.into_iter().enumerate().map(|(index, label)| {
            let mark = if index < self.completed { "✓" } else { "○" };
            Element::text(format!("{mark}  {label}")).text_style(TextStyle {
                color: theme.foreground,
                ..TextStyle::default()
            })
        });
        Element::column(
            rows.chain([Button::new("next", "Complete next", theme.button())
                .enabled(self.completed < checks.len())
                .on_click(cx.callback(|app| app.completed = (app.completed + 1).min(3)))
                .build()]),
        )
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(14.0)
        .background(theme.background)
    }
}
