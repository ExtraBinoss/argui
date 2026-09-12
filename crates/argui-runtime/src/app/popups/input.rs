use crate::app::Application;
use argui_core::{Point, Size};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(crate) fn popup_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        mut event: WindowEvent,
    ) {
        let Some(popup) = self
            .popups
            .entries
            .iter()
            .find(|popup| popup.native.window().id() == id)
        else {
            return;
        };
        let origin = popup.bounds.origin;
        let node = popup.node;
        let native = popup.native.window().clone();
        match &mut event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                self.dismiss_popup(node, event_loop);
                return;
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return;
            }
            WindowEvent::Resized(size) => {
                let popup = self
                    .popups
                    .entries
                    .iter_mut()
                    .find(|popup| popup.node == node)
                    .unwrap();
                let logical = Size::new(
                    size.width as f32 / self.scale_factor,
                    size.height as f32 / self.scale_factor,
                );
                popup.renderer.resize(size.width, size.height);
                if logical != popup.bounds.size && logical.width > 0.0 && logical.height > 0.0 {
                    popup.bounds.size = logical;
                    if let Some(ui) = &mut self.ui_tree {
                        ui.set_native_portal(node, Some(popup.bounds));
                    }
                    self.pending_ui_frame.request_layout();
                }
            }
            WindowEvent::Focused(focused) => {
                self.popups.pending_blur = !*focused;
                if *focused && self.popups.suspended {
                    self.popups.suspended = false;
                    if let Some(window) = self.window.clone() {
                        self.window_focus(true, &window, event_loop);
                    }
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
                return;
            }
            WindowEvent::CursorMoved { position, .. } => {
                position.x += f64::from(origin.x * self.scale_factor);
                position.y += f64::from(origin.y * self.scale_factor);
            }
            WindowEvent::Touch(touch) => {
                touch.location.x += f64::from(origin.x * self.scale_factor);
                touch.location.y += f64::from(origin.y * self.scale_factor);
            }
            WindowEvent::Moved(_)
            | WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::Occluded(_) => return,
            _ => {}
        }
        if matches!(event, WindowEvent::Resized(_)) {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
            return;
        }
        if let Some(crate::host::HostId::Winit(parent)) = self.window_id() {
            self.window_event(event_loop, parent, event);
            // Cursor state is shared by the logical tree; install it on the hovered OS surface too.
            native.set_cursor(super::super::cursor::to_winit(self.last_cursor));
        }
    }

    pub(crate) fn popup_ime(&self, enabled: bool, caret: Option<argui_core::Rect>) -> bool {
        let focused = self.ui_tree.as_ref().and_then(|ui| {
            ui.focused_node()
                .and_then(|node| ui.native_portal_owner(node))
        });
        for popup in &self.popups.entries {
            let active = Some(popup.node) == focused;
            popup.native.window().set_ime_allowed(enabled && active);
            if enabled
                && active
                && let Some(caret) = caret
            {
                let position = Point::new(
                    caret.origin.x - popup.bounds.origin.x,
                    caret.origin.y + caret.size.height - popup.bounds.origin.y,
                );
                // Convert with the tree's scale: the popup may straddle a differently scaled monitor.
                popup.native.window().set_ime_cursor_area(
                    winit::dpi::PhysicalPosition::new(
                        position.x * self.scale_factor,
                        position.y * self.scale_factor,
                    ),
                    winit::dpi::PhysicalSize::new(
                        self.scale_factor,
                        caret.size.height * self.scale_factor,
                    ),
                );
            }
        }
        focused.is_some()
    }
}
