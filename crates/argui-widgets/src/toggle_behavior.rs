use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Role, SemanticAction,
    SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind,
};

pub const TOGGLE_SCOPE: StateScopeId = StateScopeId::new("toggle");
pub const TOGGLE_CHECKED: StateName = StateName::new("checked");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TogglePart {
    Root,
    Indicator,
    Label,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToggleAction {
    Toggle,
}

#[derive(Clone, Debug)]
pub struct ToggleBehavior {
    key: String,
    label: String,
    role: Role,
    checked: bool,
    enabled: bool,
    position: Option<(u32, u32)>,
}

impl ToggleBehavior {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        role: Role,
        checked: bool,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            role,
            checked,
            enabled: true,
            position: None,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn position_in_set(mut self, position: u32, size: u32) -> Self {
        self.position = Some((position, size));
        self
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn decorate(&self, part: TogglePart, element: Element) -> Element {
        if part != TogglePart::Root {
            return element.semantic_hidden(true);
        }
        let mut semantics = Semantics::new(self.role)
            .label(self.label.clone())
            .state(SemanticState {
                checked: Some(self.checked),
                disabled: !self.enabled,
                ..SemanticState::default()
            })
            .action(SemanticAction::Click)
            .action(SemanticAction::Focus);
        if let Some((position, size)) = self.position {
            semantics = semantics.position_in_set(position, size);
        }
        element
            .keyed(self.key.clone())
            .state_scope(TOGGLE_SCOPE)
            .active_state(TOGGLE_CHECKED, self.checked)
            .interaction(
                Interaction::default()
                    .enabled(self.enabled)
                    .focusable(self.enabled)
                    .cursor(if self.enabled {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::NotAllowed
                    })
                    .gestures(GestureSet::NONE.tap())
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(semantics)
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<ToggleAction> {
        (self.enabled
            && event.key.as_deref() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Clicked))
        .then_some(ToggleAction::Toggle)
    }
}
