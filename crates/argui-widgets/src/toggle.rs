use argui_ui::{Element, EventType, UiEvent, ValueHandler};

use crate::{Button, ButtonBehavior, WidgetTheme};

/// Controlled toggle button. Apply `action` to the retained pressed state.
#[derive(Clone, Debug)]
pub struct Toggle {
    pub key: String,
    pub label: String,
    pub pressed: bool,
    pub enabled: bool,
    pub outline: bool,
    change_handlers: Vec<ValueHandler<bool>>,
}

impl Toggle {
    /// Creates a controlled toggle with accessible `label` and initial `pressed` state.
    /// `key` identifies the button.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, pressed: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            pressed,
            enabled: true,
            outline: false,
            change_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Returns the requested pressed state when `event` activates this enabled toggle.
    pub fn action(&self, event: &UiEvent) -> Option<bool> {
        ButtonBehavior::new(&self.key, &self.label)
            .enabled(self.enabled)
            .action(event)
            .map(|_| !self.pressed)
    }

    /// Adds a callback receiving the next pressed state.
    ///
    /// `handler` is normally created with `Context::value_callback`.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the toggle using `theme` for its current-state styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let style = if self.pressed {
            theme.secondary_button()
        } else if self.outline {
            theme.outline_button()
        } else {
            theme.ghost_button()
        };
        let mut button = Button::new(&self.key, &self.label, style)
            .enabled(self.enabled)
            .build();
        button
            .semantics
            .as_mut()
            .expect("button semantics")
            .state
            .pressed = Some(self.pressed);
        for handler in self.change_handlers.iter().copied() {
            button = button.on(handler.direct_listener_value(EventType::Click, !self.pressed));
        }
        button
    }
}
