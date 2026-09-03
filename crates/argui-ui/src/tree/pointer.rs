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
        for gesture in gestures {
            update
                .events
                .extend(self.event_deliveries(gesture.target, UiEventKind::Gesture(gesture)));
        }
        update
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
