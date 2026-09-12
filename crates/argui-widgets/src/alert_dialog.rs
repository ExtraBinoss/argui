use crate::{Button, ButtonBehavior, Dialog, DialogAction, DialogBehavior, WidgetTheme};
use argui_ui::{Element, InitialFocus, UiEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertDialogAction {
    Open,
    Cancel,
    Confirm,
}

/// Confirmation dialog. Initial focus goes to Cancel; outside clicks never confirm or close.
#[derive(Clone, Debug)]
pub struct AlertDialog {
    pub key: String,
    pub title: String,
    pub description: String,
    pub open: bool,
    pub trigger: Element,
    pub cancel_label: String,
    pub confirm_label: String,
    pub confirm_enabled: bool,
}

impl AlertDialog {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        open: bool,
        trigger: Element,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: description.into(),
            open,
            trigger,
            cancel_label: "Cancel".into(),
            confirm_label: "Continue".into(),
            confirm_enabled: true,
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<AlertDialogAction> {
        if self.open
            && ButtonBehavior::new(format!("{}::confirm", self.key), &self.confirm_label)
                .enabled(self.confirm_enabled)
                .action(event)
                .is_some()
        {
            return Some(AlertDialogAction::Confirm);
        }
        match DialogBehavior::new(&self.key, &self.title, self.open)
            .dismiss_on_backdrop(false)
            .action(event)?
        {
            DialogAction::Open => Some(AlertDialogAction::Open),
            DialogAction::Close if self.open => Some(AlertDialogAction::Cancel),
            DialogAction::Close => None,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let close = format!("{}::close", self.key);
        let title_key = format!("{}::title", self.key);
        let description_key = format!("{}::description", self.key);
        let content = Element::column([
            crate::Typography::new(&self.title, crate::TypographyVariant::Heading(2))
                .build(theme)
                .keyed(&title_key),
            crate::Typography::new(self.description, crate::TypographyVariant::Paragraph)
                .build(theme)
                .keyed(&description_key),
            Element::row([
                Button::new(&close, self.cancel_label, theme.outline_button()).build(),
                Button::new(
                    format!("{}::confirm", self.key),
                    self.confirm_label,
                    theme.destructive_button(),
                )
                .enabled(self.confirm_enabled)
                .build(),
            ])
            .gap(8.0)
            .flex_wrap(argui_ui::FlexWrap::Wrap),
        ])
        .gap(16.0);
        let mut root = Dialog::new(self.key, self.title, self.open, self.trigger, content)
            .alert(true)
            .initial_focus(InitialFocus::Target(close.into()))
            .build(theme);
        if let Some(overlay) = root.children.get_mut(1) {
            let panel = &mut overlay.children[1];
            *panel = panel
                .clone()
                .labelled_by([title_key])
                .described_by([description_key]);
        }
        root
    }
}
