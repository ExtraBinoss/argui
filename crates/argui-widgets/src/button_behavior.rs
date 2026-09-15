use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Role, SemanticAction,
    SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind, UserSelect,
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
    /// Creates enabled, idle button behavior with the given identity and accessible label.
    ///
    /// `key` identifies matching events; `label` is exposed to accessibility consumers.
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
    /// Sets whether interactions with the button are enabled.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Sets whether the button is busy and must reject activation.
    pub const fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    #[must_use]
    /// Adds button semantics and interaction behavior to `element`.
    /// `part` selects whether this element is the button's trigger or content.
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
            .user_select(UserSelect::None)
            .state_scope(BUTTON_SCOPE)
            .active_state(BUTTON_BUSY, self.busy)
            .interaction(
                Interaction::default()
                    .enabled(enabled)
                    .focus_policy(if enabled {
                        argui_ui::FocusPolicy::TabStop
                    } else {
                        argui_ui::FocusPolicy::None
                    })
                    .cursor(cursor)
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
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
    /// Returns an activation action when `event` clicks this enabled, idle button.
    pub fn action(&self, event: &UiEvent) -> Option<ButtonAction> {
        (event.target_key() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_))
            && self.enabled
            && !self.busy)
            .then_some(ButtonAction::Activate)
    }
}
