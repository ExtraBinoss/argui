use web_time::Instant;
use winit::event_loop::ActiveEventLoop;

use argui_inspect::Invalidation;
use argui_ui::{FocusRequest, InteractionUpdate, TextSelectionRequest, TreeUpdate};

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
    requested: bool,
    rebuild: bool,
    layout: bool,
    text_input: bool,
    scroll: bool,
    paint: bool,
    scroll_request: Option<ScrollRequest>,
    focus_request: Option<FocusRequest>,
    text_selection_request: Option<TextSelectionRequest>,
}

impl PendingUiFrame {
    pub(crate) fn merge(&mut self, update: &InteractionUpdate, rebuild: bool) {
        self.requested |= rebuild
            | update.layout_changed
            | update.text_input_changed
            | update.scroll_changed
            | update.paint_changed;
        self.rebuild |= rebuild;
        self.layout |= update.layout_changed;
        self.text_input |= update.text_input_changed;
        self.scroll |= update.scroll_changed;
        self.paint |= update.paint_changed;
    }

    pub(super) fn request_scroll(&mut self, request: Option<ScrollRequest>) {
        self.requested |= request.is_some();
        if request.is_some() {
            self.scroll_request = request;
        }
    }

    pub(super) fn request_focus(&mut self, request: Option<FocusRequest>) {
        self.requested |= request.is_some();
        if request.is_some() {
            self.focus_request = request;
        }
    }

    pub(super) fn request_text_selection(&mut self, request: Option<TextSelectionRequest>) {
        self.requested |= request.is_some();
        if request.is_some() {
            self.text_selection_request = request;
        }
    }

    pub(super) fn request_layout(&mut self) {
        self.requested = true;
        self.layout = true;
    }

    pub(super) fn request_scroll_update(&mut self) {
        self.requested = true;
        self.scroll = true;
    }

    pub(super) fn request_paint(&mut self) {
        self.requested = true;
        self.paint = true;
    }

    pub(super) fn request_rebuild(&mut self) {
        self.requested = true;
        self.rebuild = true;
    }

    pub(super) fn needs_frame(&self) -> bool {
        self.requested
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
            TreeUpdate::Scroll => {
                let started = Instant::now();
                self.scroll_or_exit(event_loop);
                self.frame_record.paint += started.elapsed();
            }
            TreeUpdate::Semantics => {}
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
            TreeUpdate::Scroll => Invalidation::Paint,
            TreeUpdate::Semantics => Invalidation::None,
            TreeUpdate::None if pending.layout => Invalidation::Layout,
            TreeUpdate::None if pending.text_input || pending.scroll || pending.paint => {
                Invalidation::Paint
            }
            TreeUpdate::None => Invalidation::None,
        };
        if let Some(request) = pending.scroll_request {
            self.apply_scroll_request(request, event_loop);
        }
        let focus_update = match (&mut self.ui_tree, &self.ui_layout) {
            (Some(ui), Some(layout)) => ui.sync_focus(&layout.hit_regions, pending.focus_request),
            _ => InteractionUpdate::default(),
        };
        if (!focus_update.events.is_empty() || focus_update.paint_changed)
            && let Some(window) = self.window.clone()
        {
            self.apply_ui_update(focus_update, &window, event_loop);
            self.update_ime(&window);
        }
        if let (Some(ui), Some(request)) = (&mut self.ui_tree, pending.text_selection_request) {
            let selection_update = ui.select_text(request);
            if (!selection_update.events.is_empty() || selection_update.paint_changed)
                && let Some(window) = self.window.clone()
            {
                self.apply_ui_update(selection_update, &window, event_loop);
                self.update_ime(&window);
            }
        }
        if tree_update != TreeUpdate::None {
            (self.on_event)(RuntimeEvent::ViewUpdated(tree_update));
        }
        self.sync_accessibility();
        tree_update
    }
}
