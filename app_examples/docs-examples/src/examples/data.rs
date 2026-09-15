use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, EventType, Sides, UiEventKind, VirtualList, length, percent},
    widgets::default_theme,
};

pub struct Example {
    list: VirtualList,
    offset: f32,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            list: VirtualList::fixed(10_000, 42.0, 300.0).overscan(6),
            offset: 0.0,
        }
    }
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        self.list
            .build("records", self.offset, |index| {
                Element::text(format!("Record #{:05}", index + 1))
                    .text_style(TextStyle {
                        color: theme.foreground,
                        ..TextStyle::default()
                    })
                    .height(length(42.0))
                    .padding(Sides::length(10.0))
            })
            .width(percent(1.0))
            .height(length(300.0))
            .background(theme.background)
            .on(cx.listener(EventType::Scroll, |app, event, cx| {
                if let UiEventKind::Scrolled { offset, .. } = event.kind {
                    app.offset = offset.y;
                    cx.notify();
                }
            }))
    }
}
