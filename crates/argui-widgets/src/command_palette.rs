use crate::{Input, InputKind, Menu, MenuItem, MenuResponse, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{Element, UiEvent, UiEventKind};

/// Searchable action menu. Query and visibility stay owned by the application.
#[derive(Clone, Debug)]
pub struct CommandPalette {
    menu: Menu,
    query: String,
}

impl CommandPalette {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        open: bool,
        query: impl Into<String>,
        items: Vec<MenuItem>,
    ) -> Self {
        let query = query.into();
        let needle = query.to_lowercase();
        let items = items
            .into_iter()
            .filter(|item| item.state.label.to_lowercase().contains(&needle))
            .collect();
        Self {
            menu: Menu::new(key, "Commands", open, items),
            query,
        }
    }
    #[must_use]
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.menu = self.menu.presence(presence);
        self
    }
    #[must_use]
    pub fn query_key(&self) -> String {
        format!("{}::query", self.menu.key)
    }
    #[must_use]
    pub fn build(&self, trigger: Element, theme: &WidgetTheme) -> Element {
        self.menu.with_content(
            trigger,
            Element::column([
                Input::new(
                    self.query_key(),
                    &self.query,
                    "Search commands…",
                    theme.input(),
                )
                .kind(InputKind::Search)
                .label("Search commands")
                .build(),
                if self.menu.items.is_empty() {
                    Element::text("No matching commands").text_style(theme.ghost_button().label)
                } else {
                    self.menu.content(theme)
                },
            ])
            .gap(6.0),
            theme,
        )
    }
    #[must_use]
    pub fn response(&self, event: &UiEvent) -> Option<MenuResponse> {
        if self.menu.open
            && event.target_key() == Some(self.query_key().as_str())
            && let UiEventKind::KeyInput(input) = &event.kind
            && input.state == KeyState::Pressed
            && !input.repeat
            && input.key == Key::Enter
        {
            return self
                .menu
                .items
                .iter()
                .find(|item| item.state.enabled)
                .map(|item| MenuResponse::Invoke(item.invocation));
        }
        self.menu.response(event)
    }
}
