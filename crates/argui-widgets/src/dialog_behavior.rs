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
    initial_focus: Option<InitialFocus>,
    dismiss_on_backdrop: bool,
    role: Role,
}

impl DialogBehavior {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, open: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
            initial_focus: None,
            dismiss_on_backdrop: true,
            role: Role::Dialog,
        }
    }

    #[must_use]
    pub fn initial_focus(mut self, focus: InitialFocus) -> Self {
        self.initial_focus = Some(focus);
        self
    }

    #[must_use]
    pub const fn dismiss_on_backdrop(mut self, dismiss: bool) -> Self {
        self.dismiss_on_backdrop = dismiss;
        self
    }

    #[must_use]
    pub const fn alert(mut self, alert: bool) -> Self {
        self.role = if alert {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        self
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
            DialogPart::Backdrop => element.keyed(self.backdrop_key()).interaction(
                Interaction::blocker()
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default())),
            ),
            DialogPart::Panel => element
                .keyed(self.panel_key())
                .interaction(Interaction::blocker().focus_policy(argui_ui::FocusPolicy::TabStop))
                .semantics(
                    Semantics::new(self.role)
                        .label(self.label.clone())
                        .action(SemanticAction::Focus),
                ),
            DialogPart::Overlay => element.focus_scope(FocusScope::modal(
                self.initial_focus
                    .clone()
                    .unwrap_or_else(|| InitialFocus::Target(self.panel_key().into())),
            )),
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<DialogAction> {
        let event_key = event.target_key();
        if matches!(event.kind, UiEventKind::Click(_)) {
            if event_key == Some(self.trigger_key().as_str()) {
                return Some(DialogAction::Open);
            }
            if event_key == Some(self.close_key().as_str())
                || (self.dismiss_on_backdrop && event_key == Some(self.backdrop_key().as_str()))
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
