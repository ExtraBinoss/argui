use std::{cell::Cell, rc::Rc};

use argui_core::{KeyInput, Point, PointerEvent, PointerId, PointerPhase, Rect, ScrollDelta};

use crate::{ClickEvent, Element, NodeId, SelectionCapabilities, SemanticAction, SemanticValue};

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
    Focus,
    Blur,
    Input,
    Submit,
    Gesture,
    SemanticAction,
    SelectionChange,
}

impl EventType {
    /// Event kinds that can be emitted by the UI event system.
    pub const ALL: [Self; 23] = [
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
        Self::Focus,
        Self::Blur,
        Self::Input,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventListener {
    pub event: EventType,
    pub options: EventListenerOptions,
    pub(crate) handler: EventHandlerId,
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
            },
            handler,
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
    Focused,
    Blurred,
    TextChanged(String),
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
            Self::Focused => EventType::Focus,
            Self::Blurred => EventType::Blur,
            Self::TextChanged(_) => EventType::Input,
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
    phase: EventPhase,
    passive: bool,
    once: Option<Rc<Cell<bool>>>,
    control: Rc<EventControl>,
    default_action: bool,
    focused_node: Option<NodeId>,
    history: Option<(bool, bool)>,
}

impl PartialEq for UiEvent {
    fn eq(&self, other: &Self) -> bool {
        (
            self.target,
            &self.kind,
            &self.target_key,
            self.current_target,
            &self.current_key,
            self.phase,
        ) == (
            other.target,
            &other.kind,
            &other.target_key,
            other.current_target,
            &other.current_key,
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
            phase: EventPhase::Target,
            passive: false,
            once: None,
            control: Rc::new(EventControl::default()),
            default_action: false,
            focused_node: None,
            history: None,
        }
    }

    pub(crate) fn delivery(
        &self,
        current_target: NodeId,
        current_key: Option<String>,
        current_handler: Option<EventHandlerId>,
        phase: EventPhase,
        passive: bool,
        once: Option<Rc<Cell<bool>>>,
    ) -> Self {
        Self {
            target: self.target,
            kind: self.kind.clone(),
            target_key: self.target_key.clone(),
            current_target,
            current_key,
            current_handler,
            phase,
            passive,
            once,
            control: Rc::clone(&self.control),
            default_action: self.default_action,
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
