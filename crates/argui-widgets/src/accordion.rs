use crate::{ChoiceMode, Collapsible, WidgetTheme, choice_navigation::navigate};
use argui_ui::{Element, Orientation, Role, Semantics, UiEvent};

#[derive(Clone, Debug)]
pub struct AccordionItem {
    pub id: String,
    pub label: String,
    pub content: Element,
    pub open: bool,
    pub enabled: bool,
}

impl AccordionItem {
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
}

impl Accordion {
    #[must_use]
    pub fn new(key: impl Into<String>, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        Self {
            key: key.into(),
            items: items.into_iter().collect(),
            mode: ChoiceMode::Single,
            collapsible: true,
        }
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
    pub fn trigger_key(&self, id: &str) -> String {
        format!("{}::item::{id}::trigger", self.key)
    }

    #[must_use]
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
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::column(self.items.iter().map(|item| {
            self.disclosure(item)
                .indicator(Element::text(if item.open { "−" } else { "+" }).text_style(
                    argui_text::TextStyle {
                        color: theme.foreground,
                        ..Default::default()
                    },
                ))
                .build(theme)
        }))
        .keyed(&self.key)
        .gap(8.0)
        .semantics(Semantics::new(Role::Group))
    }
}
