use crate::{Dialog, DialogAction, DialogBehavior, DialogPlacement, WidgetTheme};
use argui_ui::{Element, EventHandler, UiEvent, ValueHandler};

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
    open_handlers: Vec<ValueHandler<bool>>,
    dismiss_handlers: Vec<EventHandler>,
}

impl Sheet {
    /// Creates a controlled sheet with a labelled trigger and panel content.
    /// `key` identifies the sheet, `open` supplies visibility, and `label` names it accessibly.
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
            open_handlers: Vec::new(),
            dismiss_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Returns an open or close action when `event` targets the sheet.
    pub fn action(&self, event: &UiEvent) -> Option<DialogAction> {
        DialogBehavior::new(&self.key, &self.label, self.open).action(event)
    }

    /// Adds a callback receiving the requested controlled open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    /// Adds a callback for backdrop, close-control, or Escape dismissal.
    #[must_use]
    pub fn on_dismiss(mut self, handler: EventHandler) -> Self {
        self.dismiss_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the sheet using `theme` for the panel and backdrop.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let mut dialog = Dialog::new(self.key, self.label, self.open, self.trigger, self.content)
            .panel_width(self.width)
            .viewport_margin(0.0)
            .placement(match self.side {
                SheetSide::Left => DialogPlacement::Left,
                SheetSide::Right => DialogPlacement::Right,
                SheetSide::Top => DialogPlacement::Top,
                SheetSide::Bottom => DialogPlacement::Bottom,
            });
        for handler in self.open_handlers {
            dialog = dialog.on_open_change(handler);
        }
        for handler in self.dismiss_handlers {
            dialog = dialog.on_dismiss(handler);
        }
        dialog.build(theme)
    }
}
