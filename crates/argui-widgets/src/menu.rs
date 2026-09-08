use crate::{Button, Popover, PopoverAction, PopoverBehavior, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    ActionInvocation, ActionState, Element, FocusTarget, JustifyContent, Role, Semantics, UiEvent,
    UiEventKind, percent,
};

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub invocation: ActionInvocation,
    pub state: ActionState,
}

impl MenuItem {
    #[must_use]
    pub fn new(invocation: ActionInvocation, state: ActionState) -> Self {
        Self { invocation, state }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuResponse {
    Toggle,
    Close,
    Focus(FocusTarget),
    Invoke(ActionInvocation),
}

/// Controlled single-level action menu. Handle `response` in the owning component.
#[derive(Clone, Debug)]
pub struct Menu {
    pub key: String,
    pub label: String,
    pub open: bool,
    pub items: Vec<MenuItem>,
    presence: Option<crate::Presence>,
}

impl Menu {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        items: Vec<MenuItem>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
            items,
            presence: None,
        }
    }
    /// Retain entry/exit motion; the owner advances this presence each frame.
    #[must_use]
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.open = presence.is_open();
        self.presence = Some(presence.clone());
        self
    }
    #[must_use]
    pub fn build(&self, trigger: Element, theme: &WidgetTheme) -> Element {
        self.with_content(trigger, self.content(theme), theme)
    }
    pub(crate) fn with_content(
        &self,
        trigger: Element,
        content: Element,
        theme: &WidgetTheme,
    ) -> Element {
        let mut popover = Popover::new(&self.key, &self.label, self.open, trigger, content)
            .trap_focus(true)
            .padding(6.0)
            .size(340.0, 360.0);
        if let Some(presence) = &self.presence {
            popover = popover.presence(presence);
        }
        popover.build(theme)
    }
    pub(crate) fn content(&self, theme: &WidgetTheme) -> Element {
        Element::column(self.items.iter().enumerate().map(|(index, item)| {
            let mut button = Button::new(
                self.item_key(index),
                &item.state.label,
                theme.ghost_button(),
            )
            .enabled(item.state.enabled);
            if let Some(shortcut) = &item.state.shortcut {
                button = button.trailing(
                    Element::text(shortcut.label()).text_style(theme.ghost_button().label),
                );
            }
            let mut element = button
                .build()
                .width(percent(1.0))
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .action_from(item.invocation);
            if let Some(semantics) = &mut element.semantics {
                semantics.role = Role::MenuItem;
            }
            element
        }))
        .gap(2.0)
        .semantics(Semantics::new(Role::Menu).label(&self.label))
    }
    pub(crate) fn item_key(&self, index: usize) -> String {
        format!("{}::item::{index}", self.key)
    }
    #[must_use]
    pub fn response(&self, event: &UiEvent) -> Option<MenuResponse> {
        if matches!(event.kind, UiEventKind::KeyInput(_))
            && !event
                .target_key()
                .is_some_and(|key| key == self.key || key.starts_with(&format!("{}::", self.key)))
        {
            return None;
        }
        if let Some(action) = PopoverBehavior::new(&self.key, &self.label, self.open).action(event)
        {
            return Some(match action {
                PopoverAction::Toggle => MenuResponse::Toggle,
                PopoverAction::Close => MenuResponse::Close,
            });
        }
        if !self.open {
            return None;
        }
        if matches!(event.kind, UiEventKind::Click(_))
            && self.items.iter().enumerate().any(|(index, item)| {
                item.state.enabled && event.target_key() == Some(self.item_key(index).as_str())
            })
        {
            return Some(MenuResponse::Close);
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed {
            return None;
        }
        let enabled: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| item.state.enabled.then_some(i))
            .collect();
        if enabled.is_empty() {
            return None;
        }
        let current = enabled
            .iter()
            .position(|&index| event.target_key() == Some(self.item_key(index).as_str()));
        let next = match input.key {
            Key::ArrowDown => current.map_or(0, |index| (index + 1) % enabled.len()),
            Key::ArrowUp => current.map_or(enabled.len() - 1, |index| {
                (index + enabled.len() - 1) % enabled.len()
            }),
            Key::Home => 0,
            Key::End => enabled.len() - 1,
            _ => return None,
        };
        Some(MenuResponse::Focus(self.item_key(enabled[next]).into()))
    }
}
