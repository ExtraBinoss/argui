use crate::{Menu, MenuResponse, WidgetTheme};
use argui_core::{Key, KeyState, Point, Rect, Size};
use argui_ui::{Element, FloatingPlacement, PortalTarget, UiEvent, UiEventKind, ValueHandler};

/// Context-menu presentation reuses the same entries and keyboard policy as Menu.
pub struct ContextMenu {
    pub menu: Menu,
    pub position: Option<Point>,
}

impl ContextMenu {
    /// Adds a handler that receives the stable id of an activated menu item.
    #[must_use]
    pub fn on_action(mut self, handler: ValueHandler<String>) -> Self {
        self.menu = self.menu.on_action(handler);
        self
    }

    /// Adds a handler receiving the requested menu open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.menu = self.menu.on_open_change(handler);
        self
    }

    /// Sets the surface policy used by the context menu's overlay.
    #[must_use]
    pub fn surface(mut self, surface: argui_ui::OverlaySurface) -> Self {
        self.menu = self.menu.surface(surface);
        self
    }

    /// Builds the menu attached to `target`, using `theme` for styling.
    pub fn build(&self, target: Element, theme: &WidgetTheme) -> Element {
        let mut root = self.menu.build(target, theme);
        if let Some(position) = self.position
            && let Some(content) = root.children.get_mut(1)
            && let Some(portal) = &mut content.portal
        {
            portal.target = PortalTarget::Rect {
                bounds: Rect::new(position, Size::default()),
                placement: FloatingPlacement::default().offset(2.0),
            };
        }
        root
    }

    /// Returns the pointer anchor, or None for keyboard anchoring to the target.
    /// `event` is the UI event checked for a context-menu opening gesture.
    pub fn open_action(&self, event: &UiEvent) -> Option<Option<Point>> {
        if event.target_key() != Some(self.menu.key.as_str()) {
            return None;
        }
        match &event.kind {
            UiEventKind::ContextMenu { position, .. } => Some(Some(*position)),
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed
                    && (input.key == Key::ContextMenu
                        || (input.modifiers.shift && input.key == Key::Function(10))) =>
            {
                Some(None)
            }
            _ => None,
        }
    }
    /// Returns a menu response for `event`, excluding clicks on the attached target.
    pub fn response(&self, event: &UiEvent) -> Option<MenuResponse> {
        if event.target_key() == Some(self.menu.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_))
        {
            return None;
        }
        self.menu.response(event)
    }
}
