use crate::{Input, InputKind, Menu, MenuItem, MenuResponse, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{Element, UiEvent, UiEventKind, ValueHandler};

/// Searchable action menu. Query and visibility stay owned by the application.
#[derive(Clone, Debug)]
pub struct CommandPalette {
    menu: Menu,
    query: String,
    input_handlers: Vec<ValueHandler<String>>,
}

impl CommandPalette {
    /// Creates an action menu filtered by the supplied query.
    ///
    /// `key` identifies the menu, `open` sets its visibility, `query` filters labels,
    /// and `items` supplies the candidate commands.
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
            input_handlers: Vec::new(),
        }
    }
    #[must_use]
    /// Sets the retained presence state used to animate the menu.
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.menu = self.menu.presence(presence);
        self
    }

    /// Adds a callback receiving the edited search query.
    #[must_use]
    pub fn on_input(mut self, handler: ValueHandler<String>) -> Self {
        self.input_handlers.push(handler);
        self
    }

    /// Adds a callback receiving the stable ID of an activated command.
    #[must_use]
    pub fn on_action(mut self, handler: ValueHandler<String>) -> Self {
        self.menu = self.menu.on_action(handler);
        self
    }

    /// Adds a callback receiving the requested palette open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.menu = self.menu.on_open_change(handler);
        self
    }
    #[must_use]
    /// Returns the state key for the search input.
    pub fn query_key(&self) -> String {
        format!("{}::query", self.menu.key)
    }
    #[must_use]
    /// Builds the palette around `trigger`, using `theme` for its controls.
    pub fn build(&self, trigger: Element, theme: &WidgetTheme) -> Element {
        let mut input = Input::new(
            self.query_key(),
            &self.query,
            "Search commands…",
            theme.input(),
        )
        .kind(InputKind::Search)
        .label("Search commands");
        for handler in &self.input_handlers {
            input = input.on_input(*handler);
        }
        self.menu.with_content(
            trigger,
            Element::column([
                input.build(),
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
    /// Returns the invoked menu response for `event`, if any.
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
                .find(|item| item.state.enabled && item.invocation().is_some())
                .and_then(|item| item.invocation())
                .map(MenuResponse::Invoke);
        }
        self.menu.response(event)
    }
}
