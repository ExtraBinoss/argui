use crate::{Button, ButtonBehavior, ScrollArea, WidgetTheme};
use argui_core::Point;
use argui_ui::{Element, EventHandler, ScrollAlignment, ScrollRequest, UiEvent, UiEventKind};

/// Retained reading position. Supply measured maximum offsets after layout changes.
#[derive(Clone, Debug, PartialEq)]
pub struct MessageScrollState {
    pub following: bool,
    pub unread: usize,
    offset: f32,
    maximum: f32,
}

impl Default for MessageScrollState {
    fn default() -> Self {
        Self {
            following: true,
            unread: 0,
            offset: 0.0,
            maximum: 0.0,
        }
    }
}

impl MessageScrollState {
    /// Updates scroll retention from `event` for the viewport identified by `viewport_key`.
    /// `maximum` is the greatest valid vertical scroll offset.
    /// User scrolls or reading interactions suspend following; reaching the live edge resumes it.
    pub fn observe(&mut self, event: &UiEvent, viewport_key: &str, maximum: f32) {
        if event.target_key() == Some(viewport_key)
            && let UiEventKind::Scrolled { delta, offset } = event.kind
        {
            self.offset = offset.y.max(0.0);
            self.maximum = maximum.max(0.0);
            self.following = delta.y >= 0.0 && self.maximum - self.offset <= 8.0;
            if self.following {
                self.unread = 0;
            }
        }
        if matches!(
            event.kind,
            UiEventKind::DocumentSelectionChanged { .. } | UiEventKind::Click(_)
        ) || matches!(&event.kind, UiEventKind::KeyInput(input) if input.state == argui_core::KeyState::Pressed)
        {
            self.following = false;
        }
    }

    /// Call once per append (zero for a streaming update), after measuring the new content.
    /// Handles `count` appended messages in the scroller identified by `key`.
    /// `maximum` is the new greatest scroll offset.
    pub fn appended(&mut self, key: &str, count: usize, maximum: f32) -> Option<ScrollRequest> {
        self.maximum = maximum.max(0.0);
        if self.following {
            Some(self.latest(key))
        } else {
            self.unread = self.unread.saturating_add(count);
            None
        }
    }

    /// Preserve the visible content when measured history is inserted above it.
    /// Preserves visible content after insertion; `key` identifies the scroller and `added_height` is the inserted height.
    pub fn prepended(&mut self, key: &str, added_height: f32) -> ScrollRequest {
        let added = added_height.max(0.0);
        self.offset += added;
        self.maximum += added;
        ScrollRequest::offset(key, Point::new(0.0, self.offset))
    }

    /// Requests scrolling to the latest edge of the viewport identified by `key`.
    pub fn latest(&mut self, key: &str) -> ScrollRequest {
        self.following = true;
        self.unread = 0;
        self.offset = self.maximum;
        ScrollRequest::offset(key, Point::new(0.0, self.maximum))
    }

    /// Requests a scroll to the message identified by `message_key`.
    pub fn jump(&mut self, message_key: impl Into<String>) -> ScrollRequest {
        self.following = false;
        ScrollRequest::reveal(message_key.into())
            .align(ScrollAlignment::Nearest, ScrollAlignment::Start)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageScrollerAction {
    LoadEarlier,
    Latest,
}

/// A conversation viewport with explicit history and live-edge controls.
#[derive(Clone, Debug)]
pub struct MessageScroller {
    pub key: String,
    pub label: String,
    pub content: Element,
    pub height: f32,
    pub state: MessageScrollState,
    pub has_earlier: bool,
    pub loading: bool,
    pub earlier_label: String,
    pub latest_label: String,
    earlier_handlers: Vec<EventHandler>,
    latest_handlers: Vec<EventHandler>,
}

impl MessageScroller {
    /// Creates a scroller identified by `key`, named accessibly by `label`, around `content`.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            content,
            height: 360.0,
            state: MessageScrollState::default(),
            has_earlier: false,
            loading: false,
            earlier_label: "Load earlier messages".into(),
            latest_label: "Jump to latest".into(),
            earlier_handlers: Vec::new(),
            latest_handlers: Vec::new(),
        }
    }

    /// Adds a handler invoked when available earlier history is requested.
    #[must_use]
    pub fn on_load_earlier(mut self, handler: EventHandler) -> Self {
        self.earlier_handlers.push(handler);
        self
    }

    /// Adds a handler invoked when the user requests the latest message.
    #[must_use]
    pub fn on_latest(mut self, handler: EventHandler) -> Self {
        self.latest_handlers.push(handler);
        self
    }

    #[must_use]
    /// Returns a message-scroller action when `event` targets its controls.
    pub fn action(&self, event: &UiEvent) -> Option<MessageScrollerAction> {
        if self.has_earlier
            && ButtonBehavior::new(format!("{}::earlier", self.key), &self.earlier_label)
                .enabled(!self.loading)
                .action(event)
                .is_some()
        {
            return Some(MessageScrollerAction::LoadEarlier);
        }
        (!self.state.following
            && ButtonBehavior::new(format!("{}::latest", self.key), &self.latest_label)
                .action(event)
                .is_some())
        .then_some(MessageScrollerAction::Latest)
    }

    #[must_use]
    /// Builds the message scroller and its controls using `theme`.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let history = self.has_earlier.then(|| {
            let mut button = Button::new(
                format!("{}::earlier", self.key),
                &self.earlier_label,
                theme.ghost_button(),
            )
            .enabled(!self.loading);
            if !self.loading {
                for handler in &self.earlier_handlers {
                    button = button.on_click(*handler);
                }
            }
            button.build()
        });
        let viewport = ScrollArea::new(
            &self.key,
            self.label,
            self.height,
            Element::column(history.into_iter().chain([self.content])).gap(12.0),
        )
        .build(theme);
        let latest = (!self.state.following).then(|| {
            let mut button = Button::new(
                format!("{}::latest", self.key),
                format!("{} ({})", self.latest_label, self.state.unread),
                theme.outline_button(),
            );
            for handler in &self.latest_handlers {
                button = button.on_click(*handler);
            }
            button.build()
        });
        Element::column(std::iter::once(viewport).chain(latest)).gap(8.0)
    }
}
