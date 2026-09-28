//! Application service, theme, and presentation events.

use super::{ActiveEventLoop, Application, UserEvent};
#[cfg(target_arch = "wasm32")]
use crate::RuntimeEvent;

impl Application {
    /// Applies a host event to its owning scene using the active native loop.
    pub(super) fn handle_host_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        if let UserEvent::ThemeChanged { window, change } = &event {
            if *window == self.window_key {
                self.apply_theme_change(change);
            }
            return;
        }
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
                UserEvent::NativeHostApplication(_) => {}
                UserEvent::GpuCanvasReady(id) => self.gpu_canvas_ready(id),
                #[cfg(feature = "tasks")]
                UserEvent::TasksReady => {
                    if let Some(tasks) = &self.tasks {
                        tasks.drain();
                    }
                    self.tasks_ready(event_loop);
                }
                UserEvent::Preferences { .. } => {}
                UserEvent::ThemeChanged { .. } => {}
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
            if let UserEvent::WebHostCommit(operations) = event {
                if let Err(error) = self.commit_native_host(operations, Vec::new()) {
                    (self.on_event)(RuntimeEvent::CommandFailed(error));
                }
                return;
            }
            let Some(window) = self.window.clone() else {
                return;
            };
            match event {
                UserEvent::WebHostCommit(_) => {}
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
                UserEvent::ThemeChanged { .. } => {}
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
}
