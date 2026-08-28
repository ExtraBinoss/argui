use std::sync::Arc;

use argui_core::Point;
use argui_platform::{PlatformError, PlatformEvent, PointerButton};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::{
    RuntimeEvent,
    event::UserEvent,
    translate::{
        button_state, ime_input, key_input, modifiers_state, pointer_button, scroll_delta,
    },
};

use super::Application;

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = self.identity.as_ref().map_or_else(
            || self.window_config.clone().into_attributes(),
            |identity| {
                self.window_config
                    .clone()
                    .into_attributes_with_identity(identity)
            },
        );
        match event_loop.create_window(attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                let size = window.inner_size();
                self.scale_factor = window.scale_factor() as f32;
                self.update_viewport(size.width, size.height);
                if !self.prepare_or_exit(event_loop) {
                    return;
                }
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Opened {
                    width: size.width,
                    height: size.height,
                    scale_factor: window.scale_factor(),
                }));
                self.initialize_renderer(&window, event_loop);
                self.window = Some(window);
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::WindowCreationFailed(
                    error.to_string(),
                )));
                self.fatal_error = Some(PlatformError::from(error).into());
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Suspended));
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = event_loop;
            match event {
                #[cfg(feature = "tray")]
                UserEvent::Tray(_) => {}
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = self.window.as_ref().map(Arc::clone) else {
                return;
            };
            match event {
                UserEvent::ClipboardText {
                    window: window_key,
                    text,
                } => {
                    if window_key != self.window_key {
                        return;
                    }
                    if let Some(ui) = &mut self.ui_tree {
                        let update = ui.paste_text(&text);
                        self.apply_ui_update(update, &window, event_loop);
                    }
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        if window.id() != window_id {
            return;
        }
        let platform_event = match event {
            WindowEvent::CloseRequested => PlatformEvent::CloseRequested,
            WindowEvent::Resized(size) => {
                self.pending_window_frame.resize(size.width, size.height);
                PlatformEvent::Resized {
                    width: size.width,
                    height: size.height,
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = window.inner_size();
                self.pending_window_frame.scale_factor(
                    scale_factor as f32,
                    size.width,
                    size.height,
                );
                PlatformEvent::ScaleFactorChanged(scale_factor)
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = Point::new(
                    position.x as f32 / self.scale_factor,
                    position.y as f32 / self.scale_factor,
                );
                self.pointer_moved(point, &window, event_loop);
                PlatformEvent::PointerMoved {
                    x: point.x,
                    y: point.y,
                }
            }
            WindowEvent::CursorEntered { .. } => PlatformEvent::PointerEntered,
            WindowEvent::CursorLeft { .. } => {
                self.pointer_left(&window, event_loop);
                PlatformEvent::PointerLeft
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let state = button_state(state);
                let button = pointer_button(button);
                if button == PointerButton::Primary {
                    self.primary_button(state, &window, event_loop);
                }
                PlatformEvent::PointerButton { button, state }
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let delta = scroll_delta(delta, self.scale_factor);
                self.queue_pointer_scroll(delta, phase, &window, event_loop);
                PlatformEvent::PointerScrolled(delta)
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers_state(modifiers.state());
                return;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let input = key_input(event, self.modifiers);
                self.keyboard_input(&input, &window, event_loop);
                PlatformEvent::Keyboard(input)
            }
            WindowEvent::Ime(ime) => {
                let input = ime_input(ime);
                self.ime_input(input.clone(), &window, event_loop);
                PlatformEvent::Ime(input)
            }
            WindowEvent::Focused(focused) => {
                self.window_focus(focused, &window, event_loop);
                PlatformEvent::Focused(focused)
            }
            WindowEvent::RedrawRequested => {
                self.begin_frame_profile();
                self.advance_pointer_inertia(&window);
                self.flush_pointer_scroll(&window, event_loop);
                self.flush_scrollbar_drag(&window, event_loop);
                self.animate(&window, event_loop);
                self.flush_window_frame();
                self.flush_ui_frame(event_loop);
                self.refresh_cursor(&window);
                self.render(event_loop);
                PlatformEvent::RedrawRequested
            }
            _ => return,
        };
        if platform_event.requires_redraw() {
            window.request_redraw();
        }
        if self.exit_on_close && platform_event.closes_window() {
            event_loop.exit();
        }
        (self.on_event)(RuntimeEvent::Platform(platform_event));
    }
}
