use crate::{Popover, WidgetTheme};
use argui_core::{Key, KeyState, PointerKind, PointerPhase};
use argui_ui::{Element, UiEvent, UiEventKind, ValueHandler};
use std::time::Duration;

/// Delayed rich preview. It opens on hover or focus without moving focus into its panel.
#[derive(Clone, Debug)]
pub struct HoverCard {
    pub key: String,
    pub label: String,
    pub trigger: Element,
    pub content: Element,
    pub open: bool,
    open_handlers: Vec<ValueHandler<bool>>,
}

impl HoverCard {
    /// Creates a preview card controlled by `open`, with a trigger and panel content.
    /// `key` identifies it and `label` supplies its accessible name.
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
            trigger,
            content,
            open,
            open_handlers: Vec::new(),
        }
    }

    /// Adds a callback receiving immediate click and dismissal open-state requests.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the hover card using `theme` for its panel styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let mut popover =
            Popover::new(&self.key, self.label, self.open, self.trigger, self.content);
        for handler in self.open_handlers {
            popover = popover.on_open_change(handler);
        }
        let mut root = popover.build(theme);
        if self.open {
            root.children[0] = root.children[0]
                .clone()
                .controls([format!("{}::content", self.key)]);
        }
        root
    }
}

/// Retain this state and schedule `advance` at `next_deadline`, without idle polling.
/// Forward capture-phase Focus/Blur/Key from the panel so arbitrary descendant keys are supported.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverCardState {
    key: String,
    hovered: [bool; 2],
    focused: [bool; 2],
    open: bool,
    deadline: Option<(Duration, bool)>,
    pub open_delay: Duration,
    pub close_delay: Duration,
}

impl HoverCardState {
    /// Creates retained hover/focus state for the element identified by `key`.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            hovered: [false; 2],
            focused: [false; 2],
            open: false,
            deadline: None,
            open_delay: Duration::from_millis(500),
            close_delay: Duration::from_millis(200),
        }
    }

    #[must_use]
    /// Returns whether the card is currently open.
    pub const fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    /// Returns the pending open or close deadline, if one exists.
    pub fn next_deadline(&self) -> Option<Duration> {
        self.deadline.map(|(time, _)| time)
    }

    /// Clears pointer/focus retention and closes the card immediately.
    pub fn reset(&mut self) {
        self.hovered = [false; 2];
        self.focused = [false; 2];
        self.open = false;
        self.deadline = None;
    }

    /// Applies `event` at monotonic time `now`; returns whether retained hover/focus state changed.
    pub fn update(&mut self, event: &UiEvent, now: Duration) -> bool {
        let content_key = format!("{}::content", self.key);
        let panel = event.target_key() == Some(content_key.as_str())
            || event.current_key() == Some(content_key.as_str());
        let trigger = event.target_key() == Some(self.key.as_str());
        if !panel && !trigger {
            return false;
        }
        let before = self.clone();
        let index = usize::from(panel);
        match &event.kind {
            UiEventKind::Pointer(pointer) if pointer.kind != PointerKind::Touch => {
                match pointer.phase {
                    PointerPhase::Entered => self.hovered[index] = true,
                    PointerPhase::Left | PointerPhase::Cancelled => self.hovered[index] = false,
                    _ => return false,
                }
            }
            UiEventKind::Focused => self.focused[index] = true,
            UiEventKind::Blurred => self.focused[index] = false,
            UiEventKind::Click(_) if trigger => {
                self.open = !self.open;
                self.deadline = None;
                return *self != before;
            }
            UiEventKind::DismissRequested | UiEventKind::PointerOutside(_) => {
                self.reset();
                return *self != before;
            }
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed && input.key == Key::Escape =>
            {
                self.reset();
                return *self != before;
            }
            _ => return false,
        }
        let desired = self.hovered.iter().chain(&self.focused).any(|value| *value);
        if desired == self.open {
            self.deadline = None;
        } else if self.deadline.is_none_or(|(_, open)| open != desired) {
            self.deadline = Some((
                now.saturating_add(if desired {
                    self.open_delay
                } else {
                    self.close_delay
                }),
                desired,
            ));
        }
        *self != before
    }

    /// Applies a pending deadline at monotonic time `now`; returns whether visibility changed.
    pub fn advance(&mut self, now: Duration) -> bool {
        let Some((deadline, open)) = self.deadline else {
            return false;
        };
        if now < deadline {
            return false;
        }
        self.deadline = None;
        let changed = self.open != open;
        self.open = open;
        changed
    }
}
