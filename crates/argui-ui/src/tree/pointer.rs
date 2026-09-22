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
        let mut update = self.decorate_pointer(update);
        self.dismiss_on_hover_exit(point, regions, &mut update);
        update
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

    /// Updates pointer location during another phase without delivering a synthetic move.
    ///
    /// * `event` — press or release sample whose location should be observed.
    /// * `regions` — current hit regions for hover and drag-slop calculation.
    ///
    /// Returns hover and visual-state changes caused by positioning the pointer.
    fn position_for_transition(
        &mut self,
        event: PointerEvent,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let mut raw = self
            .interaction
            .pointer_moved(with_phase(event, PointerPhase::Moved), regions);
        raw.events.retain(|(_, kind)| {
            !matches!(
                kind,
                UiEventKind::Pointer(PointerEvent {
                    phase: PointerPhase::Moved,
                    ..
                })
            )
        });
        self.decorate(raw)
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
                    let mut moved = self.position_for_transition(event, regions);
                    moved.merge(self.primary_pressed_for(event, regions));
                    if let Some((node, gestures)) = hit
                        && gestures.captures_on_press()
                    {
                        moved.merge(self.capture_pointer(event.id, node));
                    }
                    moved
                }
                PointerPhase::Released => {
                    let mut moved = self.position_for_transition(event, regions);
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
        if event.kind == PointerKind::Mouse
            && matches!(event.phase, PointerPhase::Moved | PointerPhase::Left)
        {
            self.dismiss_on_hover_exit(event.position, regions, &mut update);
        }
        self.dispatch_gestures(gestures, &mut update);
        update
    }

    fn light_dismiss_outside(&self, hit: Option<NodeId>) -> Option<NodeId> {
        let hit_index = hit.and_then(|node| self.index.position(node));
        let elements = crate::traversal::flattened(self.root());
        let topmost = elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                let portal = element.portal.as_ref()?;
                Some((portal.layer, element.z_index, index))
            })
            .max()?;
        elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                let portal = element.portal.as_ref()?;
                (self.contains_index(index, topmost.2)
                    && matches!(
                        portal.dismiss,
                        crate::DismissPolicy::OutsidePointer
                            | crate::DismissPolicy::OutsidePointerOrEscape
                            | crate::DismissPolicy::OutsideHoverOrEscape
                    ))
                .then_some((portal.layer, element.z_index, index, portal))
            })
            .max_by_key(|(layer, z_index, index, _)| (*layer, *z_index, *index))
            .and_then(|(_, _, index, portal)| {
                hit_index
                    .is_none_or(|hit| {
                        !self.contains_index(index, hit)
                            && !self.portal_anchor_contains(portal, hit)
                    })
                    .then_some(self.node_ids[index])
            })
    }

    /// Requests hover dismissal after the pointer leaves both a portal and its anchor.
    ///
    /// * `point` — current pointer position in window coordinates.
    /// * `regions` — hit geometry for the current frame.
    /// * `update` — interaction update receiving a dismissal delivery when applicable.
    fn dismiss_on_hover_exit(
        &mut self,
        point: Point,
        regions: &[HitRegion],
        update: &mut InteractionUpdate,
    ) {
        let elements = crate::traversal::flattened(self.root());
        let Some((index, portal)) = elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                element
                    .portal
                    .as_ref()
                    .map(|portal| (portal.layer, element.z_index, index, portal))
            })
            .max_by_key(|(layer, z_index, index, _)| (*layer, *z_index, *index))
            .and_then(|(_, _, index, portal)| {
                (portal.dismiss == crate::DismissPolicy::OutsideHoverOrEscape)
                    .then_some((index, portal))
            })
        else {
            return;
        };
        let anchor_index = match &portal.target {
            crate::PortalTarget::Anchor(anchor) => elements
                .iter()
                .position(|element| element.key.as_deref() == Some(anchor.key.as_str())),
            _ => None,
        };
        let near_surface = regions.iter().any(|region| {
            let Some(region_index) = self.index.position(region.node) else {
                return false;
            };
            (self.contains_index(index, region_index)
                || anchor_index.is_some_and(|anchor| self.contains_index(anchor, region_index)))
                && near_region(region, point, 8.0)
        });
        if !near_surface {
            update
                .events
                .extend(self.event_deliveries(self.node_ids[index], UiEventKind::DismissRequested));
        }
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

    /// Returns whether `pointer` is captured by an active drag interaction.
    ///
    /// Runtime hosts use this to avoid running default touch scrolling while a
    /// widget-owned drag, such as a splitter or slider, owns the contact.
    #[must_use]
    pub fn pointer_captured(&self, pointer: PointerId) -> bool {
        self.interaction.captured_node(pointer).is_some()
    }

    /// Applies the accepted pointer press's opt-in capture policy.
    ///
    /// * `pointer` — pointer whose press may begin capture.
    /// * `press_event` — delivered PointerDown event after listeners run, or
    ///   `None` when there were no listeners. A prevented or mismatched event
    ///   cannot start capture.
    /// * `regions` — current hit regions used to verify the pressed target is enabled.
    ///
    /// Returns capture events and visual changes, or an empty update when the
    /// press was rejected or the target did not opt into capture. Hosts call
    /// this after delivering PointerDown, before routing subsequent movement.
    pub fn pointer_press_default(
        &mut self,
        pointer: PointerId,
        press_event: Option<&crate::UiEvent>,
        regions: &[HitRegion],
    ) -> InteractionUpdate {
        let Some(target) = self.interaction.pressed_target(pointer) else {
            return InteractionUpdate::default();
        };
        if press_event.is_some_and(|delivery| {
            delivery.default_prevented()
                || delivery.target != target
                || !matches!(
                    &delivery.kind,
                    UiEventKind::Pointer(event)
                        if event.id == pointer && event.phase == PointerPhase::Pressed
                )
        }) {
            return InteractionUpdate::default();
        }
        if !regions
            .iter()
            .any(|region| region.node == target && region.enabled)
            || !self.element_for(target).is_some_and(|element| {
                element
                    .interaction
                    .as_ref()
                    .is_some_and(|interaction| interaction.enabled && interaction.capture_on_press)
            })
        {
            return InteractionUpdate::default();
        }
        self.capture_pointer(pointer, target)
    }

    /// Returns a pointer's position relative to an interacting node.
    ///
    /// * `pointer` — mouse, pen, or touch contact to observe.
    /// * `node` — hovered or captured node whose local coordinates are requested.
    /// * `regions` — current hit regions providing the node's bounds and transform.
    ///
    /// Returns `None` when the pointer is neither hovering nor captured by the
    /// node, the region is absent, or its transform is not invertible. Captured
    /// pointers can report positions outside the node's bounds.
    #[must_use]
    pub fn pointer_position(
        &self,
        pointer: PointerId,
        node: NodeId,
        regions: &[HitRegion],
    ) -> Option<Point> {
        let position = self.interaction.pointer_position(pointer, node)?;
        regions
            .iter()
            .find(|region| region.node == node)
            .and_then(|region| region.local_point(position))
    }

    /// Returns a pointer's window position while it hovers or is captured by a node.
    ///
    /// * `pointer` — mouse, pen, or touch contact to observe.
    /// * `node` — hovered or captured interaction target.
    ///
    /// Returns `None` when the pointer no longer interacts with the node. The
    /// coordinate remains stable when the node moves during a captured drag.
    #[must_use]
    pub fn pointer_global_position(&self, pointer: PointerId, node: NodeId) -> Option<Point> {
        self.interaction.pointer_position(pointer, node)
    }

    /// Returns the last press position relative to the node's bounds at press time.
    ///
    /// * `pointer` — pointer whose most recent press is requested.
    /// * `node` — node that received that press.
    ///
    /// Returns `None` until the specified pointer presses the node. The last
    /// position remains available after release and is replaced by a later press.
    #[must_use]
    pub fn pressed_position(&self, pointer: PointerId, node: NodeId) -> Option<Point> {
        self.pressed_positions
            .get(&node)
            .and_then(|(pressed_pointer, position)| {
                (*pressed_pointer == pointer).then_some(*position)
            })
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
                    *pending = coalesce_gesture(pending, gesture);
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

/// Returns whether a pointer is inside a hit region or its small hover bridge.
///
/// * `region` — region to inspect in its local coordinate space.
/// * `point` — pointer position in window coordinates.
/// * `margin` — extra distance around every side in logical pixels.
fn near_region(region: &HitRegion, point: Point, margin: f32) -> bool {
    region.local_point(point).is_some_and(|local| {
        local.x >= -margin
            && local.y >= -margin
            && local.x <= region.bounds.size.width + margin
            && local.y <= region.bounds.size.height + margin
    })
}

/// Keeps the latest frame sample while preserving motion accumulated since the last delivery.
fn coalesce_gesture(previous: &GestureEvent, mut latest: GestureEvent) -> GestureEvent {
    if let (
        GestureKind::Pan {
            delta: previous_delta,
            ..
        },
        GestureKind::Pan { delta, .. },
    ) = (&previous.kind, &mut latest.kind)
    {
        *delta = Point::new(delta.x + previous_delta.x, delta.y + previous_delta.y);
    }
    latest
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
