use argui_core::{Key, KeyState};
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_ui::{
    AlignItems, Display, Element, FocusScope, GestureSet, InitialFocus, Interaction,
    JustifyContent, Role, SemanticAction, Semantics, Sides, UiEvent, UiEventKind, length, percent,
};

use crate::WidgetTheme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogAction {
    Open,
    Close,
}

#[derive(Clone, Debug)]
pub struct Dialog {
    key: String,
    label: String,
    open: bool,
    trigger: Element,
    content: Element,
}

impl Dialog {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        trigger: Element,
        content: Element,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
            trigger,
            content,
        }
    }

    #[must_use]
    pub fn trigger_key(key: &str) -> String {
        format!("{key}::trigger")
    }

    #[must_use]
    pub fn close_key(key: &str) -> String {
        format!("{key}::close")
    }

    #[must_use]
    pub fn panel_key(key: &str) -> String {
        format!("{key}::panel")
    }

    #[must_use]
    pub fn backdrop_key(key: &str) -> String {
        format!("{key}::backdrop")
    }

    #[must_use]
    pub fn action(key: &str, event: &UiEvent) -> Option<DialogAction> {
        let event_key = event.key.as_deref();
        if matches!(event.kind, UiEventKind::Clicked) {
            if event_key == Some(Self::trigger_key(key).as_str()) {
                return Some(DialogAction::Open);
            }
            if event_key == Some(Self::close_key(key).as_str())
                || event_key == Some(Self::backdrop_key(key).as_str())
            {
                return Some(DialogAction::Close);
            }
        }
        if matches!(
            &event.kind,
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed && input.key == Key::Escape
        ) {
            return Some(DialogAction::Close);
        }
        None
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let trigger = self.trigger.keyed(Self::trigger_key(&self.key));
        let overlay = self.open.then(|| {
            let backdrop = Element::container([])
                .keyed(Self::backdrop_key(&self.key))
                .absolute(Sides::length(0.0))
                .width(percent(1.0))
                .height(percent(1.0))
                .background(argui_core::Color::rgba(0.0, 0.0, 0.0, 0.52))
                .interaction(Interaction::blocker().gestures(GestureSet::NONE.tap()));
            let panel = Element::column([self.content])
                .keyed(Self::panel_key(&self.key))
                .width(length(480.0))
                .max_width(percent(0.90))
                .padding(Sides::length(24.0))
                .gap(18.0)
                .paint_style(PaintStyle::new(
                    QuadStyle::solid(theme.popover)
                        .border(Border::all(1.0, theme.border))
                        .radius(CornerRadii::all(12.0)),
                ))
                .interaction(Interaction::blocker().focusable(true))
                .semantics(
                    Semantics::new(Role::Dialog)
                        .label(self.label.clone())
                        .action(SemanticAction::Focus),
                );
            Element::container([backdrop, panel])
                .absolute(Sides::length(0.0))
                .width(percent(1.0))
                .height(percent(1.0))
                .display(Display::Flex)
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::CENTER)
                .focus_scope(FocusScope::modal(InitialFocus::Target(
                    Self::panel_key(&self.key).into(),
                )))
                .z_index(2_000)
        });
        Element::container(std::iter::once(trigger).chain(overlay))
    }
}
