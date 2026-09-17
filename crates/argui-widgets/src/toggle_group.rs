use crate::{ChoiceMode, Toggle, WidgetTheme, choice_navigation::navigate};
use argui_ui::{
    Element, EventType, FocusPolicy, Orientation, Role, Semantics, UiEvent, ValueHandler,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToggleGroupAction {
    Change(Vec<String>),
    Focus(String),
}

/// Roving focus and controlled single/multiple selection. Item keys are stable values.
#[derive(Clone, Debug)]
pub struct ToggleGroup {
    pub key: String,
    pub label: String,
    pub items: Vec<Toggle>,
    pub mode: ChoiceMode,
    pub orientation: Orientation,
    pub rtl: bool,
    pub active: Option<String>,
    change_handlers: Vec<ValueHandler<Vec<String>>>,
}

impl ToggleGroup {
    /// Creates a controlled group from items in display order.
    /// `key` identifies the group and `label` names it accessibly.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        items: impl IntoIterator<Item = Toggle>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            mode: ChoiceMode::Single,
            orientation: Orientation::Horizontal,
            rtl: false,
            active: None,
            change_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Returns the interaction key for item `id`.
    pub fn item_key(&self, id: &str) -> String {
        format!("{}::item::{id}", self.key)
    }

    #[must_use]
    /// Returns the requested item-state change for `event`, if any.
    pub fn action(&self, event: &UiEvent) -> Option<ToggleGroupAction> {
        let index = self
            .items
            .iter()
            .position(|item| event.target_key() == Some(self.item_key(&item.key).as_str()))?;
        if !self.items[index].enabled {
            return None;
        }
        if let Some(next) = navigate(
            event,
            index,
            self.items.len(),
            self.orientation,
            self.rtl,
            |i| self.items[i].enabled,
        ) {
            return Some(ToggleGroupAction::Focus(self.items[next].key.clone()));
        }
        let mut item = self.items[index].clone();
        item.key = self.item_key(&item.key);
        let pressed = item.action(event)?;
        let mut selected: Vec<_> = if self.mode == ChoiceMode::Multiple {
            self.items
                .iter()
                .filter(|item| item.pressed && item.key != self.items[index].key)
                .map(|item| item.key.clone())
                .collect()
        } else {
            Vec::new()
        };
        if pressed {
            selected.push(self.items[index].key.clone());
        }
        Some(ToggleGroupAction::Change(selected))
    }

    /// Adds a callback receiving the selected stable item keys.
    ///
    /// `handler` is normally created with `Context::value_callback`. Single
    /// selection returns zero or one key; multiple selection returns all keys
    /// that remain selected after activation.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<Vec<String>>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    fn selection_after_activation(&self, index: usize) -> Vec<String> {
        let next_pressed = !self.items[index].pressed;
        let mut selected: Vec<_> = if self.mode == ChoiceMode::Multiple {
            self.items
                .iter()
                .enumerate()
                .filter(|(candidate, item)| *candidate != index && item.pressed)
                .map(|(_, item)| item.key.clone())
                .collect()
        } else {
            Vec::new()
        };
        if next_pressed {
            selected.push(self.items[index].key.clone());
        }
        selected
    }

    #[must_use]
    /// Builds the toggle group using `theme` for its item styling.
    ///
    /// # Panics
    ///
    /// Panics if a built toggle unexpectedly has no interaction properties.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let active = self
            .items
            .iter()
            .position(|item| item.enabled && self.active.as_deref() == Some(&item.key))
            .or_else(|| {
                self.items
                    .iter()
                    .position(|item| item.enabled && item.pressed)
            })
            .or_else(|| self.items.iter().position(|item| item.enabled));
        let items = self.items.iter().enumerate().map(|(index, item)| {
            let mut item = item.clone();
            item.key = self.item_key(&item.key);
            let enabled = item.enabled;
            let mut button = item.build(theme);
            button
                .interaction
                .as_mut()
                .expect("toggle interaction")
                .focus_policy = if !enabled {
                FocusPolicy::None
            } else if active == Some(index) {
                FocusPolicy::TabStop
            } else {
                FocusPolicy::Programmatic
            };
            let selected = self.selection_after_activation(index);
            for handler in self.change_handlers.iter().copied() {
                button =
                    button.on(handler.direct_listener_value(EventType::Click, selected.clone()));
            }
            button
        });
        let root = match self.orientation {
            Orientation::Horizontal => Element::row(items),
            Orientation::Vertical => Element::column(items),
        };
        root.keyed(&self.key)
            .gap(2.0)
            .semantics(Semantics::new(Role::Group).label(&self.label))
            .writing_direction(if self.rtl {
                argui_ui::WritingDirection::Rtl
            } else {
                argui_ui::WritingDirection::Ltr
            })
    }
}
