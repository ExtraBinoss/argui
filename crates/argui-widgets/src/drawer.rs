use crate::{Button, DialogAction, Sheet, SheetSide, WidgetTheme};
use argui_core::Transform2D;
use argui_ui::{
    Element, EventHandler, GestureCapture, GestureKind, GesturePhase, GestureSet, Interaction,
    PanAxis, PanGesture, UiEvent, UiEventKind, ValueHandler,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawerAction {
    Open,
    Close,
    Drag(f32),
}

/// Bottom sheet with a dedicated drag handle. Swipes in scrollable content retain normal scrolling.
#[derive(Clone, Debug)]
pub struct Drawer {
    pub sheet: Sheet,
    pub offset: f32,
    pub dismiss_threshold: f32,
    pub handle_label: String,
    pub close_label: String,
}

impl Drawer {
    /// Creates a bottom drawer with a labelled trigger and panel content.
    /// `key` identifies it, `open` sets current visibility, and `label` supplies its accessible name.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        trigger: Element,
        content: Element,
    ) -> Self {
        let mut sheet = Sheet::new(key, label, open, trigger, content);
        sheet.side = SheetSide::Bottom;
        Self {
            sheet,
            offset: 0.0,
            dismiss_threshold: 100.0,
            handle_label: "Drag to close".into(),
            close_label: "Close".into(),
        }
    }

    #[must_use]
    /// Returns an open, close or drag action resulting from `event`.
    pub fn action(&self, event: &UiEvent) -> Option<DrawerAction> {
        if self.sheet.open
            && event.target_key() == Some(format!("{}::handle", self.sheet.key).as_str())
            && let UiEventKind::Gesture(gesture) = event.kind
            && let GestureKind::Pan {
                total, velocity, ..
            } = gesture.kind
        {
            return Some(match gesture.phase {
                GesturePhase::Cancelled => DrawerAction::Drag(0.0),
                GesturePhase::Ended
                    if total.y >= self.dismiss_threshold.max(1.0)
                        || (total.y > 0.0 && velocity.y > 600.0) =>
                {
                    DrawerAction::Close
                }
                GesturePhase::Ended => DrawerAction::Drag(0.0),
                GesturePhase::Started | GesturePhase::Changed => {
                    DrawerAction::Drag(total.y.max(0.0))
                }
            });
        }
        self.sheet.action(event).map(|action| match action {
            DialogAction::Open => DrawerAction::Open,
            DialogAction::Close => DrawerAction::Close,
        })
    }

    /// Adds a callback receiving the requested controlled open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.sheet = self.sheet.on_open_change(handler);
        self
    }

    /// Adds a callback for explicit or gesture dismissal.
    #[must_use]
    pub fn on_dismiss(mut self, handler: EventHandler) -> Self {
        self.sheet = self.sheet.on_dismiss(handler);
        self
    }

    #[must_use]
    /// Builds the drawer using `theme` for its trigger and panel styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let mut sheet = self.sheet;
        let handle = Element::row([Element::container([])
            .width(argui_ui::length(48.0))
            .height(argui_ui::length(5.0))
            .background(theme.muted_foreground)
            .radius(argui_paint::CornerRadii::all(3.0))
            .semantic_hidden(true)])
        .keyed(format!("{}::handle", sheet.key))
        .height(argui_ui::length(28.0))
        .justify_content(argui_ui::JustifyContent::CENTER)
        .align_items(argui_ui::AlignItems::CENTER)
        .interaction(
            Interaction::default().gestures(
                GestureSet::default().pan(
                    PanGesture::default()
                        .axis(PanAxis::Vertical)
                        .capture(GestureCapture::OnPress),
                ),
            ),
        )
        .semantics(argui_ui::Semantics::new(argui_ui::Role::Group).label(self.handle_label));
        let close = Button::new(
            format!("{}::close", sheet.key),
            self.close_label,
            theme.outline_button(),
        )
        .build();
        sheet.content = Element::column([handle, sheet.content, close]).gap(12.0);
        let mut root = sheet.build(theme);
        if let Some(overlay) = root.children.get_mut(1) {
            overlay.children[1].transform =
                Transform2D::IDENTITY.translate(0.0, self.offset.max(0.0));
        }
        root
    }
}
