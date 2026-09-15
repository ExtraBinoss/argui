use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, EventType, Sides, UiEventKind, percent},
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
        let close_key =
            DialogBehavior::new("example-dialog", "Example dialog", self.open).close_key();
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
            Button::new("open-dialog", "Open dialog", theme.button()).build(),
            content,
        )
        .build(theme)
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .background(theme.background)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if !matches!(event.kind, UiEventKind::Click(_)) {
                return;
            }
            match event.target_key() {
                Some("open-dialog") => app.open = true,
                Some(key) if key.ends_with("::close") || key.ends_with("::backdrop") => {
                    app.open = false
                }
                _ => return,
            }
            cx.notify();
        }))
    }
}
