use argui::{
    accessibility::{LiveRegion, Role, Semantics},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, EventType, Sides, UiEventKind, percent},
    widgets::{Button, default_theme},
};

#[derive(Default)]
pub struct Example {
    announcements: u32,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let status = format!("{} accessible activations", self.announcements);
        Element::column([
            Element::text("Accessible by construction")
                .text_style(TextStyle {
                    color: theme.foreground,
                    ..TextStyle::default()
                })
                .semantics(Semantics::new(Role::Heading).level(1)),
            Button::new("announce", "Announce update", theme.button()).build(),
            Element::text(status)
                .text_style(TextStyle {
                    color: theme.foreground,
                    ..TextStyle::default()
                })
                .semantics(Semantics::new(Role::Status).live(LiveRegion::Polite)),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(16.0)
        .background(theme.background)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if event.target_key() == Some("announce") && matches!(event.kind, UiEventKind::Click(_))
            {
                app.announcements = app.announcements.saturating_add(1);
                cx.notify();
            }
        }))
    }
}
