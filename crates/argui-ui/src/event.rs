use std::{cell::Cell, rc::Rc};

use argui_core::{KeyInput, Point, PointerEvent, PointerId, PointerPhase, Rect, ScrollDelta};

use crate::{ClickEvent, Element, NodeId, SelectionCapabilities, SemanticAction, SemanticValue};

impl Element {
    #[must_use]
    pub fn on(mut self, listener: EventListener) -> Self {
        self.event_listeners.push(listener);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    PointerEnter,
    PointerLeave,
    PointerMove,
    PointerDown,
    PointerOutside,
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
    pub const ALL: [Self; 21] = [
        Self::PointerEnter,
        Self::PointerLeave,
        Self::PointerMove,
        Self::PointerDown,
        Self::PointerOutside,
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
    #[must_use]
    pub const fn capture(mut self, capture: bool) -> Self {
        self.capture = capture;
        self
    }

    #[must_use]
    pub const fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

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
    #[doc(hidden)]
    #[must_use]
    pub const fn new(owner: EventOwnerId, slot: u32) -> Self {
        Self { owner, slot }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn owner(self) -> EventOwnerId {
        self.owner
    }

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

    #[must_use]
    pub const fn capture(mut self, capture: bool) -> Self {
        self.options.capture = capture;
        self
    }

    #[must_use]
    pub const fn passive(mut self, passive: bool) -> Self {
        self.options.passive = passive;
        self
    }

    #[must_use]
    pub const fn once(mut self, once: bool) -> Self {
        self.options.once = once;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventKind {
    Pointer(PointerEvent),
    PointerOutside(PointerEvent),
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
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        match self {
            Self::Pointer(event) => match event.phase {
                PointerPhase::Entered => EventType::PointerEnter,
                PointerPhase::Moved => EventType::PointerMove,
                PointerPhase::Pressed => EventType::PointerDown,
                PointerPhase::Released => EventType::PointerUp,
                PointerPhase::Left => EventType::PointerLeave,
                PointerPhase::Cancelled => EventType::PointerCancel,
            },
            Self::PointerOutside(_) => EventType::PointerOutside,
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

    #[must_use]
    pub const fn bubbles(&self) -> bool {
        !matches!(
            self,
            Self::Pointer(PointerEvent {
                phase: PointerPhase::Entered | PointerPhase::Left,
                ..
            }) | Self::GotPointerCapture(_)
                | Self::LostPointerCapture(_)
                | Self::Focused
                | Self::Blurred
        )
    }

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
        }
    }

    #[must_use]
    pub const fn current_target(&self) -> NodeId {
        self.current_target
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn current_handler(&self) -> Option<EventHandlerId> {
        self.current_handler
    }

    #[must_use]
    pub fn target_key(&self) -> Option<&str> {
        self.target_key.as_deref()
    }

    #[must_use]
    pub fn current_key(&self) -> Option<&str> {
        self.current_key.as_deref()
    }

    #[must_use]
    pub const fn phase(&self) -> EventPhase {
        self.phase
    }

    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.kind.event_type()
    }

    #[must_use]
    pub const fn bubbles(&self) -> bool {
        self.kind.bubbles()
    }

    #[must_use]
    pub const fn cancelable(&self) -> bool {
        self.kind.cancelable()
    }

    pub fn stop_propagation(&self) {
        if self.control.propagation_target.get().is_none() {
            self.control
                .propagation_target
                .set(Some(self.current_target));
        }
    }

    pub fn stop_immediate_propagation(&self) {
        self.stop_propagation();
        self.control.immediate_target.set(Some(self.current_target));
    }

    #[must_use]
    pub fn prevent_default(&self) -> bool {
        if self.cancelable() && !self.passive {
            self.control.default_prevented.set(true);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn default_prevented(&self) -> bool {
        self.control.default_prevented.get()
    }

    #[must_use]
    pub fn propagation_stopped(&self) -> bool {
        self.control.propagation_target.get().is_some()
    }

    #[doc(hidden)]
    pub fn should_dispatch(&self) -> bool {
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
}
