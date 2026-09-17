use crate::{ChoiceMode, Collapsible, WidgetTheme, choice_navigation::navigate};
use argui_ui::{Element, EventType, Orientation, Role, Semantics, UiEvent, ValueHandler};

#[derive(Clone, Debug)]
pub struct AccordionItem {
    pub id: String,
    pub label: String,
    pub content: Element,
    pub open: bool,
    pub enabled: bool,
}

impl AccordionItem {
    /// Creates an item with a stable identity, trigger label and panel content.
    /// `id` identifies the item, `label` names its trigger, and `content` is shown in its panel.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, content: Element) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content,
            open: false,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccordionAction {
    Change(Vec<String>),
    Focus(String),
}

/// Controlled disclosures, with independent Tab stops and arrow/Home/End navigation.
#[derive(Clone, Debug)]
pub struct Accordion {
    pub key: String,
    pub items: Vec<AccordionItem>,
    pub mode: ChoiceMode,
    pub collapsible: bool,
    change_handlers: Vec<ValueHandler<Vec<String>>>,
}

impl Accordion {
    /// Creates an accordion from items in their display order.
    /// `key` identifies the accordion; `items` supplies its controlled disclosures.
    #[must_use]
    pub fn new(key: impl Into<String>, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        Self {
            key: key.into(),
            items: items.into_iter().collect(),
            mode: ChoiceMode::Single,
            collapsible: true,
            change_handlers: Vec::new(),
        }
    }

    /// Adds a callback receiving the stable IDs that should remain open.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<Vec<String>>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    fn disclosure(&self, item: &AccordionItem) -> Collapsible {
        Collapsible::new(
            format!("{}::item::{}", self.key, item.id),
            &item.label,
            item.open,
            item.content.clone(),
        )
        .enabled(item.enabled)
    }

    #[must_use]
    /// Returns the state key for the trigger belonging to `id`.
    pub fn trigger_key(&self, id: &str) -> String {
        format!("{}::item::{id}::trigger", self.key)
    }

    #[must_use]
    /// Interprets `event` as an accordion action when it targets an enabled item.
    pub fn action(&self, event: &UiEvent) -> Option<AccordionAction> {
        let index = self
            .items
            .iter()
            .position(|item| event.target_key() == Some(self.trigger_key(&item.id).as_str()))?;
        if !self.items[index].enabled {
            return None;
        }
        if let Some(next) = navigate(
            event,
            index,
            self.items.len(),
            Orientation::Vertical,
            false,
            |i| self.items[i].enabled,
        ) {
            return Some(AccordionAction::Focus(self.items[next].id.clone()));
        }
        let open = self.disclosure(&self.items[index]).action(event)?;
        if !open && !self.collapsible && self.mode == ChoiceMode::Single {
            return None;
        }
        let mut selected = if self.mode == ChoiceMode::Multiple {
            self.items
                .iter()
                .filter(|item| item.open && item.id != self.items[index].id)
                .map(|item| item.id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if open {
            selected.push(self.items[index].id.clone());
        }
        Some(AccordionAction::Change(selected))
    }

    #[must_use]
    /// Builds the accordion using `theme` for its item styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::column(self.items.iter().map(|item| {
            let mut disclosure = self
                .disclosure(item)
                .indicator(Element::text(if item.open { "−" } else { "+" }).text_style(
                    argui_text::TextStyle {
                        color: theme.foreground,
                        ..Default::default()
                    },
                ))
                .build(theme);
            if item.enabled {
                let next = self.selection_after_toggle(item);
                for handler in &self.change_handlers {
                    disclosure.children[0] = disclosure.children[0]
                        .clone()
                        .on(handler.direct_listener_value(EventType::Click, next.clone()));
                }
            }
            disclosure
        }))
        .keyed(&self.key)
        .gap(8.0)
        .semantics(Semantics::new(Role::Group))
    }

    fn selection_after_toggle(&self, item: &AccordionItem) -> Vec<String> {
        if item.open && (!self.collapsible && self.mode == ChoiceMode::Single) {
            return vec![item.id.clone()];
        }
        let mut selected = if self.mode == ChoiceMode::Multiple {
            self.items
                .iter()
                .filter(|candidate| candidate.open && candidate.id != item.id)
                .map(|candidate| candidate.id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if !item.open {
            selected.push(item.id.clone());
        }
        selected
    }
}
