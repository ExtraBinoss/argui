use argui_core::{Point, PointerButton, PointerEvent, PointerId, PointerPhase};

use crate::{ClickEvent, HitRegion, NodeId, UiEventKind};

use super::{InteractionState, RawUpdate};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ClickRecord {
    target: NodeId,
    button: Option<PointerButton>,
    position: Point,
    timestamp: std::time::Duration,
    count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PressRecord {
    pub target: NodeId,
    origin: Point,
    pub activation_cancelled: bool,
}

impl HitRegion {
    /// Converts a window-space point to coordinates relative to this region's bounds.
    ///
    /// * `point` — point in window coordinates.
    ///
    /// Returns `None` for a singular transform; captured drags may return points outside bounds.
    #[must_use]
    pub fn local_point(&self, point: Point) -> Option<Point> {
        let point = self.transform.inverse()?.transform_point(point);
        let x = point.x - self.bounds.origin.x;
        let y = point.y - self.bounds.origin.y;
        Some(Point::new(x, y))
    }
}

impl InteractionState {
    /// Returns the target of an active primary press by `pointer`.
    ///
    /// * `pointer` — pointer whose active press is requested.
    ///
    /// Returns `None` when this pointer has no active press.
    pub(crate) fn pressed_target(&self, pointer: PointerId) -> Option<NodeId> {
        self.pressed.get(&pointer).map(|press| press.target)
    }

    /// Returns a pointer's position while it hovers or is captured by `node`.
    ///
    /// * `pointer` — pointer whose position is requested.
    /// * `node` — interaction target being observed.
    ///
    /// Returns a window-space point, or `None` when the pointer is no longer on the node.
    pub(crate) fn pointer_position(&self, pointer: PointerId, node: NodeId) -> Option<Point> {
        if self.hovered.get(&pointer) != Some(&node) && self.captured.get(&pointer) != Some(&node) {
            return None;
        }
        self.pointer_positions.get(&pointer).copied()
    }

    pub fn pointer_moved(&mut self, event: PointerEvent, regions: &[HitRegion]) -> RawUpdate {
        let hit = hit_test(regions, event.position).filter(|region| region.enabled);
        if event.id == PointerId::MOUSE || hit.is_some() || self.captured.contains_key(&event.id) {
            self.pointer_positions.insert(event.id, event.position);
        } else {
            self.pointer_positions.remove(&event.id);
        }
        let hovered = self.hovered.get(&event.id).copied();
        let mut update = RawUpdate::default();
        if hit.map(|region| region.node) != hovered {
            if let Some(previous) = hovered {
                update.push(
                    previous,
                    UiEventKind::Pointer(with_phase(event, PointerPhase::Left)),
                );
            }
            if let Some(node) = hit.map(|region| region.node) {
                self.hovered.insert(event.id, node);
                update.push(
                    node,
                    UiEventKind::Pointer(with_phase(event, PointerPhase::Entered)),
                );
            } else {
                self.hovered.remove(&event.id);
            }
            update.paint_changed = true;
        }
        if event.phase == PointerPhase::Moved
            && let Some(press) = self.pressed.get_mut(&event.id)
            && !press.activation_cancelled
        {
            let delta = Point::new(
                event.position.x - press.origin.x,
                event.position.y - press.origin.y,
            );
            let slop = self.pointer_settings.activation_slop_distance();
            if !delta.x.is_finite()
                || !delta.y.is_finite()
                || delta.x * delta.x + delta.y * delta.y > slop * slop
            {
                let keeps_pressed = self.captured.get(&event.id) == Some(&press.target);
                press.activation_cancelled = true;
                self.last_click = None;
                update.paint_changed |= !keeps_pressed;
            }
        }
        if event.phase == PointerPhase::Moved
            && let Some(target) = self
                .captured
                .get(&event.id)
                .copied()
                .or_else(|| self.hovered.get(&event.id).copied())
        {
            update.push(target, UiEventKind::Pointer(event));
        }
        update
    }

    pub fn pointer_left(&mut self, event: PointerEvent) -> RawUpdate {
        let mut update = RawUpdate::default();
        self.pointer_positions.remove(&event.id);
        if let Some(node) = self.hovered.remove(&event.id) {
            update.push(node, UiEventKind::Pointer(event));
            update.paint_changed = true;
        }
        update
    }

    pub fn primary_pressed(&mut self, event: PointerEvent) -> RawUpdate {
        let mut update = RawUpdate::default();
        let Some(target) = self.hovered.get(&event.id).copied() else {
            return update;
        };
        self.pointer_positions.insert(event.id, event.position);
        self.pressed.insert(
            event.id,
            PressRecord {
                target,
                origin: event.position,
                activation_cancelled: false,
            },
        );
        update.push(target, UiEventKind::Pointer(event));
        update.paint_changed = true;
        update
    }

    pub fn primary_released(&mut self, event: PointerEvent) -> RawUpdate {
        let mut update = RawUpdate::default();
        if event.id != PointerId::MOUSE {
            self.pointer_positions.remove(&event.id);
        }
        let captured = self.captured.remove(&event.id);
        let hovered = self.hovered.get(&event.id).copied();
        let pressed = self.pressed.remove(&event.id);
        if let Some(target) = captured.or(hovered) {
            update.push(target, UiEventKind::Pointer(event));
        }
        if let Some(target) = captured {
            update.push(target, UiEventKind::LostPointerCapture(event.id));
        }
        if let Some(press) = pressed
            && !press.activation_cancelled
            && hovered == Some(press.target)
        {
            let count = self.click_count(press.target, event);
            update.push(
                press.target,
                UiEventKind::Click(ClickEvent::pointer(event, count)),
            );
        }
        update.paint_changed = pressed.is_some();
        update
    }

    pub fn primary_cancelled(&mut self, event: PointerEvent) -> RawUpdate {
        let mut update = RawUpdate::default();
        if event.id != PointerId::MOUSE {
            self.pointer_positions.remove(&event.id);
        }
        let captured = self.captured.remove(&event.id);
        if let Some(target) =
            captured.or_else(|| self.pressed.get(&event.id).map(|press| press.target))
        {
            update.push(target, UiEventKind::Pointer(event));
        }
        if let Some(target) = captured {
            update.push(target, UiEventKind::LostPointerCapture(event.id));
        }
        update.paint_changed = self.pressed.remove(&event.id).is_some();
        self.last_click = None;
        update
    }

    pub fn capture_pointer(&mut self, pointer: PointerId, target: NodeId) -> RawUpdate {
        let mut update = RawUpdate::default();
        if self.captured.get(&pointer) == Some(&target) {
            return update;
        }
        if let Some(previous) = self.captured.insert(pointer, target)
            && previous != target
        {
            update.push(previous, UiEventKind::LostPointerCapture(pointer));
        }
        update.push(target, UiEventKind::GotPointerCapture(pointer));
        update
    }

    pub fn release_pointer(&mut self, pointer: PointerId, target: NodeId) -> RawUpdate {
        let mut update = RawUpdate::default();
        if self.captured.get(&pointer) == Some(&target) {
            self.captured.remove(&pointer);
            update.push(target, UiEventKind::LostPointerCapture(pointer));
        }
        update
    }

    pub fn window_blurred(&mut self) -> RawUpdate {
        let mut update = RawUpdate::default();
        for (pointer, press) in self.pressed.drain() {
            update.push(
                press.target,
                UiEventKind::Pointer(PointerEvent {
                    id: pointer,
                    phase: PointerPhase::Cancelled,
                    ..PointerEvent::mouse(PointerPhase::Cancelled, Point::default())
                }),
            );
            update.paint_changed = true;
        }
        self.hovered.clear();
        self.pointer_positions.clear();
        self.release_keyboard(&mut update, None);
        for (pointer, target) in self.captured.drain() {
            update.push(target, UiEventKind::LostPointerCapture(pointer));
        }
        if let Some(target) = self.focused.take() {
            update.push(target, UiEventKind::Blurred);
            update.paint_changed = true;
        }
        self.focus_visible = false;
        update
    }

    fn click_count(&mut self, target: NodeId, event: PointerEvent) -> u8 {
        let count = self.last_click.as_ref().map_or(1, |previous| {
            let delta = Point::new(
                event.position.x - previous.position.x,
                event.position.y - previous.position.y,
            );
            if previous.target == target
                && previous.button == event.button
                && event.timestamp.saturating_sub(previous.timestamp)
                    <= self.pointer_settings.multi_click_interval()
                && delta.x * delta.x + delta.y * delta.y
                    <= self.pointer_settings.multi_click_distance()
                        * self.pointer_settings.multi_click_distance()
            {
                previous.count.saturating_add(1)
            } else {
                1
            }
        });
        self.last_click = Some(ClickRecord {
            target,
            button: event.button,
            position: event.position,
            timestamp: event.timestamp,
            count,
        });
        count
    }
}

fn hit_test(regions: &[HitRegion], point: Point) -> Option<&HitRegion> {
    regions.iter().rev().find(|region| region.contains(point))
}

fn with_phase(mut event: PointerEvent, phase: PointerPhase) -> PointerEvent {
    event.phase = phase;
    event
}
