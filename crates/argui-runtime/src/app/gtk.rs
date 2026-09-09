use super::Application;
use crate::{
    RuntimeError, RuntimeEvent,
    event::UserEvent,
    host::{LoopControl, WindowHost, gtk::GtkHost},
};
use argui_core::{Point, PointerEvent, PointerPhase};
use argui_platform::{PlatformEvent, gtk_host::GtkWindow};
use argui_render::SurfaceRenderer;
use std::{cell::Cell, rc::Rc};
use tao::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoopWindowTarget,
};

impl Application {
    pub(crate) fn gtk_preferences(&mut self, preferences: argui_platform::SystemPreferences) {
        self.apply_preferences(preferences);
    }
    pub(crate) fn initialize_gtk(
        &mut self,
        target: &EventLoopWindowTarget<UserEvent>,
        control: &dyn LoopControl,
    ) {
        if self.window.is_some() {
            return;
        }
        let platform = match GtkWindow::new(target, &self.window_config) {
            Ok(window) => window,
            Err(error) => {
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::WindowCreationFailed(
                    error.clone(),
                )));
                self.fatal_error = Some(RuntimeError::NativeHost(error));
                control.exit();
                return;
            }
        };
        let host = Rc::new(GtkHost {
            platform,
            ime_enabled: Cell::new(false),
        });
        let (width, height, scale) = host.platform.client_size();
        self.window = Some(host.clone());
        self.window_config.transparent = true;
        self.initialize_gtk_webviews(host.platform.container().clone());
        self.scale_factor = scale;
        self.initialize_preference_snapshot(None);
        self.initialize_model_tree();
        self.update_viewport(width, height);
        if !self.prepare_or_exit(control) {
            return;
        }
        let shared = self.renderer_device.borrow().clone();
        let config = self.surface_renderer_config();
        let renderer = match shared {
            Some(device) => pollster::block_on(SurfaceRenderer::new_with_device(
                host.platform.canvas(),
                width.max(1),
                height.max(1),
                config,
                device,
            )),
            None => pollster::block_on(SurfaceRenderer::new(
                host.platform.canvas(),
                width.max(1),
                height.max(1),
                config,
            )),
        };
        self.install_renderer(renderer, &host, control);
        host.set_visible(self.initial_visible);
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Opened {
            width,
            height,
            scale_factor: f64::from(scale),
            capabilities: host.capabilities(),
        }));
        self.window = Some(host);
        self.initialize_preferences();
    }

    pub(crate) fn gtk_geometry(&mut self) {
        self.sync_host_visibility();
        if !self.presentation_visible {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(host) = window.gtk() else {
            return;
        };
        if let Err(error) = host.platform.sync_canvas() {
            self.fatal_error = Some(RuntimeError::NativeHost(error));
            return;
        }
        let (width, height, scale) = host.platform.client_size();
        if self.scale_factor != scale {
            self.pending_window_frame.scale_factor(scale, width, height);
            window.request_redraw();
        } else if self.viewport.width != width as f32 / scale
            || self.viewport.height != height as f32 / scale
        {
            self.pending_window_frame.resize(width, height);
            window.request_redraw();
        }
        let radius = host.platform.corner_radius();
        if self.native_corner_radius != Some(radius) {
            self.native_corner_radius = Some(radius);
            self.pending_ui_frame.request_rebuild();
            window.request_redraw();
        }
    }

    pub(crate) fn gtk_event(&mut self, event: WindowEvent<'_>, control: &dyn LoopControl) {
        self.sync_host_visibility();
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(host) = window.gtk() else {
            return;
        };
        let platform = match event {
            WindowEvent::CloseRequested => PlatformEvent::CloseRequested,
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.gtk_geometry();
                return;
            }
            WindowEvent::CursorMoved { .. } => {
                let Some(point) = host.platform.pointer_position() else {
                    return;
                };
                if self
                    .native_views
                    .as_ref()
                    .is_some_and(|pool| pool.contains_pointer(point))
                    && self.pointer_buttons == 0
                {
                    self.gtk_pointer_boundary(control);
                    return;
                }
                self.pointer_moved(point, &window, control);
                PlatformEvent::Pointer(PointerEvent::mouse(PointerPhase::Moved, point))
            }
            WindowEvent::CursorEntered { .. } => PlatformEvent::Pointer(PointerEvent::mouse(
                PointerPhase::Entered,
                self.pointer.unwrap_or_default(),
            )),
            WindowEvent::CursorLeft { .. } => {
                let point = self.pointer.unwrap_or_default();
                self.pointer_left(&window, control);
                PlatformEvent::Pointer(PointerEvent::mouse(PointerPhase::Left, point))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let point = host.platform.pointer_position();
                if point.is_some_and(|point| {
                    self.native_views
                        .as_ref()
                        .is_some_and(|pool| pool.contains_pointer(point))
                }) && self.pointer_buttons == 0
                {
                    self.gtk_pointer_boundary(control);
                    return;
                }
                if state == ElementState::Pressed {
                    host.platform.focus_canvas();
                }
                if let Some(point) = point {
                    self.pointer_moved(point, &window, control);
                }
                let button = match button {
                    MouseButton::Left => argui_core::PointerButton::Primary,
                    MouseButton::Right => argui_core::PointerButton::Secondary,
                    MouseButton::Middle => argui_core::PointerButton::Middle,
                    MouseButton::Other(value) => argui_core::PointerButton::Other(value),
                    _ => return,
                };
                let pressed = state == ElementState::Pressed;
                let mask = crate::translate::pointer_button_mask(button);
                if pressed {
                    self.pointer_buttons |= mask;
                } else {
                    self.pointer_buttons &= !mask;
                }
                let state = if pressed {
                    argui_platform::ButtonState::Pressed
                } else {
                    argui_platform::ButtonState::Released
                };
                match button {
                    argui_core::PointerButton::Primary => {
                        self.primary_button(state, &window, control)
                    }
                    argui_core::PointerButton::Secondary => {
                        self.secondary_button(state, &window, control)
                    }
                    _ => (),
                }
                PlatformEvent::Pointer(PointerEvent {
                    button: Some(button),
                    buttons: self.pointer_buttons,
                    modifiers: self.modifiers,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(
                        if pressed {
                            PointerPhase::Pressed
                        } else {
                            PointerPhase::Released
                        },
                        self.pointer.unwrap_or_default(),
                    )
                })
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                if host.platform.pointer_position().is_some_and(|point| {
                    self.native_views
                        .as_ref()
                        .is_some_and(|pool| pool.contains_pointer(point))
                }) {
                    return;
                }
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        argui_core::ScrollDelta::Lines(Point::new(x, y))
                    }
                    MouseScrollDelta::PixelDelta(point) => {
                        argui_core::ScrollDelta::Pixels(Point::new(
                            point.x as f32 / self.scale_factor,
                            point.y as f32 / self.scale_factor,
                        ))
                    }
                    _ => return,
                };
                let phase = match phase {
                    tao::event::TouchPhase::Started => winit::event::TouchPhase::Started,
                    tao::event::TouchPhase::Moved => winit::event::TouchPhase::Moved,
                    tao::event::TouchPhase::Ended => winit::event::TouchPhase::Ended,
                    tao::event::TouchPhase::Cancelled => winit::event::TouchPhase::Cancelled,
                    _ => return,
                };
                self.queue_pointer_scroll(delta, phase, &window, control);
                PlatformEvent::PointerScrolled(delta)
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = argui_core::Modifiers {
                    shift: modifiers.shift_key(),
                    control: modifiers.control_key(),
                    alt: modifiers.alt_key(),
                    super_key: modifiers.super_key(),
                };
                return;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if host.platform.native_content_focused() {
                    return;
                }
                let input = argui_core::KeyInput {
                    key: key(event.logical_key),
                    text: event.text.map(str::to_owned),
                    state: if event.state == ElementState::Pressed {
                        argui_core::KeyState::Pressed
                    } else {
                        argui_core::KeyState::Released
                    },
                    modifiers: self.modifiers,
                    repeat: event.repeat,
                };
                self.keyboard_input(&input, &window, control);
                PlatformEvent::Keyboard(input)
            }
            WindowEvent::ReceivedImeText(text)
                if host.ime_enabled.get() && !host.platform.native_content_focused() =>
            {
                let input = argui_core::ImeInput::Commit(text);
                self.ime_input(input.clone(), &window, control);
                PlatformEvent::Ime(input)
            }
            WindowEvent::Focused(focused) => {
                self.window_focus(focused, &window, control);
                PlatformEvent::Focused(focused)
            }
            _ => return,
        };
        if platform.requires_redraw() {
            window.request_redraw();
        }
        (self.on_event)(RuntimeEvent::Platform(platform));
    }
}

impl Application {
    pub(crate) fn gtk_pointer_boundary(&mut self, control: &dyn LoopControl) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(host) = window.gtk() else {
            return;
        };
        if self.pointer_buttons == 0
            && self.pointer.is_some()
            && host.platform.pointer_position().is_some_and(|point| {
                self.native_views
                    .as_ref()
                    .is_some_and(|pool| pool.contains_pointer(point))
            })
        {
            self.pointer_left(&window, control);
        }
        if host.platform.native_content_focused()
            && let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout)
            && ui.focused_node().is_some()
        {
            let update = ui.sync_focus(&layout.hit_regions, Some(argui_ui::FocusRequest::Clear));
            self.apply_ui_update(update, &window, control);
            self.update_ime(&window);
        }
    }
}

fn key(key: tao::keyboard::Key<'_>) -> argui_core::Key {
    use argui_core::Key;
    use tao::keyboard::Key as K;
    match key {
        K::Character(text) => Key::Character(text.to_owned()),
        K::ArrowLeft => Key::ArrowLeft,
        K::ArrowRight => Key::ArrowRight,
        K::ArrowUp => Key::ArrowUp,
        K::ArrowDown => Key::ArrowDown,
        K::PageUp => Key::PageUp,
        K::PageDown => Key::PageDown,
        K::ContextMenu => Key::ContextMenu,
        K::F1 => Key::Function(1),
        K::F2 => Key::Function(2),
        K::F3 => Key::Function(3),
        K::F4 => Key::Function(4),
        K::F5 => Key::Function(5),
        K::F6 => Key::Function(6),
        K::F7 => Key::Function(7),
        K::F8 => Key::Function(8),
        K::F9 => Key::Function(9),
        K::F10 => Key::Function(10),
        K::F11 => Key::Function(11),
        K::F12 => Key::Function(12),
        K::F13 => Key::Function(13),
        K::F14 => Key::Function(14),
        K::F15 => Key::Function(15),
        K::F16 => Key::Function(16),
        K::F17 => Key::Function(17),
        K::F18 => Key::Function(18),
        K::F19 => Key::Function(19),
        K::F20 => Key::Function(20),
        K::F21 => Key::Function(21),
        K::F22 => Key::Function(22),
        K::F23 => Key::Function(23),
        K::F24 => Key::Function(24),
        K::F25 => Key::Function(25),
        K::F26 => Key::Function(26),
        K::F27 => Key::Function(27),
        K::F28 => Key::Function(28),
        K::F29 => Key::Function(29),
        K::F30 => Key::Function(30),
        K::F31 => Key::Function(31),
        K::F32 => Key::Function(32),
        K::F33 => Key::Function(33),
        K::F34 => Key::Function(34),
        K::F35 => Key::Function(35),
        K::Home => Key::Home,
        K::End => Key::End,
        K::Backspace => Key::Backspace,
        K::Delete => Key::Delete,
        K::Enter => Key::Enter,
        K::Tab => Key::Tab,
        K::Escape => Key::Escape,
        _ => Key::Other,
    }
}
