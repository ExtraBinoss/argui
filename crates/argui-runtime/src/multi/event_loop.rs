use super::MultiApplication;
#[cfg(target_arch = "wasm32")]
use crate::RuntimeEvent;
use crate::event::UserEvent;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    window::WindowId,
};

impl ApplicationHandler<UserEvent> for MultiApplication {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        for entry in self.windows.values_mut() {
            entry.runtime.wake_due_animation();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "android")]
        if self.resume_android_windows(event_loop) {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if let Err(error) = argui_platform::apply_web_identity(&self.config.identity) {
            self.emit(RuntimeEvent::CommandFailed(error));
        }
        let initial = self.config.windows.clone();
        for spec in initial {
            self.open_window(event_loop, spec);
        }
        self.sync_tray();
        self.sync_global_shortcuts();
        self.process_pending(event_loop);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.shutdown();
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.suspend_windows(event_loop);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let _ = event_loop;
        match event {
            UserEvent::ModelsReady => self.models_ready(event_loop),
            #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
            UserEvent::HotReload { generation } => self.hot_reload(generation),
            #[cfg(feature = "tasks")]
            UserEvent::TasksReady => self.tasks_ready(event_loop),
            #[cfg(all(feature = "webview", target_os = "linux"))]
            UserEvent::NativeInput { .. } => {}
            UserEvent::Preferences { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(target_arch = "wasm32")]
            UserEvent::ClipboardText { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(target_arch = "wasm32")]
            UserEvent::Accessibility { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            UserEvent::AccessKit(event) => {
                if let Some(key) = self
                    .by_native
                    .get(&crate::host::HostId::Winit(event.window_id))
                    .cloned()
                    && let Some(entry) = self.windows.get_mut(&key)
                {
                    entry
                        .runtime
                        .user_event(event_loop, UserEvent::AccessKit(event));
                }
            }
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            UserEvent::Tray(event) => self.tray_event(event_loop, event),
            #[cfg(all(
                feature = "global-shortcuts",
                any(target_os = "linux", target_os = "windows", target_os = "macos")
            ))]
            UserEvent::GlobalShortcut(event) => {
                self.global_shortcut_event(event_loop, event);
            }
            #[cfg(all(
                feature = "global-shortcuts",
                any(target_os = "linux", target_os = "windows", target_os = "macos")
            ))]
            UserEvent::GlobalShortcutsFailed(error) => {
                self.emit(crate::RuntimeEvent::GlobalShortcutsFailed(error));
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let deadline = self
            .windows
            .values()
            .filter_map(|entry| entry.runtime.next_animation_deadline())
            .min();
        event_loop.set_control_flow(deadline.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        if let Some(entry) = self
            .windows
            .values_mut()
            .find(|entry| entry.runtime.is_popup_window(window_id))
        {
            entry.runtime.popup_event(event_loop, window_id, event);
            self.process_pending(event_loop);
            return;
        }
        let Some(key) = self
            .by_native
            .get(&crate::host::HostId::Winit(window_id))
            .cloned()
        else {
            return;
        };
        let close = matches!(event, WindowEvent::CloseRequested);
        if let Some(entry) = self.windows.get_mut(&key) {
            entry.runtime.window_event(event_loop, window_id, event);
        }
        self.synchronize_ui_zoom(&key);
        self.process_pending(event_loop);
        if close {
            self.handle_close(&key, event_loop);
        }
    }
}
