use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Role, SemanticAction,
    SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind, UserSelect,
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
    SetChecked(argui_ui::CheckedState),
}

#[derive(Clone, Debug)]
pub struct ToggleBehavior {
    key: String,
    label: String,
    role: Role,
    checked: argui_ui::CheckedState,
    enabled: bool,
    position: Option<(u32, u32)>,
}

impl ToggleBehavior {
    /// Creates toggle behavior with identity, accessible name, role and checked state.
    /// `key` identifies matching events; `label` is the accessible name.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        role: Role,
        checked: argui_ui::CheckedState,
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
    /// Sets whether this toggle can be activated; `enabled` controls interaction availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Supplies this option's one-based `position` in a set of `size` entries.
    pub const fn position_in_set(mut self, position: u32, size: u32) -> Self {
        self.position = Some((position, size));
        self
    }

    #[must_use]
    /// Returns whether this toggle is enabled.
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    /// Applies toggle interaction and semantics to `element` for `part`.
    pub fn decorate(&self, part: TogglePart, element: Element) -> Element {
        if part != TogglePart::Root {
            return element.semantic_hidden(true);
        }
        let mut semantics =
            Semantics::new(self.role)
                .label(self.label.clone())
                .state(SemanticState {
                    checked: Some(self.checked),
                    disabled: !self.enabled,
                    ..SemanticState::default()
                });
        if self.enabled {
            semantics = semantics
                .action(SemanticAction::Click)
                .action(SemanticAction::Focus);
        }
        if let Some((position, size)) = self.position {
            semantics = semantics.position_in_set(position, size);
        }
        element
            .keyed(self.key.clone())
            .user_select(UserSelect::None)
            .state_scope(TOGGLE_SCOPE)
            .active_state(
                TOGGLE_CHECKED,
                self.checked == argui_ui::CheckedState::Checked,
            )
            .interaction(
                Interaction::default()
                    .enabled(self.enabled)
                    .focus_policy(if self.enabled {
                        argui_ui::FocusPolicy::TabStop
                    } else {
                        argui_ui::FocusPolicy::None
                    })
                    .cursor(if self.enabled {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::NotAllowed
                    })
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(semantics)
    }

    #[must_use]
    /// Returns the checked-state action when `event` activates this enabled toggle.
    pub fn action(&self, event: &UiEvent) -> Option<ToggleAction> {
        (self.enabled
            && event.target_key() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_)))
        .then_some(ToggleAction::SetChecked(self.checked.toggled()))
    }
}
