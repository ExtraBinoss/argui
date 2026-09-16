use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, Sides, percent},
    widgets::{Button, Dialog, DialogBehavior, default_theme},
};

#[derive(Default)]
pub struct Example {
    open: bool,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let behavior = DialogBehavior::new("example-dialog", "Example dialog", self.open);
        let close_key = behavior.close_key();
        let trigger_key = behavior.trigger_key();
        let content = Element::column([
            Element::text("A real modal portal").text_style(TextStyle {
                color: theme.foreground,
                font_size: 24.0,
                weight: 700,
                ..TextStyle::default()
            }),
            Element::text("Focus stays inside until the dialog closes.").text_style(TextStyle {
                color: theme.muted_foreground,
                ..TextStyle::default()
            }),
            Button::new(close_key, "Close dialog", theme.outline_button()).build(),
        ])
        .gap(14.0);
        Dialog::new(
            "example-dialog",
            "Example dialog",
            self.open,
            Button::new(trigger_key, "Open dialog", theme.button()).build(),
            content,
        )
        .on_open_change(cx.value_callback(|app, open| app.open = open))
        .build(theme)
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .background(theme.background)
    }
}
