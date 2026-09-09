use crate::{Menu, MenuResponse, WidgetTheme};
use argui_core::{Key, KeyState, Point, Rect, Size};
use argui_ui::{DismissPolicy, Element, FloatingPlacement, UiEvent, UiEventKind, WindowLayer};

/// Context-menu presentation reuses the same entries and keyboard policy as Menu.
pub struct ContextMenu {
    pub menu: Menu,
    pub position: Option<Point>,
}

impl ContextMenu {
    pub fn build(&self, target: Element, theme: &WidgetTheme) -> Element {
        let mut children = vec![target.keyed(&self.menu.key)];
        if self.menu.open {
            let placement = FloatingPlacement::default();
            let mut content = self
                .menu
                .content(theme)
                .keyed(format!("{}::content", self.menu.key))
                .padding(argui_ui::Sides::length(6.0))
                .background(theme.background)
                .focus_scope(Menu::focus_scope());
            content = if let Some(position) = self.position {
                content.rect_portal(
                    WindowLayer::Popover,
                    Rect::new(position, Size::default()),
                    placement,
                )
            } else {
                content.anchored_portal(WindowLayer::Popover, &self.menu.key, placement)
            };
            children.push(content.portal_dismiss(DismissPolicy::OutsidePointer));
        }
        Element::container(children).semantic_scope()
    }

    /// Returns the pointer anchor, or None for keyboard anchoring to the target.
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
    pub fn response(&self, event: &UiEvent) -> Option<MenuResponse> {
        if event.target_key() == Some(self.menu.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_))
        {
            return None;
        }
        self.menu.response(event)
    }
}
