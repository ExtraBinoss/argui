use argui_core::{Point, ScrollDelta};

use crate::{InteractionUpdate, NodeId, ScrollRegion, UiEvent, UiEventKind};

use super::UiTree;

impl UiTree {
    /// Requests a presenter update when native scrolling selects another item chunk.
    ///
    /// * `node` — virtual scroll container.
    /// * `viewport_extent` — measured width or height along its active axis.
    ///
    /// Returns listener deliveries only when the selected range or viewport
    /// differs from both the mounted range and the most recently requested one.
    pub fn request_virtual_window(&mut self, node: NodeId, viewport_extent: f32) -> Vec<UiEvent> {
        let Some(viewport) = self
            .element_for(node)
            .and_then(|element| element.virtual_viewport())
            .cloned()
        else {
            return Vec::new();
        };
        let viewport_extent = viewport_extent.max(0.0);
        self.virtual_viewport_extents.insert(node, viewport_extent);
        let offset = self.scroll_offset(node);
        let offset = if viewport.list.is_horizontal() {
            offset.x
        } else {
            offset.y
        };
        let configured_extent = viewport.list.viewport_extent();
        let range = viewport
            .list
            .with_viewport(viewport_extent)
            .window(offset)
            .range;
        if range == viewport.mounted && (viewport_extent - configured_extent).abs() <= 0.5 {
            self.virtual_requests.remove(&node);
            return Vec::new();
        }
        if self
            .virtual_requests
            .get(&node)
            .is_some_and(|(requested, extent)| {
                *requested == range && (*extent - viewport_extent).abs() <= 0.5
            })
        {
            return Vec::new();
        }
        self.virtual_requests
            .insert(node, (range.clone(), viewport_extent));
        self.event_deliveries(
            node,
            UiEventKind::VirtualWindowChanged {
                start: range.start,
                end: range.end,
                offset,
                viewport_extent,
            },
        )
    }

    /// Applies newly declared offsets while preserving user scrolling between declarations.
    ///
    /// Returns whether any retained scroll position changed.
    pub(super) fn sync_declared_scroll_offsets(&mut self) -> bool {
        let declarations = crate::traversal::flattened(&self.root)
            .into_iter()
            .zip(self.node_ids.iter().copied())
            .filter_map(|(element, node)| {
                element.declared_scroll_offset.map(|offset| (node, offset))
            })
            .collect::<Vec<_>>();
        let mut changed = false;
        let mut next = std::collections::HashMap::with_capacity(declarations.len());
        for (node, offset) in declarations {
            if self.declared_scroll_offsets.get(&node) != Some(&offset) {
                changed |= self.set_scroll_offset(node, offset);
            }
            next.insert(node, offset);
        }
        self.declared_scroll_offsets = next;
        changed
    }

    /// Default keyboard scrolling for a focused viewport. Editors and collection controls
    /// retain their own arrow-key contracts. Hosts call this after cancelable key delivery.
    ///
    /// * `input` — pressed key and modifiers.
    /// * `regions` — current scroll regions and their viewport geometry.
    ///
    /// Returns the resulting interaction update; unrelated keys or unchanged offsets
    /// produce an empty update.
    pub fn scroll_keyboard(
        &mut self,
        input: &argui_core::KeyInput,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        use argui_core::{Key, KeyState};
        if input.state != KeyState::Pressed || input.modifiers.command() || input.modifiers.alt {
            return InteractionUpdate::default();
        }
        let Some(node) = self.focused_node() else {
            return InteractionUpdate::default();
        };
        let Some(element) = self.element_for(node) else {
            return InteractionUpdate::default();
        };
        if !self.input_available(node)
            || matches!(element.kind, crate::ElementKind::TextEditor { .. })
            || element.semantics.as_ref().is_some_and(|semantics| {
                !matches!(
                    semantics.role,
                    crate::Role::Generic
                        | crate::Role::Group
                        | crate::Role::Dialog
                        | crate::Role::AlertDialog
                )
            })
        {
            return InteractionUpdate::default();
        }
        let Some(region) = regions
            .iter()
            .find(|region| region.node == node && region.config.enabled)
        else {
            return InteractionUpdate::default();
        };
        let previous = self.scroll_offset(node);
        let mut offset = previous;
        match &input.key {
            Key::ArrowUp => offset.y -= 40.0,
            Key::ArrowDown => offset.y += 40.0,
            Key::ArrowLeft => offset.x -= 40.0,
            Key::ArrowRight => offset.x += 40.0,
            Key::PageUp => offset.y -= region.bounds.size.height * 0.9,
            Key::PageDown => offset.y += region.bounds.size.height * 0.9,
            Key::Character(value) if value == " " => {
                offset.y +=
                    region.bounds.size.height * if input.modifiers.shift { -0.9 } else { 0.9 }
            }
            Key::Home => offset = Point::default(),
            Key::End => offset = region.max_offset,
            _ => return InteractionUpdate::default(),
        }
        offset.x = offset.x.clamp(0.0, region.max_offset.x.max(0.0));
        offset.y = offset.y.clamp(0.0, region.max_offset.y.max(0.0));
        if !self.set_scroll_offset(node, offset) {
            return InteractionUpdate::default();
        }
        self.activate_scrollbar(node, regions);
        self.scroll_update(crate::scroll::ScrollChange {
            node,
            delta: Point::new(offset.x - previous.x, offset.y - previous.y),
            offset,
        })
    }

    /// Dispatches a wheel event to the topmost enabled scroll region at `point`.
    ///
    /// * `point` — pointer position in window coordinates.
    /// * `delta` — wheel or trackpad displacement.
    /// * `regions` — current scroll regions used for hit testing.
    pub fn wheel_event(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        let target = regions
            .iter()
            .rev()
            .find(|region| region.config.enabled && region.contains(point))
            .map(|region| region.node)
            .or_else(|| self.node_ids().first().copied());
        target.map_or_else(InteractionUpdate::default, |target| {
            self.wheel_event_from(target, point, delta)
        })
    }

    /// Dispatch a wheel event to the viewport retained by the current gesture.
    ///
    /// * `target` — latched viewport receiving the wheel event.
    /// * `point` — pointer position in window coordinates.
    /// * `delta` — wheel or trackpad displacement.
    pub fn wheel_event_from(
        &mut self,
        target: NodeId,
        point: Point,
        delta: ScrollDelta,
    ) -> InteractionUpdate {
        InteractionUpdate {
            events: self.event_deliveries(
                target,
                UiEventKind::Wheel {
                    delta,
                    position: point,
                },
            ),
            ..InteractionUpdate::default()
        }
    }

    /// Returns the resolved visual scroll offset of a node.
    ///
    /// * `node` — scroll container node.
    #[must_use]
    pub fn scroll_offset(&self, node: NodeId) -> Point {
        let mut base = self.scroll.visual_offset(node);
        super::transition::apply_scroll(&self.transitions, node, &mut base);
        self.element_for(node).map_or(base, |element| {
            crate::binding::resolved_scroll(&element.bindings, base)
        })
    }

    /// Sets a scroll container's logical offset.
    ///
    /// * `node` — scroll container node.
    /// * `offset` — requested horizontal and vertical offsets.
    ///
    /// Returns whether the stored offset changed.
    pub fn set_scroll_offset(&mut self, node: NodeId, offset: Point) -> bool {
        let changed = self.scroll.set_offset(node, offset);
        if changed {
            self.sync_scroll_motion(node, offset);
        }
        changed
    }

    /// Marks a node's scrollbar as active for its configured visibility policy.
    ///
    /// * `node` — scroll container node.
    /// * `regions` — current scroll regions containing scrollbar configuration.
    pub fn activate_scrollbar(&mut self, node: NodeId, regions: &[ScrollRegion]) {
        self.scroll.activate_scrollbar(node, regions);
    }

    /// Advances overscroll and scrollbar animations by elapsed seconds.
    ///
    /// * `elapsed` — nonnegative time since the previous physics update, in seconds.
    /// * `regions` — current scroll regions for scrollbar animation.
    ///
    /// Returns the interaction update produced by animated changes.
    pub fn advance_scroll_physics(
        &mut self,
        elapsed: f32,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        let changes = self.scroll.advance_overscroll(elapsed);
        let mut update = InteractionUpdate::default();
        for change in changes {
            update.merge(self.scroll_update(change));
        }
        if self.scroll.advance_scrollbars(elapsed, regions) {
            update.paint_changed = true;
        }
        update
    }

    /// Returns whether scrolling physics or scrollbar activity needs another frame.
    #[must_use]
    pub fn wants_scroll_frame(&self) -> bool {
        self.scroll.overscroll_active() || self.scroll.scrollbar_activity_active()
    }

    /// Routes a scroll delta from the pointer through eligible scroll ancestors.
    ///
    /// * `point` — pointer position in window coordinates.
    /// * `delta` — wheel or trackpad displacement.
    /// * `regions` — current scroll regions and geometry.
    pub fn scroll(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        self.scroll_routed(point, delta, regions, None)
    }

    /// Scroll a latched viewport and its ancestors, never descendants newly under the pointer.
    ///
    /// * `target` — viewport retained when the scroll gesture began.
    /// * `point` — current pointer position in window coordinates.
    /// * `delta` — wheel or trackpad displacement.
    /// * `regions` — current scroll regions and geometry.
    pub fn scroll_from(
        &mut self,
        target: NodeId,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        if !regions
            .iter()
            .any(|region| region.node == target && region.config.enabled)
        {
            return InteractionUpdate::default();
        }
        let mut targets = vec![target];
        let mut ancestor = self.parent_of(target);
        while let Some(node) = ancestor {
            targets.push(node);
            ancestor = self.parent_of(node);
        }
        self.scroll_routed(point, delta, regions, Some(&targets))
    }

    fn scroll_routed(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
        targets: Option<&[NodeId]>,
    ) -> InteractionUpdate {
        let Some(outcome) = self.scroll.scroll(point, delta, regions, targets) else {
            return InteractionUpdate::default();
        };
        match outcome {
            crate::scroll::ScrollOutcome::Changed(changes) => {
                let mut update = InteractionUpdate::default();
                for change in changes {
                    self.sync_scroll_motion(change.node, change.offset);
                    update.merge(self.scroll_update(change));
                }
                update
            }
            crate::scroll::ScrollOutcome::Consumed => InteractionUpdate::default(),
        }
    }

    /// Begins scrollbar interaction at a pointer position.
    ///
    /// * `point` — pointer position in window coordinates.
    /// * `regions` — current scroll regions and scrollbar geometry.
    ///
    /// Returns `None` when no scrollbar handles the press.
    pub fn scrollbar_pressed(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<InteractionUpdate> {
        let change = self.scroll.scrollbar_pressed(point, regions)?;
        let mut update = change.map_or_else(InteractionUpdate::default, |change| {
            self.sync_scroll_motion(change.node, change.offset);
            self.scroll_update(change)
        });
        update.merge(transition_update(self.sync_transitions()));
        update.paint_changed = true;
        Some(update)
    }

    /// Updates the active scrollbar drag.
    ///
    /// * `point` — current pointer position in window coordinates.
    /// * `regions` — current scroll regions and scrollbar geometry.
    ///
    /// Returns `None` when no scrollbar drag is active.
    pub fn scrollbar_dragged(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<InteractionUpdate> {
        if !self.scroll.dragging() {
            return None;
        }
        let change = self.scroll.scrollbar_dragged(point, regions);
        Some(change.map_or_else(InteractionUpdate::default, |change| {
            self.sync_scroll_motion(change.node, change.offset);
            self.scroll_update(change)
        }))
    }

    /// Ends the active scrollbar drag, if any.
    ///
    /// Returns an update when a drag was released.
    pub fn scrollbar_released(&mut self) -> Option<InteractionUpdate> {
        self.scroll.scrollbar_released().then(|| {
            let mut update = transition_update(self.sync_transitions());
            update.paint_changed = true;
            update
        })
    }

    /// Updates scrollbar hover state at the given pointer position.
    ///
    /// * `point` — pointer position, or `None` when outside the window.
    /// * `regions` — current scroll regions and scrollbar geometry.
    pub fn scrollbar_pointer_moved(
        &mut self,
        point: Option<Point>,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        if !self.scroll.update_hover(point, regions) {
            return InteractionUpdate::default();
        }
        let mut update = transition_update(self.sync_transitions());
        update.paint_changed = true;
        update
    }

    /// Returns whether a scrollbar drag is active.
    #[must_use]
    pub fn scrollbar_dragging(&self) -> bool {
        self.scroll.dragging()
    }

    fn scroll_update(&mut self, change: crate::scroll::ScrollChange) -> InteractionUpdate {
        let text_input_changed = self.text_inputs.get(change.node).is_some();
        let mut events = self.event_deliveries(
            change.node,
            UiEventKind::Scrolled {
                delta: change.delta,
                offset: change.offset,
            },
        );
        if let Some(extent) = self.virtual_viewport_extents.get(&change.node).copied() {
            events.extend(self.request_virtual_window(change.node, extent));
        }
        InteractionUpdate {
            events,
            paint_changed: true,
            scroll_changed: true,
            text_input_changed,
            ..InteractionUpdate::default()
        }
    }

    fn sync_scroll_motion(&mut self, node: NodeId, offset: Point) {
        self.transitions.set_scroll(node, offset);
        let Some(element) = self.element_for(node) else {
            return;
        };
        for binding in &element.bindings {
            if let crate::PropertyBinding::Scroll(binding) = binding {
                binding.motion.set(offset);
            }
        }
    }
}

fn transition_update(update: super::TreeUpdate) -> InteractionUpdate {
    InteractionUpdate {
        composite_changed: update == super::TreeUpdate::Composite,
        paint_changed: update == super::TreeUpdate::Paint,
        scroll_changed: update == super::TreeUpdate::Scroll,
        layout_changed: update == super::TreeUpdate::Layout,
        ..InteractionUpdate::default()
    }
}
