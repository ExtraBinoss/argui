use argui::{
    runtime::{Context, Render},
    ui::{NodeId, UiEvent, UiEventKind},
    widgets::{Button, CommandPalette, Menu, MenuItem, MenuResponse, WidgetTheme},
};

/// Gallery-owned visibility/query; the reusable widgets own presentation and keyboard policy.
pub(super) struct ActionMenu {
    pub open: bool,
    pub palette: bool,
    pub query: String,
    pub origin: Option<NodeId>,
    menu_presence: argui::widgets::Presence,
    palette_presence: argui::widgets::Presence,
}

impl Default for ActionMenu {
    fn default() -> Self {
        Self {
            open: false,
            palette: false,
            query: String::new(),
            origin: None,
            menu_presence: argui::widgets::Presence::default().fade_in(false),
            palette_presence: argui::widgets::Presence::default().fade_in(false),
        }
    }
}

impl ActionMenu {
    pub fn sync<T: Render>(&mut self, cx: &mut Context<T>) {
        let reduced = cx.environment().reduced_motion;
        self.menu_presence.set_open(self.open, reduced);
        self.palette_presence.set_open(self.palette, reduced);
        if self.animating() {
            cx.request_animation_frame();
        }
    }

    pub fn animating(&self) -> bool {
        self.menu_presence.animating() || self.palette_presence.animating()
    }

    pub fn advance<T: Render>(&mut self, frame: argui::animation::Frame, cx: &mut Context<T>) {
        let menu_changed = self.menu_presence.advance(frame.elapsed);
        let palette_changed = self.palette_presence.advance(frame.elapsed);
        if menu_changed || palette_changed {
            cx.notify();
        } else {
            cx.request_paint();
        }
    }

    pub fn items(&self, items: impl IntoIterator<Item = MenuItem>) -> Vec<MenuItem> {
        items
            .into_iter()
            .map(|mut item| {
                item.invocation.origin = self.origin.or(item.invocation.origin);
                item
            })
            .collect()
    }
    pub fn build(&self, items: Vec<MenuItem>, theme: &WidgetTheme) -> argui::ui::Element {
        argui::ui::Element::row([
            Menu::new(
                "app-menu",
                "Actions menu",
                self.open,
                self.items(items.clone()),
            )
            .presence(&self.menu_presence)
            .build(
                Button::new("menu-trigger", "Actions", theme.outline_button()).build(),
                theme,
            ),
            CommandPalette::new("app-palette", self.palette, &self.query, self.items(items))
                .presence(&self.palette_presence)
                .build(
                    Button::new(
                        "palette-trigger",
                        "Commands · Ctrl+K",
                        theme.outline_button(),
                    )
                    .build(),
                    theme,
                ),
        ])
        .gap(8.0)
    }
    pub fn handle<T: Render>(
        &mut self,
        event: &UiEvent,
        items: Vec<MenuItem>,
        cx: &mut Context<T>,
    ) {
        if matches!(event.kind, UiEventKind::Pointer(_))
            && matches!(event.target_key(), Some("app-menu" | "app-palette"))
            && !self.open
            && !self.palette
        {
            self.origin = event.focused_node();
        }
        if event.target_key() == Some("app-palette::query")
            && let UiEventKind::TextChanged(value) = &event.kind
        {
            self.query = value.clone();
            cx.notify();
            return;
        }
        let menu = Menu::new(
            "app-menu",
            "Actions menu",
            self.open,
            self.items(items.clone()),
        );
        let palette =
            CommandPalette::new("app-palette", self.palette, &self.query, self.items(items));
        let (response, palette_response) = if let Some(response) = palette.response(event) {
            (response, true)
        } else if let Some(response) = menu.response(event) {
            (response, false)
        } else {
            return;
        };
        match response {
            MenuResponse::Toggle => {
                if palette_response {
                    self.palette = !self.palette;
                    self.open = false;
                } else {
                    self.open = !self.open;
                    self.palette = false;
                }
            }
            MenuResponse::Close => {
                if matches!(event.kind, UiEventKind::KeyInput(_)) {
                    let _ = event.prevent_default();
                }
                self.open = false;
                self.palette = false;
            }
            MenuResponse::Focus(target) => {
                let _ = event.prevent_default();
                cx.request_focus(target);
            }
            MenuResponse::Invoke(invocation) => {
                let _ = event.prevent_default();
                cx.invoke_action(invocation);
                self.open = false;
                self.palette = false;
            }
        }
        cx.notify();
    }
    pub fn show_palette(&mut self, event: &UiEvent) {
        self.origin = event.focused_node();
        self.palette = true;
        self.open = false;
        self.query.clear();
    }
}
