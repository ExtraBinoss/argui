use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Role, SemanticAction,
    SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind,
};

pub const BUTTON_SCOPE: StateScopeId = StateScopeId::new("button");
pub const BUTTON_BUSY: StateName = StateName::new("busy");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonPart {
    Root,
    Content,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonAction {
    Activate,
}

#[derive(Clone, Debug)]
pub struct ButtonBehavior {
    key: String,
    label: String,
    enabled: bool,
    busy: bool,
}

impl ButtonBehavior {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            enabled: true,
            busy: false,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    #[must_use]
    pub fn decorate(&self, part: ButtonPart, element: Element) -> Element {
        if part == ButtonPart::Content {
            return element.semantic_hidden(true);
        }
        let enabled = self.enabled && !self.busy;
        let cursor = if !self.enabled {
            CursorIcon::NotAllowed
        } else if self.busy {
            CursorIcon::Progress
        } else {
            CursorIcon::Pointer
        };
        element
            .keyed(self.key.clone())
            .state_scope(BUTTON_SCOPE)
            .active_state(BUTTON_BUSY, self.busy)
            .interaction(
                Interaction::default()
                    .enabled(enabled)
                    .focusable(enabled)
                    .cursor(cursor)
                    .gestures(GestureSet::NONE.tap())
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(
                Semantics::new(Role::Button)
                    .label(self.label.clone())
                    .state(SemanticState {
                        disabled: !enabled,
                        busy: self.busy,
                        ..SemanticState::default()
                    })
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            )
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<ButtonAction> {
        (event.key.as_deref() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Clicked)
            && self.enabled
            && !self.busy)
            .then_some(ButtonAction::Activate)
    }
}
