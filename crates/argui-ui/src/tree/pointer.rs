use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};

use crate::{HitRegion, InteractionUpdate, NodeId, UiEventKind};

use super::UiTree;

impl UiTree {
    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self
            .interaction
            .pointer_moved(PointerId::MOUSE, point, regions);
        self.decorate(update)
    }

    pub fn pointer_event(
        &mut self,
        event: PointerEvent,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let hit = regions
            .iter()
            .rev()
            .find(|region| region.contains(event.position))
            .filter(|region| region.enabled)
            .map(|region| (region.node, region.gestures));
        let hit_node = hit.map(|(node, _)| node);
        let gestures = self.gestures.update(event, hit);
        let mut update = if event.kind == PointerKind::Touch && !event.primary {
            InteractionUpdate::default()
        } else {
            match event.phase {
                PointerPhase::Entered | PointerPhase::Moved => {
                    let update = self
                        .interaction
                        .pointer_moved(event.id, event.position, regions);
                    self.decorate(update)
                }
                PointerPhase::Pressed => {
                    let moved = self
                        .interaction
                        .pointer_moved(event.id, event.position, regions);
                    let mut moved = self.decorate(moved);
                    moved.merge(self.primary_pressed_for(event.id, regions));
                    moved
                }
                PointerPhase::Released => {
                    let moved = self
                        .interaction
                        .pointer_moved(event.id, event.position, regions);
                    let mut moved = self.decorate(moved);
                    moved.merge(self.primary_released_for(event.id));
                    moved
                }
                PointerPhase::Left => {
                    let update = self.interaction.pointer_left(event.id);
                    self.decorate(update)
                }
                PointerPhase::Cancelled => {
                    let update = self.interaction.primary_cancelled(event.id);
                    self.decorate(update)
                }
            }
        };
        if event.phase == PointerPhase::Pressed
            && let Some(portal) = self.light_dismiss_outside(hit_node)
        {
            update
                .events
                .extend(self.event_deliveries(portal, UiEventKind::PointerOutside));
        }
        for gesture in gestures {
            update
                .events
                .extend(self.event_deliveries(gesture.target, UiEventKind::Gesture(gesture)));
        }
        update
    }

    fn light_dismiss_outside(&self, hit: Option<NodeId>) -> Option<NodeId> {
        let hit_index = hit.and_then(|node| self.node_ids.iter().position(|id| *id == node));
        crate::traversal::flattened(self.root())
            .into_iter()
            .enumerate()
            .filter_map(|(index, element)| {
                let portal = element.portal.as_ref()?;
                (portal.dismiss == crate::DismissPolicy::OutsidePointer
                    && hit_index.is_none_or(|hit| {
                        !self.contains_index(index, hit)
                            && !self.portal_anchor_contains(portal, hit)
                    }))
                .then_some((
                    portal.layer,
                    element.z_index,
                    index,
                    self.node_ids[index],
                ))
            })
            .max_by_key(|(layer, z_index, index, _)| (*layer, *z_index, *index))
            .map(|(_, _, _, node)| node)
    }

    fn portal_anchor_contains(&self, portal: &crate::Portal, hit: usize) -> bool {
        let crate::PortalTarget::Anchor(anchor) = &portal.target else {
            return false;
        };
        crate::traversal::flattened(self.root())
            .iter()
            .position(|element| element.key.as_deref() == Some(anchor.key.as_str()))
            .is_some_and(|anchor| self.contains_index(anchor, hit))
    }

    fn contains_index(&self, ancestor: usize, mut node: usize) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            let Some(parent) = self.events.parent(node) else {
                return false;
            };
            node = parent;
        }
    }

    pub fn pointer_left(&mut self) -> InteractionUpdate {
        let update = self.interaction.pointer_left(PointerId::MOUSE);
        self.decorate(update)
    }

    pub fn primary_released(&mut self) -> InteractionUpdate {
        let update = self.interaction.primary_released(PointerId::MOUSE);
        self.decorate(update)
    }

    pub fn capture_pointer(&mut self, pointer: PointerId, target: NodeId) -> InteractionUpdate {
        let update = self.interaction.capture_pointer(pointer, target);
        self.decorate(update)
    }

    pub fn release_pointer_capture(
        &mut self,
        pointer: PointerId,
        target: NodeId,
    ) -> InteractionUpdate {
        let update = self.interaction.release_pointer(pointer, target);
        self.decorate(update)
    }

    fn primary_released_for(&mut self, pointer: PointerId) -> InteractionUpdate {
        let update = self.interaction.primary_released(pointer);
        self.decorate(update)
    }

    pub fn window_blurred(&mut self) -> InteractionUpdate {
        self.suspend_focus();
        let update = self.interaction.window_blurred();
        let mut update = self.decorate(update);
        for gesture in self.gestures.cancel_all() {
            update
                .events
                .extend(self.event_deliveries(gesture.target, UiEventKind::Gesture(gesture)));
        }
        update
    }
}
