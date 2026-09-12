use crate::{Dialog, DialogAction, DialogBehavior, DialogPlacement, WidgetTheme};
use argui_ui::{Element, UiEvent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// Modal edge panel with scroll containment, focus restoration and explicit close control.
#[derive(Clone, Debug)]
pub struct Sheet {
    pub key: String,
    pub label: String,
    pub open: bool,
    pub trigger: Element,
    pub content: Element,
    pub side: SheetSide,
    pub width: f32,
}

impl Sheet {
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
            side: SheetSide::Right,
            width: 400.0,
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<DialogAction> {
        DialogBehavior::new(&self.key, &self.label, self.open).action(event)
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        Dialog::new(self.key, self.label, self.open, self.trigger, self.content)
            .panel_width(self.width)
            .viewport_margin(0.0)
            .placement(match self.side {
                SheetSide::Left => DialogPlacement::Left,
                SheetSide::Right => DialogPlacement::Right,
                SheetSide::Top => DialogPlacement::Top,
                SheetSide::Bottom => DialogPlacement::Bottom,
            })
            .build(theme)
    }
}
