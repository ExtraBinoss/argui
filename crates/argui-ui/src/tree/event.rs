use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{EventHandlerId, EventListener, EventPhase, EventType, NodeId, UiEvent, UiEventKind};

use super::UiTree;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ConsumedListener {
    node: NodeId,
    handler: EventHandlerId,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EventRegistry {
    once: HashMap<ConsumedListener, Rc<Cell<bool>>>,
}

impl EventRegistry {
    pub(super) fn sync(&mut self, index: &super::index::TreeIndex) {
        self.once.retain(|consumed, _| {
            index.element(consumed.node).is_some_and(|element| {
                element
                    .event_listeners
                    .iter()
                    .any(|listener| listener.handler == consumed.handler && listener.options.once)
            })
        });
    }

    fn once_state(&mut self, node: NodeId, listener: &EventListener) -> Option<Rc<Cell<bool>>> {
        if !listener.options.once {
            return None;
        }
        let key = ConsumedListener {
            node,
            handler: listener.handler,
        };
        Some(
            self.once
                .entry(key)
                .or_insert_with(|| Rc::new(Cell::new(false)))
                .clone(),
        )
    }
}

impl UiTree {
    /// Returns whether `event_type` has a potential listener from `target` to the root.
    pub(crate) fn has_event_listener(&self, target: NodeId, event_type: EventType) -> bool {
        let Some(mut index) = self.index.position(target) else {
            return false;
        };
        loop {
            let node = self.node_ids[index];
            if self.element_at(index).is_some_and(|element| {
                element.event_listeners.iter().any(|listener| {
                    listener.event == event_type
                        && (!listener.options.target_only || node == target)
                        && listener
                            .target_key
                            .as_deref()
                            .is_none_or(|key| self.key_for(target) == Some(key))
                })
            }) {
                return true;
            }
            let Some(parent) = self.index.parent(index) else {
                return false;
            };
            index = parent;
        }
    }

    /// Creates event deliveries for listeners along the target's propagation path.
    ///
    /// * `target` — retained node receiving the event.
    /// * `kind` — event kind and payload to dispatch.
    ///
    /// Returns deliveries in propagation order, or an empty vector for an unknown
    /// target or a click on an unavailable input.
    pub fn event_deliveries(&mut self, target: NodeId, kind: UiEventKind) -> Vec<UiEvent> {
        let Some(target_index) = self.index.position(target) else {
            return Vec::new();
        };
        if matches!(kind, UiEventKind::Click(_)) && !self.input_available(target) {
            return Vec::new();
        }
        let event_type = kind.event_type();
        let target_key = self.key_for(target).map(ToOwned::to_owned);
        if matches!(kind, UiEventKind::Blurred | UiEventKind::Focused)
            && let Some(state) = self.text_inputs.get_mut(target)
        {
            state.break_edit_group();
        }
        let mut base = UiEvent::new(target, target_key, kind);
        base.set_focused_node(self.focused_node());
        base.set_history(
            self.text_inputs
                .get(target)
                .map(|state| (state.can_undo(), state.can_redo())),
        );
        let mut ancestry = Vec::new();
        let mut cursor = self.index.parent(target_index);
        while let Some(index) = cursor {
            ancestry.push(index);
            cursor = self.index.parent(index);
        }
        ancestry.reverse();

        let mut deliveries = Vec::new();
        for index in ancestry.iter().copied() {
            self.push_listeners(
                &base,
                index,
                event_type,
                true,
                EventPhase::Capture,
                &mut deliveries,
            );
        }

        self.push_listeners(
            &base,
            target_index,
            event_type,
            true,
            EventPhase::Target,
            &mut deliveries,
        );
        self.push_listeners(
            &base,
            target_index,
            event_type,
            false,
            EventPhase::Target,
            &mut deliveries,
        );
        if base.bubbles() {
            for index in ancestry.into_iter().rev() {
                self.push_listeners(
                    &base,
                    index,
                    event_type,
                    false,
                    EventPhase::Bubble,
                    &mut deliveries,
                );
            }
        }
        if let Some(action) = self.default_action(&base) {
            deliveries.push(action);
        }
        deliveries
    }

    fn push_listeners(
        &mut self,
        base: &UiEvent,
        index: usize,
        event_type: EventType,
        capture: bool,
        phase: EventPhase,
        output: &mut Vec<UiEvent>,
    ) {
        let node = self.node_ids[index];
        let listeners = self
            .element_at(index)
            .map(|element| element.event_listeners.clone())
            .unwrap_or_default();
        for listener in listeners.into_iter().filter(|listener| {
            listener.event == event_type
                && listener.options.capture == capture
                && (!listener.options.target_only || base.target == node)
                && listener
                    .target_key
                    .as_deref()
                    .is_none_or(|key| base.target_key() == Some(key))
                && listener.options.filter.accepts(&base.kind)
        }) {
            let value = match &listener.value {
                Some(source) => {
                    let Some(value) = source.resolve(
                        &base.kind,
                        self.interaction_bounds.get(&base.target).copied(),
                    ) else {
                        continue;
                    };
                    Some(value)
                }
                None => None,
            };
            let once = self.events.once_state(node, &listener);
            if once.as_ref().is_some_and(|consumed| consumed.get()) {
                continue;
            }
            let element = self.element_at(index);
            output.push(base.delivery(crate::event::EventDelivery {
                current_target: self.node_ids[index],
                current_key: element.and_then(|value| value.key.clone()),
                current_handler: Some(listener.handler),
                phase,
                passive: listener.options.passive,
                once,
                handler_value: value,
            }));
        }
    }
}
