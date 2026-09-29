//! Immediate retained layout overrides for native resize gestures.

use argui_core::{PointerButton, PointerEvent, PointerPhase, Size};

use crate::{Dimension, InteractionUpdate, NodeId, ResizeAxis, UiEventKind, length};

use super::UiTree;
mod types;
pub(super) use types::ResizeState;
use types::{Drag, Override, Target};

impl UiTree {
    /// Resolves unique target keys and retains overrides through unrelated producer updates.
    pub(super) fn sync_resize_handles(&mut self) {
        let mut keys = std::collections::HashMap::new();
        for (index, node) in self.node_ids.iter().enumerate() {
            if let Some(key) = self
                .index
                .at(index)
                .and_then(|element| element.key.as_ref())
            {
                keys.entry(key.clone())
                    .and_modify(|node| *node = None)
                    .or_insert(Some(*node));
            }
        }
        self.resize.targets.clear();
        for (index, handle) in self.node_ids.iter().enumerate() {
            let Some(config) = self
                .index
                .at(index)
                .and_then(|element| element.interaction.as_ref())
                .filter(|interaction| interaction.enabled)
                .and_then(|interaction| interaction.resize.as_ref())
            else {
                continue;
            };
            if let Some(Some(node)) = keys.get(&config.target) {
                self.resize.targets.insert(
                    *handle,
                    Target {
                        node: *node,
                        config: config.clone(),
                    },
                );
            }
        }
        if self.resize.drag.as_ref().is_some_and(|drag| {
            self.resize.targets.get(&drag.handle).is_none_or(|target| {
                target.node != drag.target.node || target.config != drag.target.config
            }) || self.index.element(drag.target.node).is_none_or(|element| {
                dimension(&element.style, drag.target.config.axis) != drag.authored
            })
        }) {
            let drag = self.resize.drag.take().expect("checked active drag");
            self.restore_resize_drag(drag);
        }
        let active = self
            .resize
            .targets
            .values()
            .map(|target| target.node)
            .collect::<std::collections::HashSet<_>>();
        self.resize.overrides.retain(|node, value| {
            let retained = active.contains(node)
                && self
                    .resize
                    .targets
                    .values()
                    .any(|target| target.node == *node && target.config.axis == value.axis)
                && self
                    .index
                    .element(*node)
                    .is_some_and(|element| dimension(&element.style, value.axis) == value.authored);
            if !retained && self.index.position(*node).is_some() {
                self.resize.restored.insert(*node);
            }
            retained
        });
        self.resize.bounds.retain(|node, _| active.contains(node));
        self.resize
            .restored
            .retain(|node| self.index.position(*node).is_some());
    }

    /// Records measured pane sizes after layout, without observing them in JavaScript.
    /// `bounds` supplies retained IDs and actual logical sizes from the layout engine.
    pub fn observe_resize_bounds(&mut self, bounds: impl IntoIterator<Item = (NodeId, Size)>) {
        if self.resize.targets.is_empty() {
            return;
        }
        for (node, size) in bounds {
            if self
                .resize
                .targets
                .values()
                .any(|target| target.node == node)
            {
                self.resize.bounds.insert(node, size);
            }
        }
    }

    /// Applies the native override for `node` to its otherwise resolved layout `style`.
    pub(super) fn apply_resize_style(&self, node: NodeId, style: &mut crate::LayoutStyle) {
        if let Some(value) = self.resize.overrides.get(&node) {
            match value.axis {
                ResizeAxis::Horizontal => style.size.width = length(value.value),
                ResizeAxis::Vertical => style.size.height = length(value.value),
            }
        }
    }

    /// Lists only targets whose cached layout style needs native resizing or restoration.
    pub(super) fn resize_layout_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.resize
            .overrides
            .keys()
            .chain(&self.resize.restored)
            .filter_map(|node| self.index.position(*node))
    }

    /// Handles `event` against its enabled hit target, retaining one contact until release.
    pub(super) fn resize_pointer(
        &mut self,
        event: PointerEvent,
        hit: Option<NodeId>,
    ) -> InteractionUpdate {
        let mut update = InteractionUpdate::default();
        if event.phase == PointerPhase::Pressed && event.button == Some(PointerButton::Primary) {
            if self.resize.drag.is_some() {
                return update;
            }
            let Some((handle, target)) = hit.and_then(|handle| {
                self.resize
                    .targets
                    .get(&handle)
                    .cloned()
                    .map(|target| (handle, target))
            }) else {
                return update;
            };
            if !target.config.minimum.is_finite()
                || !target.config.maximum.is_finite()
                || target.config.minimum < 0.0
                || target.config.maximum < target.config.minimum
            {
                return update;
            }
            let Some(size) = self.resize.bounds.get(&target.node) else {
                return update;
            };
            let initial = match target.config.axis {
                ResizeAxis::Horizontal => size.width,
                ResizeAxis::Vertical => size.height,
            };
            let origin = coordinate(event, target.config.axis);
            if !initial.is_finite() || !origin.is_finite() {
                return update;
            }
            let previous = self.resize.overrides.get(&target.node).copied();
            let Some(element) = self.index.element(target.node) else {
                return update;
            };
            let authored = dimension(&element.style, target.config.axis);
            self.resize.drag = Some(Drag {
                handle,
                pointer: event.id,
                target,
                origin,
                initial,
                authored,
                previous,
            });
            update.merge(self.capture_pointer(event.id, handle));
            return update;
        }
        let Some(drag) = self.resize.drag.as_ref() else {
            return update;
        };
        if event.id != drag.pointer
            || (event.phase == PointerPhase::Released
                && event.button != Some(PointerButton::Primary))
        {
            return update;
        }
        if !matches!(
            event.phase,
            PointerPhase::Moved | PointerPhase::Released | PointerPhase::Cancelled
        ) {
            return update;
        }
        let at = coordinate(event, drag.target.config.axis);
        if event.phase == PointerPhase::Cancelled
            || (event.phase == PointerPhase::Released && !at.is_finite())
        {
            let drag = self.resize.drag.take().expect("checked active drag");
            update.layout_changed = self.restore_resize_drag(drag);
            return update;
        }
        if !at.is_finite() {
            return update;
        }
        let direction = if drag.target.config.trailing {
            -1.0
        } else {
            1.0
        };
        let value = (drag.initial + (at - drag.origin) * direction)
            .clamp(drag.target.config.minimum, drag.target.config.maximum);
        let handle = drag.handle;
        let initial = drag.initial;
        let previous = self
            .resize
            .overrides
            .get(&drag.target.node)
            .map_or(drag.initial, |value| value.value);
        if (value - previous).abs() >= 0.01 {
            let Some(element) = self.index.element(drag.target.node) else {
                self.resize.drag = None;
                return update;
            };
            self.resize.overrides.insert(
                drag.target.node,
                Override {
                    axis: drag.target.config.axis,
                    authored: dimension(&element.style, drag.target.config.axis),
                    value,
                },
            );
            self.layout_dirty = true;
            update.layout_changed = true;
        }
        if event.phase == PointerPhase::Released {
            self.resize.drag = None;
            if (value - initial).abs() >= 0.01 {
                update
                    .events
                    .extend(self.event_deliveries(handle, UiEventKind::ResizeCommitted { value }));
            }
        }
        update
    }

    /// Restores cached target sizing after cancellation or a producer-side interruption.
    fn restore_resize_drag(&mut self, drag: Drag) -> bool {
        if self.resize.overrides.remove(&drag.target.node).is_none() {
            return false;
        }
        if let Some(previous) = drag.previous {
            self.resize.overrides.insert(drag.target.node, previous);
        }
        self.resize.restored.insert(drag.target.node);
        self.layout_dirty = true;
        true
    }
}

/// Selects the authored dimension of `style` controlled by `axis`.
fn dimension(style: &crate::LayoutStyle, axis: ResizeAxis) -> Dimension {
    match axis {
        ResizeAxis::Horizontal => style.size.width,
        ResizeAxis::Vertical => style.size.height,
    }
}

/// Selects the absolute logical pointer coordinate for `axis` from `event`.
fn coordinate(event: PointerEvent, axis: ResizeAxis) -> f32 {
    match axis {
        ResizeAxis::Horizontal => event.position.x,
        ResizeAxis::Vertical => event.position.y,
    }
}
