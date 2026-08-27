use std::time::Instant;
use winit::event_loop::ActiveEventLoop;

use argui_inspect::Invalidation;
use argui_ui::{InteractionUpdate, TreeUpdate};

use crate::{RuntimeEvent, ScrollRequest, app::Application};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PendingWindowFrame {
    size: Option<(u32, u32)>,
    scale_factor: Option<f32>,
    events: u32,
}

impl PendingWindowFrame {
    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.size = Some((width, height));
        self.events = self.events.saturating_add(1);
    }

    pub(super) fn scale_factor(&mut self, scale_factor: f32, width: u32, height: u32) {
        self.scale_factor = Some(scale_factor);
        self.resize(width, height);
    }

    pub(super) fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    pub(super) const fn size(self) -> Option<(u32, u32)> {
        self.size
    }

    pub(super) const fn scale(self) -> Option<f32> {
        self.scale_factor
    }

    pub(super) const fn events(self) -> u32 {
        self.events
    }
}

#[derive(Default)]
pub(crate) struct PendingUiFrame {
    rebuild: bool,
    layout: bool,
    text_input: bool,
    scroll: bool,
    paint: bool,
    scroll_request: Option<ScrollRequest>,
}

impl PendingUiFrame {
    pub(crate) fn merge(&mut self, update: &InteractionUpdate, rebuild: bool) {
        self.rebuild |= rebuild;
        self.layout |= update.layout_changed;
        self.text_input |= update.text_input_changed;
        self.scroll |= update.scroll_changed;
        self.paint |= update.paint_changed;
    }

    pub(super) fn request_scroll(&mut self, request: Option<ScrollRequest>) {
        if request.is_some() {
            self.scroll_request = request;
        }
    }

    pub(super) fn request_layout(&mut self) {
        self.layout = true;
    }

    pub(super) fn needs_frame(&self) -> bool {
        self.rebuild
            || self.layout
            || self.text_input
            || self.scroll
            || self.paint
            || self.scroll_request.is_some()
    }
}

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn begin_frame_profile(&mut self) {
        let now = Instant::now();
        self.frame_record = argui_inspect::FrameRecord {
            interval: self
                .last_redraw
                .map_or(std::time::Duration::ZERO, |previous| {
                    now.duration_since(previous)
                }),
            ..argui_inspect::FrameRecord::default()
        };
        self.last_redraw = Some(now);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn flush_window_frame(&mut self) {
        let pending = self.pending_window_frame.take();
        let scale_changed = pending.scale().is_some_and(|scale_factor| {
            let changed = self.scale_factor != scale_factor;
            self.scale_factor = scale_factor;
            changed
        });
        let Some((width, height)) = pending.size() else {
            return;
        };
        let started = Instant::now();
        if let super::RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
            renderer.resize(width, height);
        }
        let previous_viewport = self.viewport;
        self.update_viewport(width, height);
        if scale_changed || self.viewport != previous_viewport {
            self.pending_ui_frame.request_layout();
        }
        self.frame_record.surface += started.elapsed();
        self.frame_record.resize_events = pending.events();
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn flush_ui_frame(&mut self, event_loop: &ActiveEventLoop) -> TreeUpdate {
        let pending = std::mem::take(&mut self.pending_ui_frame);
        let tree_started = Instant::now();
        let tree_update = if pending.rebuild {
            let root = self.inspected_view();
            match (root, &mut self.ui_tree) {
                (Some(root), Some(tree)) => tree.update(root),
                _ => TreeUpdate::None,
            }
        } else {
            TreeUpdate::None
        };
        self.frame_record.tree += tree_started.elapsed();
        match tree_update {
            TreeUpdate::Layout => {
                let started = Instant::now();
                self.prepare_or_exit(event_loop);
                self.frame_record.layout += started.elapsed();
            }
            _ if pending.layout => {
                let started = Instant::now();
                self.prepare_or_exit(event_loop);
                self.frame_record.layout += started.elapsed();
            }
            _ if pending.text_input => {
                let started = Instant::now();
                self.refresh_text_inputs();
                self.frame_record.paint += started.elapsed();
            }
            _ if pending.scroll => {
                let started = Instant::now();
                self.scroll_or_exit(event_loop);
                self.frame_record.paint += started.elapsed();
            }
            TreeUpdate::Paint => {
                let started = Instant::now();
                self.repaint();
                self.frame_record.paint += started.elapsed();
            }
            TreeUpdate::None if pending.paint => {
                let started = Instant::now();
                self.repaint();
                self.frame_record.paint += started.elapsed();
            }
            TreeUpdate::None => {}
        }
        self.frame_record.update = match tree_update {
            TreeUpdate::Layout => Invalidation::Layout,
            TreeUpdate::Paint => Invalidation::Paint,
            TreeUpdate::None if pending.layout => Invalidation::Layout,
            TreeUpdate::None if pending.text_input || pending.scroll || pending.paint => {
                Invalidation::Paint
            }
            TreeUpdate::None => Invalidation::None,
        };
        if let Some(request) = pending.scroll_request {
            self.apply_scroll_request(&request.key, request.offset, event_loop);
        }
        if tree_update != TreeUpdate::None {
            (self.on_event)(RuntimeEvent::ViewUpdated(tree_update));
        }
        tree_update
    }
}

#[cfg(test)]
mod tests {
    use argui_core::Point;
    use argui_ui::InteractionUpdate;

    use crate::ScrollRequest;

    use super::{PendingUiFrame, PendingWindowFrame};

    #[test]
    fn window_changes_coalesce_to_the_latest_frame_values() {
        let mut pending = PendingWindowFrame::default();
        pending.resize(800, 600);
        pending.resize(1_200, 900);
        pending.scale_factor(2.0, 1_600, 1_200);

        let frame = pending.take();
        assert_eq!(frame.size(), Some((1_600, 1_200)));
        assert_eq!(frame.scale(), Some(2.0));
        assert_eq!(frame.events(), 3);
        assert_eq!(pending.take(), PendingWindowFrame::default());
    }

    #[test]
    fn ui_work_coalesces_without_dropping_any_invalidation_class() {
        let mut pending = PendingUiFrame::default();
        assert!(!pending.needs_frame());
        pending.request_scroll(None);
        assert!(!pending.needs_frame());

        let updates = [
            InteractionUpdate {
                layout_changed: true,
                ..InteractionUpdate::default()
            },
            InteractionUpdate {
                text_input_changed: true,
                ..InteractionUpdate::default()
            },
            InteractionUpdate {
                scroll_changed: true,
                ..InteractionUpdate::default()
            },
            InteractionUpdate {
                paint_changed: true,
                ..InteractionUpdate::default()
            },
        ];
        for update in updates {
            let mut isolated = PendingUiFrame::default();
            isolated.merge(&update, false);
            assert!(isolated.needs_frame());
            pending.merge(&update, false);
        }
        let mut rebuild = PendingUiFrame::default();
        rebuild.merge(&InteractionUpdate::default(), true);
        assert!(rebuild.needs_frame());

        pending.request_scroll(Some(ScrollRequest {
            key: "tree".into(),
            offset: Point::new(0.0, 80.0),
        }));
        pending.request_layout();
        assert!(pending.needs_frame());
    }
}
