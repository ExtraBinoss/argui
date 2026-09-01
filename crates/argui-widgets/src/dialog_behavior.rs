use argui_core::{Key, KeyState};
use argui_ui::{
    Element, FocusScope, GestureSet, InitialFocus, Interaction, Role, SemanticAction, Semantics,
    StateName, StateScopeId, UiEvent, UiEventKind,
};

pub const DIALOG_SCOPE: StateScopeId = StateScopeId::new("dialog");
pub const DIALOG_OPEN: StateName = StateName::new("open");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogPart {
    Root,
    Trigger,
    Overlay,
    Backdrop,
    Panel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogAction {
    Open,
    Close,
}

#[derive(Clone, Debug)]
pub struct DialogBehavior {
    key: String,
    label: String,
    open: bool,
}

impl DialogBehavior {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, open: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
        }
    }

    #[must_use]
    pub fn trigger_key(&self) -> String {
        format!("{}::trigger", self.key)
    }

    #[must_use]
    pub fn close_key(&self) -> String {
        format!("{}::close", self.key)
    }

    #[must_use]
    pub fn panel_key(&self) -> String {
        format!("{}::panel", self.key)
    }

    #[must_use]
    pub fn backdrop_key(&self) -> String {
        format!("{}::backdrop", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: DialogPart, element: Element) -> Element {
        match part {
            DialogPart::Root => element
                .state_scope(DIALOG_SCOPE)
                .active_state(DIALOG_OPEN, self.open),
            DialogPart::Trigger => element.keyed(self.trigger_key()),
            DialogPart::Backdrop => element
                .keyed(self.backdrop_key())
                .interaction(Interaction::blocker().gestures(GestureSet::NONE.tap())),
            DialogPart::Panel => element
                .keyed(self.panel_key())
                .interaction(Interaction::blocker().focusable(true))
                .semantics(
                    Semantics::new(Role::Dialog)
                        .label(self.label.clone())
                        .action(SemanticAction::Focus),
                ),
            DialogPart::Overlay => element.focus_scope(FocusScope::modal(InitialFocus::Target(
                self.panel_key().into(),
            ))),
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<DialogAction> {
        let event_key = event.key.as_deref();
        if matches!(event.kind, UiEventKind::Clicked) {
            if event_key == Some(self.trigger_key().as_str()) {
                return Some(DialogAction::Open);
            }
            if event_key == Some(self.close_key().as_str())
                || event_key == Some(self.backdrop_key().as_str())
            {
                return Some(DialogAction::Close);
            }
        }
        matches!(
            &event.kind,
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed && input.key == Key::Escape
        )
        .then_some(DialogAction::Close)
    }
}
