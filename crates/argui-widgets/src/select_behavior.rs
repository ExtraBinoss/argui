use argui_core::{Key, KeyState};
use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Orientation, Role,
    SemanticAction, SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind,
    UserSelect,
};

use crate::select::SelectOption;

pub const SELECT_SCOPE: StateScopeId = StateScopeId::new("select");
pub const SELECT_OPEN: StateName = StateName::new("open");
pub const SELECT_SELECTED: StateName = StateName::new("selected");
pub const SELECT_HIGHLIGHTED: StateName = StateName::new("highlighted");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectPart {
    Root,
    Trigger,
    Value,
    List,
    Option(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectAction {
    Toggle,
    Close,
    Highlight(usize),
    Select(usize),
}

#[derive(Clone, Debug)]
pub struct SelectBehavior {
    key: String,
    label: String,
    options: Vec<SelectOption>,
    selected: Option<usize>,
    highlighted: usize,
    open: bool,
}

impl SelectBehavior {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            highlighted: selected.unwrap_or(0),
            open: false,
        }
    }

    #[must_use]
    pub const fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub const fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }

    #[must_use]
    pub fn option_key(&self, index: usize) -> String {
        format!("{}::option::{index}", self.key)
    }

    #[must_use]
    pub fn list_key(&self) -> String {
        format!("{}::list", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: SelectPart, element: Element) -> Element {
        match part {
            SelectPart::Root => element
                .state_scope(SELECT_SCOPE)
                .active_state(SELECT_OPEN, self.open),
            SelectPart::Value => element.semantic_hidden(true),
            SelectPart::List => element.keyed(self.list_key()).semantics(
                Semantics::new(Role::ListBox)
                    .label(self.label.clone())
                    .orientation(Orientation::Vertical),
            ),
            SelectPart::Trigger => element
                .keyed(self.key.clone())
                .user_select(UserSelect::None)
                .interaction(
                    Interaction::default()
                        .focus_policy(argui_ui::FocusPolicy::TabStop)
                        .cursor(CursorIcon::Pointer)
                        .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                        .keyboard_activation(KeyboardActivation::EnterOrSpace),
                )
                .semantics(
                    Semantics::new(Role::Button)
                        .label(self.label.clone())
                        .state(SemanticState {
                            expanded: Some(self.open),
                            ..SemanticState::default()
                        })
                        .action(SemanticAction::Click)
                        .action(SemanticAction::Expand)
                        .action(SemanticAction::Collapse),
                ),
            SelectPart::Option(index) => {
                let option = self.options.get(index);
                let enabled = option.is_some_and(|option| option.enabled);
                let label = option
                    .map(|option| option.label.clone())
                    .unwrap_or_default();
                element
                    .keyed(self.option_key(index))
                    .user_select(UserSelect::None)
                    .active_state(SELECT_SELECTED, self.selected == Some(index))
                    .active_state(SELECT_HIGHLIGHTED, self.highlighted == index)
                    .interaction(
                        Interaction::default()
                            .enabled(enabled)
                            .focus_policy(if enabled && self.highlighted == index {
                                argui_ui::FocusPolicy::TabStop
                            } else if enabled {
                                argui_ui::FocusPolicy::Programmatic
                            } else {
                                argui_ui::FocusPolicy::None
                            })
                            .cursor(if enabled {
                                CursorIcon::Pointer
                            } else {
                                CursorIcon::NotAllowed
                            })
                            .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                            .keyboard_activation(KeyboardActivation::EnterOrSpace),
                    )
                    .semantics(
                        Semantics::new(Role::Option)
                            .label(label)
                            .state(SemanticState {
                                selected: self.selected == Some(index),
                                disabled: !enabled,
                                ..SemanticState::default()
                            })
                            .position_in_set((index + 1) as u32, self.options.len() as u32)
                            .action(SemanticAction::Click)
                            .action(SemanticAction::Focus),
                    )
            }
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<SelectAction> {
        if self.open
            && matches!(
                event.kind,
                UiEventKind::PointerOutside(_) | UiEventKind::DismissRequested
            )
            && event.target_key() == Some(self.list_key().as_str())
        {
            return Some(SelectAction::Close);
        }
        let event_key = event.target_key()?;
        if matches!(event.kind, UiEventKind::Click(_)) {
            if event_key == self.key {
                return Some(SelectAction::Toggle);
            }
            let index = event_key
                .strip_prefix(&format!("{}::option::", self.key))?
                .parse::<usize>()
                .ok()?;
            return self
                .options
                .get(index)
                .is_some_and(|option| option.enabled)
                .then_some(SelectAction::Select(index));
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed || !self.contains_key(event_key) {
            return None;
        }
        match &input.key {
            Key::Escape => Some(SelectAction::Close),
            Key::Enter => self
                .options
                .get(self.highlighted)
                .filter(|option| option.enabled)
                .map(|_| SelectAction::Select(self.highlighted)),
            Key::ArrowDown => self.next_enabled(true).map(SelectAction::Highlight),
            Key::ArrowUp => self.next_enabled(false).map(SelectAction::Highlight),
            Key::Home => self.first_enabled().map(SelectAction::Highlight),
            Key::End => self.last_enabled().map(SelectAction::Highlight),
            _ => None,
        }
    }

    pub fn search(
        &self,
        event: &UiEvent,
        search: &mut crate::Typeahead,
        now: std::time::Duration,
    ) -> Option<SelectAction> {
        if !self.contains_key(event.target_key()?) {
            return None;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        let Key::Character(query) = &input.key else {
            return None;
        };
        if input.state != KeyState::Pressed || input.modifiers.command() || input.modifiers.alt {
            return None;
        }
        search
            .search(
                query,
                now,
                Some(self.highlighted),
                self.options.len(),
                |index| {
                    self.options[index]
                        .enabled
                        .then_some(self.options[index].label.as_str())
                },
                crate::unicode_prefix,
            )
            .map(SelectAction::Highlight)
    }

    fn contains_key(&self, key: &str) -> bool {
        key == self.key
            || key == self.list_key()
            || key.starts_with(&format!("{}::option::", self.key))
    }

    fn first_enabled(&self) -> Option<usize> {
        self.options.iter().position(|option| option.enabled)
    }

    fn last_enabled(&self) -> Option<usize> {
        self.options.iter().rposition(|option| option.enabled)
    }

    fn next_enabled(&self, forward: bool) -> Option<usize> {
        if self.options.is_empty() {
            return None;
        }
        (1..=self.options.len())
            .map(|offset| {
                if forward {
                    (self.highlighted + offset) % self.options.len()
                } else {
                    (self.highlighted + self.options.len() - offset % self.options.len())
                        % self.options.len()
                }
            })
            .find(|index| self.options[*index].enabled)
    }
}
