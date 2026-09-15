use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, EventType, FlexWrap, Sides, UiEventKind, percent},
    widgets::{Button, default_theme},
};

#[derive(Default)]
pub struct Example {
    last_action: Option<String>,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let status = self.last_action.as_deref().unwrap_or("Choose an action");
        Element::column([
            Element::row([
                Button::new("save", "Save", theme.button()).build(),
                Button::new("preview", "Preview", theme.outline_button()).build(),
            ])
            .gap(10.0)
            .flex_wrap(FlexWrap::Wrap),
            Element::text(status).text_style(TextStyle {
                color: theme.foreground,
                weight: 600,
                ..TextStyle::default()
            }),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(20.0)
        .background(theme.background)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if matches!(event.kind, UiEventKind::Click(_)) {
                app.last_action = event
                    .target_key()
                    .map(|key| format!("Received Click({key})"));
                cx.notify();
            }
        }))
    }
}
