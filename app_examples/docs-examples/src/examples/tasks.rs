use std::time::Duration;

use argui::{
    runtime::{
        Context, Render,
        tasks::{TaskHandle, sleep},
    },
    text::TextStyle,
    ui::{Element, Sides, percent},
    widgets::{Button, default_theme},
};

#[derive(Default)]
pub struct Example {
    result: Option<String>,
    task: Option<TaskHandle>,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let status = self.result.as_deref().unwrap_or("No task running");
        Element::column([
            Button::new("load", "Load asynchronously", theme.button())
                .enabled(self.task.is_none())
                .on_click(cx.event_handler(|app, _, cx| {
                    app.result = Some("Loading…".into());
                    app.task = cx
                        .spawn(
                            async { sleep(Duration::from_millis(650)).await },
                            |app, result, cx| {
                                result.expect("the local timer completes");
                                app.result = Some("Loaded without blocking the UI".into());
                                app.task = None;
                                cx.notify();
                            },
                        )
                        .ok();
                    cx.notify();
                }))
                .build(),
            Element::text(status).text_style(TextStyle {
                color: theme.foreground,
                ..TextStyle::default()
            }),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(16.0)
        .background(theme.background)
    }
}
