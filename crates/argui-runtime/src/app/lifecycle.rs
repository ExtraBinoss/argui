use std::sync::Arc;

use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_platform::{PlatformError, PlatformEvent, PointerButton};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

use crate::{
    RuntimeEvent,
    event::UserEvent,
    translate::{
        button_state, ime_input, key_input, modifiers_state, pointer_button, pointer_button_mask,
        pointer_phase, scroll_delta,
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
        #[cfg(not(target_arch = "wasm32"))]
        let attributes = attributes.with_visible(false);
        match event_loop.create_window(attributes) {
            Ok(window) => {
                let window = Arc::new(window);
                let size = window.inner_size();
                self.scale_factor = window.scale_factor() as f32;
                if self.preference_overrides.color_scheme.is_none()
                    && let Some(theme) = window.theme()
                {
                    self.environment.color_scheme = match theme {
                        winit::window::Theme::Light => argui_core::ColorScheme::Light,
                        winit::window::Theme::Dark => argui_core::ColorScheme::Dark,
                    };
                }
                self.update_viewport(size.width, size.height);
                if !self.prepare_or_exit(event_loop) {
                    return;
                }
                #[cfg(not(target_arch = "wasm32"))]
                self.initialize_accessibility(event_loop, &window);
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Opened {
                    width: size.width,
                    height: size.height,
                    scale_factor: window.scale_factor(),
                    capabilities: argui_platform::window_capabilities(&window),
                }));
                self.initialize_renderer(&window, event_loop);
                window.set_visible(self.initial_visible);
                self.window = Some(window);
                self.initialize_preferences();
                #[cfg(target_arch = "wasm32")]
                if self.exit_on_close
                    && let Err(error) = self.initialize_web_accessibility()
                {
                    (self.on_event)(RuntimeEvent::CommandFailed(error));
                }
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
        if let UserEvent::Preferences {
            window,
            preferences,
        } = &event
        {
            if *window == self.window_key {
                self.apply_preferences(*preferences);
            }
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            match event {
                UserEvent::Preferences { .. } => {}
                UserEvent::AccessKit(event) => {
                    let Some(window) = self.window.as_ref().map(Arc::clone) else {
                        return;
                    };
                    self.accessibility_event(event, &window, event_loop);
                }
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
                UserEvent::Preferences { .. } => {}
                UserEvent::ClipboardText {
                    window: window_key,
                    target,
                    text,
                } => {
                    if window_key != self.window_key {
                        return;
                    }
                    if let Some(ui) = &mut self.ui_tree {
                        let update = ui.paste_text(target, &text);
                        self.apply_ui_update(update, &window, event_loop);
                    }
                }
                UserEvent::Accessibility {
                    window: window_key,
                    request,
                } => {
                    if window_key == self.window_key {
                        self.web_accessibility_action(request, &window, event_loop);
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
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(adapter) = &mut self.accessibility {
            adapter.process_event(&window, &event);
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
                PlatformEvent::Pointer(PointerEvent {
                    buttons: self.pointer_buttons,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Moved, point)
                })
            }
            WindowEvent::CursorEntered { .. } => PlatformEvent::Pointer(PointerEvent {
                buttons: self.pointer_buttons,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(PointerPhase::Entered, self.pointer.unwrap_or_default())
            }),
            WindowEvent::CursorLeft { .. } => {
                let point = self.pointer.unwrap_or_default();
                self.pointer_left(&window, event_loop);
                PlatformEvent::Pointer(PointerEvent {
                    buttons: self.pointer_buttons,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Left, point)
                })
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let state = button_state(state);
                let button = pointer_button(button);
                let mask = pointer_button_mask(button);
                match state {
                    argui_platform::ButtonState::Pressed => self.pointer_buttons |= mask,
                    argui_platform::ButtonState::Released => self.pointer_buttons &= !mask,
                }
                if button == PointerButton::Primary {
                    self.primary_button(state, &window, event_loop);
                } else if button == PointerButton::Secondary {
                    self.secondary_button(state, &window, event_loop);
                }
                PlatformEvent::Pointer(PointerEvent {
                    phase: if state == argui_platform::ButtonState::Pressed {
                        PointerPhase::Pressed
                    } else {
                        PointerPhase::Released
                    },
                    button: Some(button),
                    buttons: self.pointer_buttons,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Moved, self.pointer.unwrap_or_default())
                })
            }
            WindowEvent::Touch(touch) => {
                let point = Point::new(
                    touch.location.x as f32 / self.scale_factor,
                    touch.location.y as f32 / self.scale_factor,
                );
                let event = PointerEvent {
                    id: PointerId::new(touch.id.saturating_add(1)),
                    kind: PointerKind::Touch,
                    phase: pointer_phase(touch.phase),
                    position: point,
                    button: Some(PointerButton::Primary),
                    buttons: u16::from(!matches!(
                        touch.phase,
                        winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                    )),
                    pressure: touch.force.map(|force| force.normalized() as f32),
                    primary: false,
                    timestamp: self.input_epoch.elapsed(),
                };
                let event = self.touch_pointer(event, &window, event_loop);
                PlatformEvent::Pointer(event)
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
            WindowEvent::ThemeChanged(theme) => {
                let scheme = match theme {
                    winit::window::Theme::Light => argui_core::ColorScheme::Light,
                    winit::window::Theme::Dark => argui_core::ColorScheme::Dark,
                };
                if self.preference_overrides.color_scheme.is_none() {
                    self.apply_color_scheme(scheme);
                }
                PlatformEvent::PreferencesChanged(argui_platform::SystemPreferences {
                    color_scheme: argui_platform::ResolvedPreference {
                        value: self.preference_overrides.color_scheme.unwrap_or(scheme),
                        source: if self.preference_overrides.color_scheme.is_some() {
                            argui_platform::PreferenceSource::Override
                        } else {
                            argui_platform::PreferenceSource::System
                        },
                    },
                    ..self.preferences
                })
            }
            WindowEvent::RedrawRequested => {
                self.begin_frame_profile();
                self.advance_touch_selection(&window, event_loop);
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
