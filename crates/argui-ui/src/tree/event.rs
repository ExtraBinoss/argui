use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::{Element, EventListener, EventPhase, EventType, NodeId, UiEvent, UiEventKind};

use super::UiTree;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ConsumedListener {
    node: NodeId,
    event: EventType,
    capture: bool,
}

#[derive(Clone, Debug, Default)]
pub(super) struct EventRegistry {
    parents: Vec<Option<usize>>,
    once: HashMap<ConsumedListener, Rc<Cell<bool>>>,
}

impl EventRegistry {
    pub(super) fn new(root: &Element) -> Self {
        let mut registry = Self::default();
        collect_parents(root, None, &mut registry.parents);
        registry
    }

    pub(super) fn sync(&mut self, root: &Element, ids: &[NodeId]) {
        self.parents.clear();
        collect_parents(root, None, &mut self.parents);
        self.once.retain(|consumed, _| {
            let Some(index) = ids.iter().position(|node| *node == consumed.node) else {
                return false;
            };
            element_at(root, index).is_some_and(|element| {
                element.event_listeners.iter().any(|listener| {
                    listener.event == consumed.event
                        && listener.options.capture == consumed.capture
                        && listener.options.once
                })
            })
        });
    }

    pub(super) fn parent(&self, index: usize) -> Option<usize> {
        self.parents.get(index).copied().flatten()
    }

    fn once_state(&mut self, node: NodeId, listener: EventListener) -> Option<Rc<Cell<bool>>> {
        if !listener.options.once {
            return None;
        }
        let key = ConsumedListener {
            node,
            event: listener.event,
            capture: listener.options.capture,
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
    pub fn event_deliveries(&mut self, target: NodeId, kind: UiEventKind) -> Vec<UiEvent> {
        let Some(target_index) = self.node_ids.iter().position(|node| *node == target) else {
            return Vec::new();
        };
        let event_type = kind.event_type();
        let target_key = self.key_for(target).map(ToOwned::to_owned);
        let base = UiEvent::new(target, target_key, kind);
        let mut ancestry = Vec::new();
        let mut cursor = self.events.parent(target_index);
        while let Some(index) = cursor {
            ancestry.push(index);
            cursor = self.events.parent(index);
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

        let target_declares_listener = self.element_at(target_index).is_some_and(|element| {
            element
                .event_listeners
                .iter()
                .any(|listener| listener.event == event_type)
        });
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
        if !target_declares_listener {
            deliveries.push(self.delivery(&base, target_index, EventPhase::Target, false, None));
        }

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
        for listener in listeners
            .into_iter()
            .filter(|listener| listener.event == event_type && listener.options.capture == capture)
        {
            let once = self.events.once_state(node, listener);
            if once.as_ref().is_some_and(|consumed| consumed.get()) {
                continue;
            }
            output.push(self.delivery(base, index, phase, listener.options.passive, once));
        }
    }

    fn delivery(
        &self,
        base: &UiEvent,
        index: usize,
        phase: EventPhase,
        passive: bool,
        once: Option<Rc<Cell<bool>>>,
    ) -> UiEvent {
        let element = self.element_at(index);
        base.delivery(
            self.node_ids[index],
            element.and_then(|value| value.key.clone()),
            element.and_then(|value| value.event_owner),
            phase,
            passive,
            once,
        )
    }
}

fn collect_parents(element: &Element, parent: Option<usize>, output: &mut Vec<Option<usize>>) {
    let index = output.len();
    output.push(parent);
    for child in &element.children {
        collect_parents(child, Some(index), output);
    }
}

fn element_at(root: &Element, target: usize) -> Option<&Element> {
    fn visit<'a>(element: &'a Element, target: usize, index: &mut usize) -> Option<&'a Element> {
        if *index == target {
            return Some(element);
        }
        *index += 1;
        for child in &element.children {
            if let Some(found) = visit(child, target, index) {
                return Some(found);
            }
        }
        None
    }
    visit(root, target, &mut 0)
}

#[cfg(test)]
mod tests {
    use crate::{EventListenerOptions, EventType};

    use super::*;

    #[test]
    fn dispatches_capture_target_and_bubble_in_order() {
        let root = Element::column([Element::container([
            Element::text("target").listen(EventType::Click, EventListenerOptions::default())
        ])
        .listen(EventType::Click, EventListenerOptions::default())])
        .listen(
            EventType::Click,
            EventListenerOptions::default().capture(true),
        );
        let mut tree = UiTree::new(root);
        let target = tree.node_id_at(2).unwrap();
        let events = tree.event_deliveries(target, UiEventKind::Clicked);
        assert_eq!(
            events.iter().map(UiEvent::phase).collect::<Vec<_>>(),
            [EventPhase::Capture, EventPhase::Target, EventPhase::Bubble]
        );
    }

    #[test]
    fn once_listener_is_consumed_until_removed() {
        let root = Element::text("target")
            .listen(EventType::Click, EventListenerOptions::default().once(true));
        let mut tree = UiTree::new(root);
        let target = tree.node_id_at(0).unwrap();
        let events = tree.event_deliveries(target, UiEventKind::Clicked);
        assert_eq!(events.len(), 1);
        assert!(events[0].should_dispatch());
        assert!(
            tree.event_deliveries(target, UiEventKind::Clicked)
                .is_empty()
        );
    }

    #[test]
    fn non_bubbling_events_ignore_ancestors_and_unrelated_listeners() {
        let root = Element::container([Element::text("target")
            .listen(EventType::PointerEnter, EventListenerOptions::default())
            .listen(EventType::Click, EventListenerOptions::default())])
        .listen(EventType::PointerEnter, EventListenerOptions::default());
        let mut tree = UiTree::new(root);
        let target = tree.node_id_at(1).unwrap();
        let events = tree.event_deliveries(target, UiEventKind::PointerEntered);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].current_target(), target);
        assert_eq!(events[0].phase(), EventPhase::Target);
    }
}
