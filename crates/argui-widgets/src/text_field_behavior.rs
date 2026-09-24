use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, Role, SemanticAction, SemanticState,
    SemanticValue, Semantics, StateName, StateScopeId,
};

pub const TEXT_FIELD_SCOPE: StateScopeId = StateScopeId::new("text-field");
pub const TEXT_FIELD_INVALID: StateName = StateName::new("invalid");
pub const TEXT_FIELD_READ_ONLY: StateName = StateName::new("read-only");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFieldPart {
    Root,
    Editor,
    Decoration,
}

#[derive(Clone, Debug)]
pub struct TextFieldBehavior {
    key: String,
    label: String,
    description: Option<String>,
    value: String,
    role: Role,
    enabled: bool,
    read_only: bool,
    invalid: bool,
}

impl TextFieldBehavior {
    /// Creates text-field behavior with an accessible label and current value.
    /// `key` identifies the field, `value` is exposed to accessibility consumers, and `role` sets its semantic role.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        role: Role,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: None,
            value: value.into(),
            role,
            enabled: true,
            read_only: false,
            invalid: false,
        }
    }

    #[must_use]
    /// Sets supplementary accessible description text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    /// Sets whether the field can be focused and edited; `enabled` controls availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Sets whether the field is focusable but prevents editing; `read_only` controls editability.
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    #[must_use]
    /// Sets whether the field is marked invalid.
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[must_use]
    /// Applies text-field semantics and behavior to `element` for `part`.
    pub fn decorate(&self, part: TextFieldPart, element: Element) -> Element {
        match part {
            TextFieldPart::Decoration => element.semantic_hidden(true),
            TextFieldPart::Root => element
                .state_scope(TEXT_FIELD_SCOPE)
                .active_state(TEXT_FIELD_INVALID, self.invalid)
                .active_state(TEXT_FIELD_READ_ONLY, self.read_only),
            TextFieldPart::Editor => {
                let mut semantics = Semantics::new(self.role)
                    .label(self.label.clone())
                    .value(SemanticValue::Text(self.value.clone()))
                    .state(SemanticState {
                        disabled: !self.enabled,
                        read_only: self.read_only,
                        invalid: self.invalid,
                        ..SemanticState::default()
                    });
                if self.enabled {
                    semantics = semantics.action(SemanticAction::Focus);
                    if !self.read_only {
                        semantics = semantics.action(SemanticAction::SetValue);
                    }
                }
                if let Some(description) = &self.description {
                    semantics = semantics.description(description.clone());
                }
                element
                    .keyed(self.key.clone())
                    .interaction(
                        Interaction::default()
                            .enabled(self.enabled)
                            .focus_policy(if self.enabled {
                                argui_ui::FocusPolicy::TabStop
                            } else {
                                argui_ui::FocusPolicy::None
                            })
                            .cursor(CursorIcon::Text)
                            .gestures(
                                GestureSet::default()
                                    .tap(argui_ui::TapGesture::default())
                                    .pan(argui_ui::PanGesture::default()),
                            ),
                    )
                    .semantics(semantics)
            }
        }
    }
}
