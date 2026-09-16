use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};

use crate::{
    GestureDelivery, GestureEvent, GestureKind, GesturePhase, HitRegion, InteractionUpdate, NodeId,
    UiEventKind,
};

use super::UiTree;

impl UiTree {
    /// Updates hover state for a pointer position.
    ///
    /// * `point` — pointer position in window coordinates.
    /// * `regions` — current hit-test regions.
    ///
    /// Returns the interaction changes caused by updating hover state.
    pub fn pointer_moved(&mut self, point: Point, regions: &[HitRegion]) -> InteractionUpdate {
        let update = self
            .interaction
            .pointer_moved(PointerEvent::mouse(PointerPhase::Moved, point), regions);
        self.decorate_pointer(update)
    }

    fn decorate_pointer(&mut self, update: crate::interaction::RawUpdate) -> InteractionUpdate {
        if update.paint_changed {
            return self.decorate(update);
        }
        InteractionUpdate {
            events: update
                .events
                .into_iter()
                .flat_map(|(target, kind)| self.event_deliveries(target, kind))
                .collect(),
            ..InteractionUpdate::default()
        }
    }

    /// Processes a pointer phase, dispatching interactions and gesture events.
    ///
    /// * `event` — pointer identifier, kind, phase, and position.
    /// * `regions` — current hit-test and gesture regions.
    ///
    /// Returns interaction changes and delivered events for this pointer phase.
    pub fn pointer_event(
        &mut self,
        event: PointerEvent,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        self.interaction_bounds.clear();
        self.interaction_bounds
            .extend(regions.iter().map(|region| (region.node, region.bounds)));
        let region = regions
            .iter()
            .rev()
            .find(|region| region.contains(event.position));
        // Disabled controls still belong to the panel containing them.
        let hit_node = region.map(|region| region.node);
        let hit = region
            .filter(|region| region.enabled)
            .map(|region| (region.node, region.gestures));
        let gestures = self.gestures.update(event, hit);
        let mut update = if event.kind == PointerKind::Touch && !event.primary {
            InteractionUpdate::default()
        } else {
            match event.phase {
                PointerPhase::Entered | PointerPhase::Moved => {
                    let update = self.interaction.pointer_moved(event, regions);
                    self.decorate_pointer(update)
                }
                PointerPhase::Pressed => {
                    let moved = self
                        .interaction
                        .pointer_moved(with_phase(event, PointerPhase::Moved), regions);
                    let mut moved = self.decorate(moved);
                    moved.merge(self.primary_pressed_for(event, regions));
                    if let Some((node, gestures)) = hit
                        && gestures.captures_on_press()
                    {
                        moved.merge(self.capture_pointer(event.id, node));
                    }
                    moved
                }
                PointerPhase::Released => {
                    let moved = self
                        .interaction
                        .pointer_moved(with_phase(event, PointerPhase::Moved), regions);
                    let mut moved = self.decorate(moved);
                    moved.merge(self.primary_released_for(event));
                    if event.kind == PointerKind::Touch {
                        let left = self
                            .interaction
                            .pointer_left(with_phase(event, PointerPhase::Left));
                        moved.merge(self.decorate(left));
                    }
                    moved
                }
                PointerPhase::Left => {
                    let update = self.interaction.pointer_left(event);
                    self.decorate(update)
                }
                PointerPhase::Cancelled => {
                    let update = self.interaction.primary_cancelled(event);
                    let mut update = self.decorate(update);
                    if event.kind == PointerKind::Touch {
                        let left = self
                            .interaction
                            .pointer_left(with_phase(event, PointerPhase::Left));
                        update.merge(self.decorate(left));
                    }
                    update
                }
            }
        };
        if event.phase == PointerPhase::Pressed
            && let Some(portal) = self.light_dismiss_outside(hit_node)
        {
            update
                .events
                .extend(self.event_deliveries(portal, UiEventKind::PointerOutside(event)));
        }
        self.dispatch_gestures(gestures, &mut update);
        update
    }

    fn light_dismiss_outside(&self, hit: Option<NodeId>) -> Option<NodeId> {
        let hit_index = hit.and_then(|node| self.index.position(node));
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
            let Some(parent) = self.index.parent(node) else {
                return false;
            };
            node = parent;
        }
    }

    /// Clears mouse hover state after the pointer leaves the host window.
    /// Returns the resulting interaction update.
    pub fn pointer_left(&mut self) -> InteractionUpdate {
        let update = self
            .interaction
            .pointer_left(PointerEvent::mouse(PointerPhase::Left, Point::default()));
        self.decorate(update)
    }

    /// Processes release of the primary mouse button.
    /// Returns the resulting interaction update.
    pub fn primary_released(&mut self) -> InteractionUpdate {
        let position = self.interaction.mouse_position().unwrap_or_default();
        let update = self.interaction.primary_released(PointerEvent {
            button: Some(argui_core::PointerButton::Primary),
            phase: PointerPhase::Released,
            ..PointerEvent::mouse(PointerPhase::Released, position)
        });
        self.decorate(update)
    }

    /// Captures a pointer for a target node.
    ///
    /// * `pointer` — pointer identifier to capture.
    /// * `target` — node that receives the captured interaction.
    ///
    /// Returns the resulting interaction update.
    pub fn capture_pointer(&mut self, pointer: PointerId, target: NodeId) -> InteractionUpdate {
        let update = self.interaction.capture_pointer(pointer, target);
        self.decorate(update)
    }

    /// A captured drag retains its cursor even when the pointer leaves its hit region.
    ///
    /// * `pointer` — captured pointer identifier.
    /// * `regions` — current hit regions, used to obtain the target cursor.
    ///
    /// Returns the non-default cursor for the capture target, if available.
    pub fn captured_cursor(
        &self,
        pointer: PointerId,
        regions: &[HitRegion],
    ) -> Option<crate::CursorIcon> {
        let node = self.interaction.captured_node(pointer)?;
        regions
            .iter()
            .find(|region| region.node == node && region.enabled)
            .map(|region| region.cursor)
            .filter(|cursor| *cursor != crate::CursorIcon::Auto)
    }

    /// Releases a pointer capture owned by `target`.
    ///
    /// * `pointer` — pointer identifier to release.
    /// * `target` — node expected to own the capture.
    ///
    /// Returns the resulting interaction update.
    pub fn release_pointer_capture(
        &mut self,
        pointer: PointerId,
        target: NodeId,
    ) -> InteractionUpdate {
        let update = self.interaction.release_pointer(pointer, target);
        self.decorate(update)
    }

    fn primary_released_for(&mut self, event: PointerEvent) -> InteractionUpdate {
        let update = self.interaction.primary_released(event);
        self.decorate(update)
    }

    /// Suspends focus and cancels active pointer and gesture state on window blur.
    /// Returns the resulting interaction update.
    pub fn window_blurred(&mut self) -> InteractionUpdate {
        self.suspend_focus();
        let update = self.interaction.window_blurred();
        let mut update = self.decorate(update);
        let gestures = self.gestures.cancel_all();
        self.dispatch_gestures(gestures, &mut update);
        update
    }

    /// Emits the newest pending update for each frame-coalesced gesture stream.
    /// Returns the delivered gesture events for this frame.
    pub fn flush_gesture_frame(&mut self) -> InteractionUpdate {
        let pending = std::mem::take(&mut self.pending_gestures);
        let mut update = InteractionUpdate::default();
        for gesture in pending {
            update
                .events
                .extend(self.event_deliveries(gesture.target, UiEventKind::Gesture(gesture)));
        }
        update
    }

    fn dispatch_gestures(&mut self, gestures: Vec<GestureEvent>, update: &mut InteractionUpdate) {
        for gesture in gestures {
            if gesture.phase == GesturePhase::Changed
                && gesture.delivery == GestureDelivery::FrameCoalesced
            {
                if let Some(pending) = self
                    .pending_gestures
                    .iter_mut()
                    .find(|pending| same_gesture_stream(pending, &gesture))
                {
                    *pending = gesture;
                } else {
                    self.pending_gestures.push(gesture);
                }
                update.frame_requested = true;
                continue;
            }
            self.flush_gesture_stream(&gesture, update);
            update
                .events
                .extend(self.event_deliveries(gesture.target, UiEventKind::Gesture(gesture)));
        }
    }

    fn flush_gesture_stream(&mut self, gesture: &GestureEvent, update: &mut InteractionUpdate) {
        let pending = std::mem::take(&mut self.pending_gestures);
        for candidate in pending {
            if same_gesture_stream(&candidate, gesture) {
                update.events.extend(
                    self.event_deliveries(candidate.target, UiEventKind::Gesture(candidate)),
                );
            } else {
                self.pending_gestures.push(candidate);
            }
        }
    }
}

fn with_phase(mut event: PointerEvent, phase: PointerPhase) -> PointerEvent {
    event.phase = phase;
    event
}

fn same_gesture_stream(left: &GestureEvent, right: &GestureEvent) -> bool {
    left.target == right.target
        && left.pointer == right.pointer
        && matches!(
            (&left.kind, &right.kind),
            (GestureKind::Pan { .. }, GestureKind::Pan { .. })
                | (GestureKind::Pinch { .. }, GestureKind::Pinch { .. })
                | (GestureKind::Rotation { .. }, GestureKind::Rotation { .. })
                | (GestureKind::Tap { .. }, GestureKind::Tap { .. })
        )
}
