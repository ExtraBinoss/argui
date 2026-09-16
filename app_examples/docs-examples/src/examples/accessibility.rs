use argui::{
    accessibility::{LiveRegion, Role, Semantics},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, Sides, percent},
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
            Button::new("announce", "Announce update", theme.button())
                .on_click(cx.callback(|app| {
                    app.announcements = app.announcements.saturating_add(1);
                }))
                .build(),
            Element::text(status.clone())
                .text_style(TextStyle {
                    color: theme.foreground,
                    ..TextStyle::default()
                })
                .semantics(
                    Semantics::new(Role::Status)
                        .label(status)
                        .live(LiveRegion::Polite),
                ),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(16.0)
        .background(theme.background)
    }
}
