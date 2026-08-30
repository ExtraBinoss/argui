use argui_core::{Point, ScrollDelta};

use crate::{InteractionUpdate, NodeId, ScrollRegion, UiEvent, UiEventKind};

use super::UiTree;

impl UiTree {
    #[must_use]
    pub fn scroll_offset(&self, node: NodeId) -> Point {
        let mut base = self.scroll.offset(node);
        super::transition::apply_scroll(&self.transitions, node, &mut base);
        self.element_for(node).map_or(base, |element| {
            crate::binding::resolved_scroll(&element.bindings, base)
        })
    }

    pub fn set_scroll_offset(&mut self, node: NodeId, offset: Point) -> bool {
        let changed = self.scroll.set_offset(node, offset);
        if changed {
            self.sync_scroll_motion(node, offset);
        }
        changed
    }

    pub fn scroll(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        let Some(outcome) = self.scroll.scroll(point, delta, regions) else {
            return InteractionUpdate::default();
        };
        match outcome {
            crate::scroll::ScrollOutcome::Changed(change) => {
                self.sync_scroll_motion(change.node, change.offset);
                self.scroll_update(change)
            }
            crate::scroll::ScrollOutcome::Consumed => InteractionUpdate::default(),
        }
    }

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
        Some(update)
    }

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

    pub fn scrollbar_released(&mut self) -> Option<InteractionUpdate> {
        self.scroll
            .scrollbar_released()
            .then(|| transition_update(self.sync_transitions()))
    }

    pub fn scrollbar_pointer_moved(
        &mut self,
        point: Option<Point>,
        regions: &[ScrollRegion],
    ) -> InteractionUpdate {
        if !self.scroll.update_hover(point, regions) {
            return InteractionUpdate::default();
        }
        transition_update(self.sync_transitions())
    }

    #[must_use]
    pub fn scrollbar_dragging(&self) -> bool {
        self.scroll.dragging()
    }

    fn scroll_update(&self, change: crate::scroll::ScrollChange) -> InteractionUpdate {
        InteractionUpdate {
            events: vec![UiEvent {
                target: change.node,
                key: self.key_for(change.node).map(ToOwned::to_owned),
                kind: UiEventKind::Scrolled {
                    delta: change.delta,
                    offset: change.offset,
                },
            }],
            paint_changed: true,
            scroll_changed: true,
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
        paint_changed: update == super::TreeUpdate::Paint,
        scroll_changed: update == super::TreeUpdate::Scroll,
        layout_changed: update == super::TreeUpdate::Layout,
        ..InteractionUpdate::default()
    }
}
