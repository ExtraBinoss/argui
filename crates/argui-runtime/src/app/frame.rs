use winit::event_loop::ActiveEventLoop;

use argui_ui::{InteractionUpdate, TreeUpdate};

use crate::{RuntimeEvent, ScrollRequest, app::Application};

#[derive(Default)]
pub(super) struct PendingUiFrame {
    rebuild: bool,
    layout: bool,
    text_input: bool,
    scroll: bool,
    paint: bool,
    scroll_request: Option<ScrollRequest>,
}

impl PendingUiFrame {
    pub(super) fn merge(&mut self, update: &InteractionUpdate, rebuild: bool) {
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
    pub(super) fn flush_ui_frame(&mut self, event_loop: &ActiveEventLoop) -> TreeUpdate {
        let pending = std::mem::take(&mut self.pending_ui_frame);
        let tree_update = if pending.rebuild {
            let root = self.inspected_view();
            match (root, &mut self.ui_tree) {
                (Some(root), Some(tree)) => tree.update(root),
                _ => TreeUpdate::None,
            }
        } else {
            TreeUpdate::None
        };
        match tree_update {
            TreeUpdate::Layout => {
                self.prepare_or_exit(event_loop);
            }
            _ if pending.layout => {
                self.prepare_or_exit(event_loop);
            }
            _ if pending.text_input => {
                self.refresh_text_inputs();
            }
            _ if pending.scroll => {
                self.scroll_or_exit(event_loop);
            }
            TreeUpdate::Paint => self.repaint(),
            TreeUpdate::None if pending.paint => self.repaint(),
            TreeUpdate::None => {}
        }
        if let Some(request) = pending.scroll_request {
            self.apply_scroll_request(&request.key, request.offset, event_loop);
        }
        if tree_update != TreeUpdate::None {
            (self.on_event)(RuntimeEvent::ViewUpdated(tree_update));
        }
        tree_update
    }
}
