use argui::{
    runtime::{Context, Render},
    ui::{Element, EventType, Sides, UiEvent, percent},
    widgets::{Button, shadcn},
};

#[derive(Default)]
pub(crate) struct HotReloadDemo {
    count: u32,
}

impl HotReloadDemo {
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        match event.target_key() {
            Some("hot-reload-increment") => self.count = self.count.saturating_add(1),
            Some("hot-reload-reset") => self.count = 0,
            _ => return,
        }
        cx.notify();
    }
}

impl Render for HotReloadDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let live_copy = "Edit this sentence while the counter is running.";
        let status = if cfg!(feature = "hot-reload") && cfg!(debug_assertions) {
            "Subsecond hot patching is enabled for this debug build."
        } else {
            "Run the gallery in debug mode with the hot-reload feature to apply Rust patches."
        };

        super::preview(
            "State-preserving Rust patches",
            "Change a render function or event handler and keep the current application state.",
            Element::column([
                crate::app::text(status, 13.0, theme.muted_foreground, 500),
                crate::app::text(live_copy, 18.0, theme.foreground, 650),
                crate::app::text(
                    format!("Preserved counter: {}", self.count),
                    28.0,
                    theme.foreground,
                    740,
                ),
                Element::row([
                    Button::new("hot-reload-increment", "Increment", theme.button()).build(),
                    Button::new("hot-reload-reset", "Reset", theme.outline_button()).build(),
                ])
                .gap(8.0),
                crate::app::text(
                    "Try it: increment the counter, edit live_copy in pages/hot_reload.rs, then save. The sentence updates without resetting the counter.",
                    13.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(14.0)
            .padding(Sides::length(4.0))
            .width(percent(1.0)),
            theme,
        )
        .on(cx.listener(EventType::Click, Self::event))
    }
}
