use crate::{Button, ButtonBehavior, ScrollArea, WidgetTheme};
use argui_core::Point;
use argui_ui::{Element, ScrollAlignment, ScrollRequest, UiEvent, UiEventKind};

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
    pub fn prepended(&mut self, key: &str, added_height: f32) -> ScrollRequest {
        let added = added_height.max(0.0);
        self.offset += added;
        self.maximum += added;
        ScrollRequest::offset(key, Point::new(0.0, self.offset))
    }

    pub fn latest(&mut self, key: &str) -> ScrollRequest {
        self.following = true;
        self.unread = 0;
        self.offset = self.maximum;
        ScrollRequest::offset(key, Point::new(0.0, self.maximum))
    }

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
}

impl MessageScroller {
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
        }
    }

    #[must_use]
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
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let history = self.has_earlier.then(|| {
            Button::new(
                format!("{}::earlier", self.key),
                self.earlier_label,
                theme.ghost_button(),
            )
            .enabled(!self.loading)
            .build()
        });
        let viewport = ScrollArea::new(
            &self.key,
            self.label,
            self.height,
            Element::column(history.into_iter().chain([self.content])).gap(12.0),
        )
        .build(theme);
        let latest = (!self.state.following).then(|| {
            Button::new(
                format!("{}::latest", self.key),
                format!("{} ({})", self.latest_label, self.state.unread),
                theme.outline_button(),
            )
            .build()
        });
        Element::column(std::iter::once(viewport).chain(latest)).gap(8.0)
    }
}
