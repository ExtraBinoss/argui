use std::{cell::Cell, rc::Rc};

use argui_core::{KeyInput, Point, PointerId, Rect, ScrollDelta};

use crate::{Element, NodeId, SelectionCapabilities, SemanticAction, SemanticValue};

impl Element {
    #[must_use]
    pub fn listen(mut self, event: EventType, options: EventListenerOptions) -> Self {
        let listener = EventListener { event, options };
        if !self.event_listeners.contains(&listener) {
            self.event_listeners.push(listener);
        }
        self
    }

    #[doc(hidden)]
    pub fn assign_event_owner(&mut self, owner: EventOwnerId) {
        if self.event_owner.is_some() {
            return;
        }
        self.event_owner = Some(owner);
        for child in &mut self.children {
            child.assign_event_owner(owner);
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventListener {
    pub event: EventType,
    pub options: EventListenerOptions,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventOwnerId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventKind {
    PointerEntered,
    PointerLeft,
    PointerMoved(Point),
    Pressed,
    PointerOutside,
    Released,
    Clicked,
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
            Self::PointerEntered => EventType::PointerEnter,
            Self::PointerLeft => EventType::PointerLeave,
            Self::PointerMoved(_) => EventType::PointerMove,
            Self::Pressed => EventType::PointerDown,
            Self::PointerOutside => EventType::PointerOutside,
            Self::Released => EventType::PointerUp,
            Self::Clicked => EventType::Click,
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
            Self::PointerEntered
                | Self::PointerLeft
                | Self::GotPointerCapture(_)
                | Self::LostPointerCapture(_)
                | Self::Focused
                | Self::Blurred
        )
    }

    #[must_use]
    pub const fn cancelable(&self) -> bool {
        matches!(
            self,
            Self::Pressed
                | Self::PointerOutside
                | Self::Released
                | Self::Clicked
                | Self::ContextMenu { .. }
                | Self::PointerMoved(_)
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
    pub key: Option<String>,
    pub kind: UiEventKind,
    target_key: Option<String>,
    current_target: NodeId,
    current_owner: Option<EventOwnerId>,
    phase: EventPhase,
    passive: bool,
    once: Option<Rc<Cell<bool>>>,
    control: Rc<EventControl>,
}

impl PartialEq for UiEvent {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
            && self.key == other.key
            && self.kind == other.kind
            && self.target_key == other.target_key
            && self.current_target == other.current_target
            && self.phase == other.phase
    }
}

impl UiEvent {
    #[must_use]
    pub fn new(target: NodeId, key: Option<String>, kind: UiEventKind) -> Self {
        Self {
            target,
            key: key.clone(),
            kind,
            target_key: key,
            current_target: target,
            current_owner: None,
            phase: EventPhase::Target,
            passive: false,
            once: None,
            control: Rc::new(EventControl::default()),
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn with_current_owner(mut self, owner: EventOwnerId) -> Self {
        self.current_owner = Some(owner);
        self
    }

    pub(crate) fn delivery(
        &self,
        current_target: NodeId,
        current_key: Option<String>,
        current_owner: Option<EventOwnerId>,
        phase: EventPhase,
        passive: bool,
        once: Option<Rc<Cell<bool>>>,
    ) -> Self {
        Self {
            target: self.target,
            key: current_key,
            kind: self.kind.clone(),
            target_key: self.target_key.clone(),
            current_target,
            current_owner,
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

    #[must_use]
    pub const fn current_owner(&self) -> Option<EventOwnerId> {
        self.current_owner
    }

    #[must_use]
    pub fn target_key(&self) -> Option<&str> {
        self.target_key.as_deref()
    }

    #[must_use]
    pub fn current_key(&self) -> Option<&str> {
        self.key.as_deref()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn event(target: u64) -> UiEvent {
        UiEvent::new(
            NodeId::new(target),
            Some("target".into()),
            UiEventKind::Clicked,
        )
    }

    #[test]
    fn element_listener_and_owner_registration_are_idempotent() {
        let options = EventListenerOptions::default();
        let mut element = Element::container([Element::text("child")])
            .listen(EventType::Click, options)
            .listen(EventType::Click, options);
        assert_eq!(element.event_listeners.len(), 1);
        element.assign_event_owner(EventOwnerId(3));
        element.assign_event_owner(EventOwnerId(4));
        assert_eq!(element.event_owner, Some(EventOwnerId(3)));
        assert_eq!(element.children[0].event_owner, Some(EventOwnerId(3)));
    }

    #[test]
    fn event_equality_compares_every_observable_delivery_field() {
        let base = event(1);
        assert_eq!(base, base.clone());
        assert_ne!(base, event(2));

        let mut changed = base.clone();
        changed.key = Some("current".into());
        assert_ne!(base, changed);
        let mut changed = base.clone();
        changed.kind = UiEventKind::Pressed;
        assert_ne!(base, changed);
        let mut changed = base.clone();
        changed.target_key = None;
        assert_ne!(base, changed);
        let changed = base.delivery(
            NodeId::new(2),
            Some("target".into()),
            None,
            EventPhase::Bubble,
            false,
            None,
        );
        assert_ne!(base, changed);
        let changed = base.delivery(
            NodeId::new(1),
            Some("target".into()),
            None,
            EventPhase::Bubble,
            false,
            None,
        );
        assert_ne!(base, changed);
    }

    #[test]
    fn controls_cover_cancelation_propagation_and_once_delivery() {
        let click = event(1);
        assert!(click.prevent_default());
        assert!(click.default_prevented());
        click.stop_propagation();
        click.stop_propagation();
        assert!(click.should_dispatch());

        let enter = UiEvent::new(NodeId::new(1), None, UiEventKind::PointerEntered);
        assert!(!enter.prevent_default());
        enter.stop_immediate_propagation();
        assert!(!enter.should_dispatch());

        let once = Rc::new(Cell::new(false));
        let delivery = event(1).delivery(
            NodeId::new(1),
            None,
            None,
            EventPhase::Target,
            false,
            Some(once),
        );
        assert!(delivery.should_dispatch());
        assert!(!delivery.should_dispatch());
    }
}
