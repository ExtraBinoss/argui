use std::{cell::Cell, rc::Rc};

use argui_core::{KeyInput, Point, PointerEvent, PointerId, PointerPhase, Rect, ScrollDelta};

use crate::{ClickEvent, Element, NodeId, SelectionCapabilities, SemanticAction, SemanticValue};

mod listener;
pub use listener::{
    ColorHandlerValue, ColorValueFormat, ContinuousValuePhase, EventFilter, EventHandler,
    EventHandlerId, EventListener, EventListenerOptions, EventOwnerId, EventPhase,
    FromHandlerValue, HandlerValue, RangeHandlerValue, SplitHandlerValue, ValueHandler,
};

impl Element {
    /// Registers an event listener on this element.
    ///
    /// * `listener` — event type, handler identity, and listener options.
    #[must_use]
    pub fn on(mut self, listener: EventListener) -> Self {
        self.event_listeners.push(listener);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    Action,
    PointerEnter,
    PointerLeave,
    PointerMove,
    PointerDown,
    PointerOutside,
    Dismiss,
    PointerUp,
    PointerCancel,
    Click,
    ContextMenu,
    GotPointerCapture,
    LostPointerCapture,
    Key,
    Wheel,
    Scroll,
    VirtualMeasure,
    VirtualWindow,
    Focus,
    Blur,
    Input,
    TextEdit,
    Submit,
    Gesture,
    SemanticAction,
    SelectionChange,
}

impl EventType {
    /// Event kinds that can be emitted by the UI event system.
    pub const ALL: [Self; 26] = [
        Self::Action,
        Self::PointerEnter,
        Self::PointerLeave,
        Self::PointerMove,
        Self::PointerDown,
        Self::PointerOutside,
        Self::Dismiss,
        Self::PointerUp,
        Self::PointerCancel,
        Self::Click,
        Self::ContextMenu,
        Self::GotPointerCapture,
        Self::LostPointerCapture,
        Self::Key,
        Self::Wheel,
        Self::Scroll,
        Self::VirtualMeasure,
        Self::VirtualWindow,
        Self::Focus,
        Self::Blur,
        Self::Input,
        Self::TextEdit,
        Self::Submit,
        Self::Gesture,
        Self::SemanticAction,
        Self::SelectionChange,
    ];

    /// Returns whether dispatch for this event type depends on pointer hit testing.
    #[must_use]
    pub const fn requires_hit_test(self) -> bool {
        matches!(
            self,
            Self::PointerEnter
                | Self::PointerLeave
                | Self::PointerMove
                | Self::PointerDown
                | Self::PointerUp
                | Self::Click
                | Self::ContextMenu
                | Self::Wheel
                | Self::Scroll
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventKind {
    Action(crate::ActionInvocation),
    Pointer(PointerEvent),
    PointerOutside(PointerEvent),
    /// The native host requested that this overlay close (focus loss or OS dismissal).
    DismissRequested,
    Click(ClickEvent),
    ContextMenu {
        position: Point,
        capabilities: SelectionCapabilities,
    },
    GotPointerCapture(PointerId),
    LostPointerCapture(PointerId),
    KeyInput(KeyInput),
    Wheel {
        delta: ScrollDelta,
        position: Point,
    },
    Scrolled {
        delta: Point,
        offset: Point,
    },
    /// Mounted virtual items changed size or the effective viewport changed.
    VirtualMeasured {
        items: Vec<crate::VirtualMeasurement>,
        corrected_offset: f32,
        viewport_extent: f32,
    },
    /// The native viewport selected a new bounded item range.
    VirtualWindowChanged {
        start: usize,
        end: usize,
        offset: f32,
        viewport_extent: f32,
    },
    Focused,
    Blurred,
    TextChanged(String),
    TextEdited(crate::TextEdit),
    Submitted(String),
    Gesture(crate::GestureEvent),
    SemanticAction {
        action: SemanticAction,
        value: Option<SemanticValue>,
    },
    DocumentSelectionChanged {
        text: Option<String>,
        bounds: Option<Rect>,
        touch: bool,
        dragging: bool,
    },
}

impl UiEventKind {
    /// Returns the event type corresponding to this event payload.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        match self {
            Self::Action(_) => EventType::Action,
            Self::Pointer(event) => match event.phase {
                PointerPhase::Entered => EventType::PointerEnter,
                PointerPhase::Moved => EventType::PointerMove,
                PointerPhase::Pressed => EventType::PointerDown,
                PointerPhase::Released => EventType::PointerUp,
                PointerPhase::Left => EventType::PointerLeave,
                PointerPhase::Cancelled => EventType::PointerCancel,
            },
            Self::PointerOutside(_) => EventType::PointerOutside,
            Self::DismissRequested => EventType::Dismiss,
            Self::Click(_) => EventType::Click,
            Self::ContextMenu { .. } => EventType::ContextMenu,
            Self::GotPointerCapture(_) => EventType::GotPointerCapture,
            Self::LostPointerCapture(_) => EventType::LostPointerCapture,
            Self::KeyInput(_) => EventType::Key,
            Self::Wheel { .. } => EventType::Wheel,
            Self::Scrolled { .. } => EventType::Scroll,
            Self::VirtualMeasured { .. } => EventType::VirtualMeasure,
            Self::VirtualWindowChanged { .. } => EventType::VirtualWindow,
            Self::Focused => EventType::Focus,
            Self::Blurred => EventType::Blur,
            Self::TextChanged(_) => EventType::Input,
            Self::TextEdited(_) => EventType::TextEdit,
            Self::Submitted(_) => EventType::Submit,
            Self::Gesture(_) => EventType::Gesture,
            Self::SemanticAction { .. } => EventType::SemanticAction,
            Self::DocumentSelectionChanged { .. } => EventType::SelectionChange,
        }
    }

    /// Returns whether this event propagates through ancestors.
    #[must_use]
    pub const fn bubbles(&self) -> bool {
        !matches!(
            self,
            Self::Action(_)
                | Self::Pointer(PointerEvent {
                    phase: PointerPhase::Entered | PointerPhase::Left,
                    ..
                })
                | Self::GotPointerCapture(_)
                | Self::LostPointerCapture(_)
                | Self::Focused
                | Self::Blurred
        )
    }

    /// Returns whether a handler may prevent the default action for this event.
    #[must_use]
    pub const fn cancelable(&self) -> bool {
        matches!(
            self,
            Self::Pointer(PointerEvent {
                phase: PointerPhase::Moved
                    | PointerPhase::Pressed
                    | PointerPhase::Released
                    | PointerPhase::Cancelled,
                ..
            }) | Self::PointerOutside(_)
                | Self::Click(_)
                | Self::ContextMenu { .. }
                | Self::KeyInput(_)
                | Self::Wheel { .. }
                | Self::Scrolled { .. }
                | Self::Submitted(_)
                | Self::Gesture(_)
                | Self::SemanticAction { .. }
        )
    }
}

#[derive(Debug, Default)]
struct EventControl {
    default_prevented: Cell<bool>,
    propagation_target: Cell<Option<NodeId>>,
    immediate_target: Cell<Option<NodeId>>,
}

#[derive(Clone, Debug)]
pub struct UiEvent {
    pub target: NodeId,
    pub kind: UiEventKind,
    target_key: Option<String>,
    current_target: NodeId,
    current_key: Option<String>,
    current_handler: Option<EventHandlerId>,
    handler_value: Option<HandlerValue>,
    phase: EventPhase,
    passive: bool,
    once: Option<Rc<Cell<bool>>>,
    control: Rc<EventControl>,
    default_action: bool,
    default_sensitive: bool,
    focused_node: Option<NodeId>,
    history: Option<(bool, bool)>,
}

/// Internal listener-delivery metadata copied onto one dispatched event.
pub(crate) struct EventDelivery {
    pub(crate) current_target: NodeId,
    pub(crate) current_key: Option<String>,
    pub(crate) current_handler: Option<EventHandlerId>,
    pub(crate) phase: EventPhase,
    pub(crate) passive: bool,
    pub(crate) once: Option<Rc<Cell<bool>>>,
    pub(crate) handler_value: Option<HandlerValue>,
    pub(crate) default_sensitive: bool,
}

impl PartialEq for UiEvent {
    fn eq(&self, other: &Self) -> bool {
        (
            self.target,
            &self.kind,
            &self.target_key,
            self.current_target,
            &self.current_key,
            &self.handler_value,
            self.phase,
        ) == (
            other.target,
            &other.kind,
            &other.target_key,
            other.current_target,
            &other.current_key,
            &other.handler_value,
            other.phase,
        )
    }
}

impl UiEvent {
    /// Creates an event targeted at a node, optionally recording its key.
    ///
    /// # Arguments
    ///
    /// * `target` — node where dispatch begins.
    /// * `key` — stable element key, if the target has one.
    /// * `kind` — event payload and semantic event type.
    #[must_use]
    pub fn new(target: NodeId, key: Option<String>, kind: UiEventKind) -> Self {
        Self {
            target,
            kind,
            target_key: key.clone(),
            current_target: target,
            current_key: key,
            current_handler: None,
            handler_value: None,
            phase: EventPhase::Target,
            passive: false,
            once: None,
            control: Rc::new(EventControl::default()),
            default_action: false,
            default_sensitive: false,
            focused_node: None,
            history: None,
        }
    }

    /// Clones this event for one listener with the supplied propagation metadata.
    pub(crate) fn delivery(&self, delivery: EventDelivery) -> Self {
        Self {
            target: self.target,
            kind: self.kind.clone(),
            target_key: self.target_key.clone(),
            current_target: delivery.current_target,
            current_key: delivery.current_key,
            current_handler: delivery.current_handler,
            handler_value: delivery.handler_value,
            phase: delivery.phase,
            passive: delivery.passive,
            once: delivery.once,
            control: Rc::clone(&self.control),
            default_action: self.default_action,
            default_sensitive: delivery.default_sensitive,
            focused_node: self.focused_node,
            history: self.history,
        }
    }

    /// Returns the node currently receiving this event during dispatch.
    #[must_use]
    pub const fn current_target(&self) -> NodeId {
        self.current_target
    }

    /// Returns the handler currently receiving this event, when available.
    #[doc(hidden)]
    #[must_use]
    pub const fn current_handler(&self) -> Option<EventHandlerId> {
        self.current_handler
    }

    /// Returns the typed value attached by the current widget listener.
    #[doc(hidden)]
    #[must_use]
    pub const fn handler_value(&self) -> Option<&HandlerValue> {
        self.handler_value.as_ref()
    }

    /// Returns the key of the original target, if it had one.
    #[must_use]
    pub fn target_key(&self) -> Option<&str> {
        self.target_key.as_deref()
    }

    /// Returns the key of the current dispatch target, if it had one.
    #[must_use]
    pub fn current_key(&self) -> Option<&str> {
        self.current_key.as_deref()
    }

    /// Returns the current phase of event propagation.
    #[must_use]
    pub const fn phase(&self) -> EventPhase {
        self.phase
    }

    /// Returns the event type represented by this event.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.kind.event_type()
    }

    /// Returns whether this event bubbles through ancestors.
    #[must_use]
    pub const fn bubbles(&self) -> bool {
        self.kind.bubbles()
    }

    /// Returns whether default handling can be prevented for this event.
    #[must_use]
    pub const fn cancelable(&self) -> bool {
        self.kind.cancelable()
    }

    /// Stops propagation after the current target finishes handling the event.
    pub fn stop_propagation(&self) {
        if self.control.propagation_target.get().is_none() {
            self.control
                .propagation_target
                .set(Some(self.current_target));
        }
    }

    /// Stops propagation and prevents remaining handlers on the current target.
    pub fn stop_immediate_propagation(&self) {
        self.stop_propagation();
        self.control.immediate_target.set(Some(self.current_target));
    }

    /// Requests cancellation of the default action.
    ///
    /// Returns `true` if the event is cancelable and the listener is not passive.
    #[must_use]
    pub fn prevent_default(&self) -> bool {
        if self.cancelable() && !self.passive {
            self.control.default_prevented.set(true);
            true
        } else {
            false
        }
    }

    /// Returns whether a listener prevented the default action.
    #[must_use]
    pub fn default_prevented(&self) -> bool {
        self.control.default_prevented.get()
    }

    /// Returns whether propagation has been stopped.
    #[must_use]
    pub fn propagation_stopped(&self) -> bool {
        self.control.propagation_target.get().is_some()
    }

    #[doc(hidden)]
    /// Returns whether this event should continue dispatching to the next listener.
    pub fn should_dispatch(&self) -> bool {
        if self.default_action {
            return !self.control.default_prevented.replace(true);
        }
        if self.default_sensitive && self.control.default_prevented.get() {
            return false;
        }
        if self.control.immediate_target.get().is_some() {
            return false;
        }
        let propagates = self
            .control
            .propagation_target
            .get()
            .is_none_or(|node| node == self.current_target);
        propagates
            && self
                .once
                .as_ref()
                .is_none_or(|consumed| !consumed.replace(true))
    }

    pub(crate) fn into_default_action(mut self) -> Self {
        self.default_action = true;
        self
    }
    pub(crate) fn set_focused_node(&mut self, node: Option<NodeId>) {
        self.focused_node = node;
    }
    pub(crate) fn set_history(&mut self, history: Option<(bool, bool)>) {
        self.history = history;
    }
    /// Returns undo and redo availability for the target editor when the event was created.
    #[must_use]
    pub const fn edit_history(&self) -> Option<(bool, bool)> {
        self.history
    }
    /// Returns focus at event creation, before a pointer default can move it to a trigger.
    #[must_use]
    pub const fn focused_node(&self) -> Option<NodeId> {
        self.focused_node
    }
}
