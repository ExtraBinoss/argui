use std::sync::Arc;

use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_platform::{PlatformError, PlatformEvent, PointerButton};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
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

#[path = "safe_area.rs"]
mod safe_area;

impl Drop for Application {
    fn drop(&mut self) {
        if let Some(model) = &self.model {
            if self.exit_on_close {
                crate::shutdown_presentations(std::slice::from_ref(model));
            } else {
                model.close_presentation();
            }
        }
        #[cfg(feature = "tasks")]
        {
            if self.exit_on_close
                && let Some(tasks) = &self.tasks
            {
                tasks.shutdown();
            }
        }
        #[cfg(all(
            feature = "webview",
            any(
                target_arch = "wasm32",
                target_os = "linux",
                target_os = "windows",
                target_os = "macos"
            )
        ))]
        self.native_views.take();
    }
}

impl Application {
    pub(super) fn initialize_model_tree(&mut self) {
        if self.model.is_none() {
            return;
        }
        self.source_index.borrow_mut().clear();
        self.interaction_snapshot = crate::model::InteractionSnapshot::default();
        self.ui_tree = self.inspected_view().map(argui_ui::UiTree::new);
        if let Some(tree) = &mut self.ui_tree {
            tree.set_pointer_settings(self.pointer_settings);
            tree.set_reduced_motion(self.environment.reduced_motion);
            tree.set_desktop_backdrop_state(argui_ui::DesktopBackdropState {
                available: self.environment.desktop_backdrop_available,
                ..argui_ui::DesktopBackdropState::default()
            });
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler<UserEvent> for Application {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.wake_due_animation();
    }

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
                #[cfg(all(feature = "webview", target_arch = "wasm32"))]
                self.initialize_browser_webviews(&window);
                #[cfg(all(feature = "webview", any(target_os = "windows", target_os = "macos")))]
                self.initialize_winit_webviews(window.clone());
                let size = crate::host::WindowHost::drawable_size(&window);
                self.native_scale_factor = window.scale_factor() as f32;
                self.scale_factor = self.native_scale_factor * self.ui_zoom_factor;
                self.initialize_preference_snapshot(window.theme());
                self.refresh_safe_area_insets(&window, self.scale_factor);
                #[cfg(all(feature = "desktop-backdrop", not(target_arch = "wasm32")))]
                self.initialize_desktop_backdrop(window.clone());
                self.initialize_model_tree();
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
                self.window = Some(std::rc::Rc::new(window));
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
        #[cfg(target_os = "android")]
        {
            // Android destroys the native window while the activity is suspended.
            // Drop the WGPU surface first, then the Winit window. The shared device
            // remains available for the surface created by the next `resumed` event.
            *self.renderer.borrow_mut() = super::RendererState::Loading;
            self.window = None;
            self.accessibility = None;
            self.semantic_snapshot = None;
            self.renderer_announced = false;
            self.pointer = None;
            self.pointer_buttons = 0;
            self.touch_points.clear();
            self.primary_touch = None;
            self.touch_scroll.cancel();
            self.touch_selection = None;
            self.touch_selection_handle = None;
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Suspended));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "android")]
        if let Some(window) = self.window.clone() {
            // Winit currently consumes Android ContentRectChanged without
            // forwarding it, so sample once after each platform event batch.
            self.refresh_safe_area_insets(window.as_ref(), self.scale_factor);
        }
        event_loop.set_control_flow(
            self.next_animation_deadline()
                .map_or(ControlFlow::Wait, ControlFlow::WaitUntil),
        );
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
                UserEvent::ModelsReady => self.models_ready(event_loop),
                UserEvent::HostCommit(batch) => {
                    let result = if batch.window == self.window_key {
                        self.commit_native_host(batch.operations, batch.controls)
                    } else {
                        Err("native host batch targeted another window".into())
                    };
                    let _ = batch.reply.send(result);
                }
                UserEvent::GpuCanvasReady(id) => self.gpu_canvas_ready(id),
                #[cfg(feature = "tasks")]
                UserEvent::TasksReady => {
                    if let Some(tasks) = &self.tasks {
                        tasks.drain();
                    }
                    self.tasks_ready(event_loop);
                }
                UserEvent::Preferences { .. } => {}
                #[cfg(all(feature = "webview", target_os = "linux"))]
                UserEvent::NativeInput { .. } => {}
                UserEvent::AccessKit(event) => {
                    let Some(window) = self.window.clone() else {
                        return;
                    };
                    self.accessibility_event(event, &window, event_loop);
                }
                #[cfg(feature = "tray")]
                UserEvent::Tray(_) => {}
                #[cfg(all(
                    feature = "global-shortcuts",
                    any(target_os = "linux", target_os = "windows", target_os = "macos")
                ))]
                UserEvent::GlobalShortcut(_) => {}
                #[cfg(all(
                    feature = "global-shortcuts",
                    any(target_os = "linux", target_os = "windows", target_os = "macos")
                ))]
                UserEvent::GlobalShortcutsFailed(_) => {}
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = self.window.clone() else {
                return;
            };
            match event {
                UserEvent::ModelsReady => self.models_ready(event_loop),
                UserEvent::GpuCanvasReady(id) => self.gpu_canvas_ready(id),
                #[cfg(feature = "tasks")]
                UserEvent::TasksReady => {
                    if let Some(tasks) = &self.tasks {
                        tasks.drain();
                    }
                    self.tasks_ready(event_loop);
                }
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
        let Some(window) = self.window.clone() else {
            return;
        };
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        if self.is_popup_window(window_id) {
            self.popup_event(event_loop, window_id, event);
            return;
        }
        if self.window_id() != Some(crate::host::HostId::Winit(window_id)) {
            return;
        }
        if let WindowEvent::PinchGesture { delta, .. } = &event
            && self.handle_ui_zoom_magnify(*delta)
        {
            return;
        }
        let safe_area_scale = match &event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                Some(*scale_factor as f32 * self.ui_zoom_factor)
            }
            WindowEvent::Resized(_) | WindowEvent::RedrawRequested => Some(self.scale_factor),
            _ => None,
        };
        if let Some(scale_factor) = safe_area_scale {
            self.refresh_safe_area_insets(window.as_ref(), scale_factor);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(adapter) = &mut self.accessibility
            && let Some(native) = window.winit()
        {
            adapter.process_event(native, &event);
        }
        let platform_event = match event {
            WindowEvent::CloseRequested => PlatformEvent::CloseRequested,
            #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
            WindowEvent::Moved(_) => {
                self.invalidate_popup_environment();
                window.request_redraw();
                return;
            }
            WindowEvent::Resized(_) => {
                #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
                self.invalidate_popup_environment();
                self.sync_host_visibility();
                let size = window.drawable_size();
                self.pending_window_frame.resize(size.width, size.height);
                PlatformEvent::Resized {
                    width: size.width,
                    height: size.height,
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
                self.invalidate_popup_environment();
                let size = window.drawable_size();
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
                    modifiers: self.modifiers,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Moved, point)
                })
            }
            WindowEvent::CursorEntered { .. } => PlatformEvent::Pointer(PointerEvent {
                buttons: self.pointer_buttons,
                modifiers: self.modifiers,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(PointerPhase::Entered, self.pointer.unwrap_or_default())
            }),
            WindowEvent::CursorLeft { .. } => {
                let point = self.pointer.unwrap_or_default();
                self.pointer_left(&window, event_loop);
                PlatformEvent::Pointer(PointerEvent {
                    buttons: self.pointer_buttons,
                    modifiers: self.modifiers,
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
                    modifiers: self.modifiers,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Moved, self.pointer.unwrap_or_default())
                })
            }
            WindowEvent::Touch(touch) => {
                let phase = pointer_phase(touch.phase);
                let zoom = self.handle_ui_zoom_touch(
                    PointerId::new(touch.id.saturating_add(1)),
                    phase,
                    Point::new(touch.location.x as f32, touch.location.y as f32),
                );
                let point = Point::new(
                    touch.location.x as f32 / self.scale_factor,
                    touch.location.y as f32 / self.scale_factor,
                );
                let event = PointerEvent {
                    id: PointerId::new(touch.id.saturating_add(1)),
                    kind: PointerKind::Touch,
                    phase,
                    position: point,
                    button: Some(PointerButton::Primary),
                    buttons: u16::from(!matches!(
                        touch.phase,
                        winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                    )),
                    pressure: touch.force.map(|force| force.normalized() as f32),
                    primary: false,
                    modifiers: self.modifiers,
                    timestamp: self.input_epoch.elapsed(),
                };
                let event = self.touch_pointer(event, zoom, &window, event_loop);
                PlatformEvent::Pointer(event)
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let delta = scroll_delta(delta, self.scale_factor);
                if !self.handle_ui_zoom_wheel(delta) {
                    self.queue_pointer_scroll(delta, phase, &window, event_loop);
                }
                PlatformEvent::PointerScrolled(delta)
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers_state(modifiers.state());
                return;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let input = key_input(event, self.modifiers);
                if !self.handle_ui_zoom_key(&input) {
                    self.keyboard_input(&input, &window, event_loop);
                }
                PlatformEvent::Keyboard(input)
            }
            WindowEvent::Ime(ime) => {
                let input = ime_input(ime);
                self.ime_input(input.clone(), &window, event_loop);
                PlatformEvent::Ime(input)
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                self.sync_host_visibility();
                return;
            }
            WindowEvent::Focused(focused) => {
                self.sync_host_visibility();
                #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
                if focused {
                    self.popups.suspended = false;
                }
                #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
                if !focused && !self.popups.entries.is_empty() {
                    self.popups.pending_blur = true;
                    window.request_redraw();
                    return;
                }
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
                self.redraw(event_loop);
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
