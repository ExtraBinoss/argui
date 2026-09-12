use argui_core::{Key, KeyState};
use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Role, SemanticAction,
    SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind, UserSelect,
};

pub const POPOVER_SCOPE: StateScopeId = StateScopeId::new("popover");
pub const POPOVER_OPEN: StateName = StateName::new("open");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverPart {
    Root,
    Trigger,
    Content,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverAction {
    Toggle,
    Close,
}

#[derive(Clone, Debug)]
pub struct PopoverBehavior {
    key: String,
    label: String,
    open: bool,
}

impl PopoverBehavior {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, open: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
        }
    }

    #[must_use]
    pub fn content_key(&self) -> String {
        format!("{}::content", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: PopoverPart, element: Element) -> Element {
        match part {
            PopoverPart::Root => element
                .state_scope(POPOVER_SCOPE)
                .active_state(POPOVER_OPEN, self.open),
            PopoverPart::Trigger => element
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
            PopoverPart::Content => {
                let mut content = element
                    .keyed(self.content_key())
                    .semantics(Semantics::new(Role::Group).label(self.label.clone()));
                content.interaction.get_or_insert_with(Interaction::blocker);
                content
            }
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<PopoverAction> {
        if !self.open {
            return (event.target_key() == Some(self.key.as_str())
                && matches!(event.kind, UiEventKind::Click(_)))
            .then_some(PopoverAction::Toggle);
        }
        if matches!(event.kind, UiEventKind::PointerOutside(_))
            && event.target_key() == Some(self.content_key().as_str())
        {
            return Some(PopoverAction::Close);
        }
        if event.target_key() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_))
        {
            return Some(PopoverAction::Toggle);
        }
        matches!(
            &event.kind,
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed && input.key == Key::Escape
        )
        .then_some(PopoverAction::Close)
    }
}
