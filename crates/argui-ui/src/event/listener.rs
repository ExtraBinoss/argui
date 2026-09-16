use std::fmt;
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use super::{EventType, UiEventKind};

mod color;
mod value;
pub use color::{ColorHandlerValue, ColorValueFormat};
use value::HandlerValueSource;
pub use value::{
    ContinuousValuePhase, FromHandlerValue, HandlerValue, RangeHandlerValue, SplitHandlerValue,
};

/// Additional deterministic predicate applied before listener delivery.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EventFilter {
    #[default]
    Any,
    EscapePressed,
    EnterPressed,
    SpacePressed,
    TabPressed,
    VerticalArrowPressed,
    ArrowUpPressed,
    ArrowDownPressed,
    ArrowLeftPressed,
    ArrowRightPressed,
    HomePressed,
    EndPressed,
    PageUpPressed,
    PageDownPressed,
    PanEndedLeft,
    PanEndedRight,
    DoubleClick,
}

impl EventFilter {
    /// Returns whether this filter accepts the supplied event payload.
    ///
    /// * `kind` — event payload considered before listener delivery.
    pub(crate) fn accepts(self, kind: &UiEventKind) -> bool {
        let pressed = |key| {
            matches!(
                kind,
                UiEventKind::KeyInput(argui_core::KeyInput {
                    key: candidate,
                    state: argui_core::KeyState::Pressed,
                    ..
                }) if *candidate == key
            )
        };
        match self {
            Self::Any => true,
            Self::EscapePressed => pressed(argui_core::Key::Escape),
            Self::EnterPressed => pressed(argui_core::Key::Enter),
            Self::SpacePressed => pressed(argui_core::Key::Character(" ".into())),
            Self::TabPressed => pressed(argui_core::Key::Tab),
            Self::VerticalArrowPressed => {
                pressed(argui_core::Key::ArrowDown) || pressed(argui_core::Key::ArrowUp)
            }
            Self::ArrowUpPressed => pressed(argui_core::Key::ArrowUp),
            Self::ArrowDownPressed => pressed(argui_core::Key::ArrowDown),
            Self::ArrowLeftPressed => pressed(argui_core::Key::ArrowLeft),
            Self::ArrowRightPressed => pressed(argui_core::Key::ArrowRight),
            Self::HomePressed => pressed(argui_core::Key::Home),
            Self::EndPressed => pressed(argui_core::Key::End),
            Self::PageUpPressed => pressed(argui_core::Key::PageUp),
            Self::PageDownPressed => pressed(argui_core::Key::PageDown),
            Self::PanEndedLeft => matches!(
                kind,
                UiEventKind::Gesture(crate::GestureEvent {
                    phase: crate::GesturePhase::Ended,
                    kind: crate::GestureKind::Pan { total, .. },
                    ..
                }) if total.x <= -40.0
            ),
            Self::PanEndedRight => matches!(
                kind,
                UiEventKind::Gesture(crate::GestureEvent {
                    phase: crate::GesturePhase::Ended,
                    kind: crate::GestureKind::Pan { total, .. },
                    ..
                }) if total.x >= 40.0
            ),
            Self::DoubleClick => matches!(kind, UiEventKind::Click(click) if click.count >= 2),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EventPhase {
    Capture,
    #[default]
    Target,
    Bubble,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EventListenerOptions {
    pub capture: bool,
    pub passive: bool,
    pub once: bool,
    pub target_only: bool,
    pub filter: EventFilter,
}

impl EventListenerOptions {
    /// Enables capture phase delivery for this listener.
    #[must_use]
    pub const fn capture(mut self, capture: bool) -> Self {
        self.capture = capture;
        self
    }

    /// Marks this listener passive, disallowing default prevention.
    #[must_use]
    pub const fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

    /// Marks this listener for delivery only once.
    ///
    /// * `once` — whether to remove the listener after its first delivery.
    #[must_use]
    pub const fn once(mut self, once: bool) -> Self {
        self.once = once;
        self
    }

    /// Restricts delivery to events whose original target is the listener's element.
    ///
    /// `target_only` prevents activation bubbling from an interactive descendant from
    /// invoking the listener while preserving ordinary capture and bubble semantics
    /// for other listeners.
    #[must_use]
    pub const fn target_only(mut self, target_only: bool) -> Self {
        self.target_only = target_only;
        self
    }

    /// Applies an additional deterministic `filter` before delivery.
    #[must_use]
    pub const fn filter(mut self, filter: EventFilter) -> Self {
        self.filter = filter;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventOwnerId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventHandlerId {
    owner: EventOwnerId,
    slot: u32,
}

impl EventHandlerId {
    /// Creates a handler identity from an owner and its local slot.
    ///
    /// * `owner` — identity of the registering owner.
    /// * `slot` — handler slot within that owner.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(owner: EventOwnerId, slot: u32) -> Self {
        Self { owner, slot }
    }

    /// Returns the owner associated with this handler.
    #[doc(hidden)]
    #[must_use]
    pub const fn owner(self) -> EventOwnerId {
        self.owner
    }

    /// Returns this handler's owner-local slot.
    #[doc(hidden)]
    #[must_use]
    pub const fn slot(self) -> u32 {
        self.slot
    }
}

/// Opaque identity of a callback registered by the application runtime.
///
/// A handler does not select an event type. Widgets bind it to their semantic
/// event while building an [`crate::Element`]. The identity is presentation-owned and
/// cannot be constructed or inspected by normal application code.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct EventHandler {
    identity: EventHandlerId,
}

impl fmt::Debug for EventHandler {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EventHandler")
    }
}

impl EventHandler {
    /// Wraps the runtime's presentation-owned handler identity.
    ///
    /// `identity` is allocated by the runtime for the current render slot.
    #[doc(hidden)]
    #[must_use]
    pub const fn from_identity(identity: EventHandlerId) -> Self {
        Self { identity }
    }

    /// Binds this handler to `event` using ordinary propagation semantics.
    #[doc(hidden)]
    #[must_use]
    pub const fn listener(self, event: EventType) -> EventListener {
        EventListener::new(event, self.identity)
    }

    /// Binds this handler to `event` only when the element is the original target.
    #[doc(hidden)]
    #[must_use]
    pub const fn direct_listener(self, event: EventType) -> EventListener {
        self.listener(event).target_only(true)
    }
}

/// Opaque runtime handler whose callback receives a domain value of type `V`.
///
/// The value adapter remains in the runtime registry; this marker never stores
/// callbacks or application data in an element description.
pub struct ValueHandler<V> {
    handler: EventHandler,
    marker: PhantomData<fn(V)>,
}

impl<V> Copy for ValueHandler<V> {}

impl<V> Clone for ValueHandler<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> PartialEq for ValueHandler<V> {
    fn eq(&self, other: &Self) -> bool {
        self.handler == other.handler
    }
}

impl<V> Eq for ValueHandler<V> {}

impl<V> Hash for ValueHandler<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.handler.hash(state);
    }
}

impl<V> ValueHandler<V> {
    /// Associates an opaque event handler with its runtime-adapted value type.
    ///
    /// `handler` is the underlying presentation-owned handler identity.
    #[doc(hidden)]
    #[must_use]
    pub const fn from_handler(handler: EventHandler) -> Self {
        Self {
            handler,
            marker: PhantomData,
        }
    }

    /// Binds this value handler to `event` for the element's own delivery.
    #[doc(hidden)]
    #[must_use]
    pub const fn direct_listener(self, event: EventType) -> EventListener {
        self.handler.direct_listener(event)
    }
}

impl<V: Into<HandlerValue>> ValueHandler<V> {
    /// Binds this handler to `event` and supplies the typed `value` on delivery.
    #[doc(hidden)]
    #[must_use]
    pub fn direct_listener_value(self, event: EventType, value: V) -> EventListener {
        self.direct_listener(event).handler_value(value)
    }

    /// Binds this handler to a propagating `event` and supplies `value`.
    #[doc(hidden)]
    #[must_use]
    pub fn listener_value(self, event: EventType, value: V) -> EventListener {
        self.handler.listener(event).handler_value(value)
    }
}

impl<V> fmt::Debug for ValueHandler<V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValueHandler")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventListener {
    pub event: EventType,
    pub options: EventListenerOptions,
    pub(crate) handler: EventHandlerId,
    pub(crate) value: Option<HandlerValueSource>,
    pub(crate) target_key: Option<String>,
}

impl EventListener {
    /// Creates a listener for an event and handler.
    ///
    /// * `event` — event type to receive.
    /// * `handler` — registered event handler identity.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(event: EventType, handler: EventHandlerId) -> Self {
        Self {
            event,
            options: EventListenerOptions {
                capture: false,
                passive: false,
                once: false,
                target_only: false,
                filter: EventFilter::Any,
            },
            handler,
            value: None,
            target_key: None,
        }
    }

    /// Enables or disables capture phase delivery.
    #[must_use]
    pub const fn capture(mut self, capture: bool) -> Self {
        self.options.capture = capture;
        self
    }

    /// Enables or disables passive listener behavior.
    #[must_use]
    pub const fn passive(mut self, passive: bool) -> Self {
        self.options.passive = passive;
        self
    }

    /// Enables or disables one-time delivery.
    ///
    /// * `once` — whether to remove the listener after its first delivery.
    #[must_use]
    pub const fn once(mut self, once: bool) -> Self {
        self.options.once = once;
        self
    }

    /// Restricts delivery to an event originally targeted at this element.
    ///
    /// `target_only` is useful for direct control callbacks that must ignore
    /// activation bubbling from interactive descendants.
    #[must_use]
    pub const fn target_only(mut self, target_only: bool) -> Self {
        self.options.target_only = target_only;
        self
    }

    /// Applies an additional deterministic `filter` before delivery.
    #[must_use]
    pub const fn filter(mut self, filter: EventFilter) -> Self {
        self.options.filter = filter;
        self
    }

    /// Associates a typed widget value with this listener delivery.
    #[doc(hidden)]
    #[must_use]
    pub fn handler_value(mut self, value: impl Into<HandlerValue>) -> Self {
        self.value = Some(HandlerValueSource::Static(value.into()));
        self
    }

    /// Derives a continuous numeric value from the delivered event and target bounds.
    #[doc(hidden)]
    #[must_use]
    pub fn range_handler_value(mut self, value: RangeHandlerValue) -> Self {
        self.value = Some(HandlerValueSource::Range(value));
        self
    }

    /// Derives a color-picker value from the delivered event and target bounds.
    #[doc(hidden)]
    #[must_use]
    pub fn color_handler_value(mut self, value: ColorHandlerValue) -> Self {
        self.value = Some(HandlerValueSource::Color(value));
        self
    }

    /// Derives a split-pane size from the delivered event.
    #[doc(hidden)]
    #[must_use]
    pub fn split_handler_value(mut self, value: SplitHandlerValue) -> Self {
        self.value = Some(HandlerValueSource::Split(value));
        self
    }

    /// Restricts delivery to events whose original target has `key`.
    #[doc(hidden)]
    #[must_use]
    pub fn target_key(mut self, key: impl Into<String>) -> Self {
        self.target_key = Some(key.into());
        self
    }
}
