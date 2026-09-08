use argui_platform::{WindowCapabilities, WindowKey, WindowLevel};

use super::MultiApplication;
use crate::{AppCommand, RuntimeEvent};

impl MultiApplication {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn apply_command(
        &mut self,
        event_loop: &dyn crate::host::WindowFactory,
        command: AppCommand,
    ) {
        match command {
            AppCommand::OpenWindow(spec) => self.open_window(event_loop, spec),
            AppCommand::CloseWindow(key) => self.close_window(&key),
            AppCommand::ShowWindow(key) => self.set_visible(&key, true),
            AppCommand::HideWindow(key) => self.set_visible(&key, false),
            AppCommand::ToggleWindow(key) => {
                if let Some(window) = self
                    .windows
                    .get(&key)
                    .and_then(|entry| entry.runtime.window())
                {
                    self.set_visible(&key, !window.is_visible().unwrap_or(true));
                }
            }
            AppCommand::FocusWindow(key) => {
                self.set_visible(&key, true);
                if let Some(window) = self
                    .windows
                    .get(&key)
                    .and_then(|entry| entry.runtime.window())
                {
                    window.set_minimized(false);
                    window.focus_window();
                }
            }
            AppCommand::SetWindowTitle { window, title } => {
                if let Some(window) = self
                    .windows
                    .get(&window)
                    .and_then(|entry| entry.runtime.window())
                {
                    window.set_title(&title);
                }
            }
            AppCommand::MinimizeWindow(key) => {
                self.with_window_capability(
                    &key,
                    "minimization",
                    |capabilities| capabilities.minimize,
                    |window| {
                        window.set_minimized(true);
                        Ok(())
                    },
                );
                if let Some(entry) = self.windows.get_mut(&key) {
                    entry.runtime.sync_host_visibility();
                }
            }
            AppCommand::SetWindowMaximized { window, maximized } => {
                self.with_window_capability(
                    &window,
                    "maximization",
                    |capabilities| capabilities.maximize,
                    |window| {
                        window.set_maximized(maximized);
                        Ok(())
                    },
                );
            }
            AppCommand::ToggleWindowMaximized(key) => {
                self.with_window_capability(
                    &key,
                    "maximization",
                    |capabilities| capabilities.maximize,
                    |window| {
                        window.set_maximized(!window.is_maximized());
                        Ok(())
                    },
                );
            }
            AppCommand::SetWindowLevel { window, level } => {
                self.with_window_capability(
                    &window,
                    "window levels",
                    |capabilities| level == WindowLevel::Normal || capabilities.window_level,
                    |window| {
                        window.set_window_level(level.into());
                        Ok(())
                    },
                );
            }
            AppCommand::SetWindowMousePassthrough {
                window,
                passthrough,
            } => {
                self.with_window_capability(
                    &window,
                    "mouse passthrough",
                    |capabilities| capabilities.mouse_passthrough,
                    |window| {
                        window
                            .set_cursor_hittest(!passthrough)
                            .map_err(|error| error.to_string())
                    },
                );
            }
            AppCommand::Quit => event_loop.exit(),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn with_window_capability(
        &self,
        key: &WindowKey,
        name: &str,
        supported: impl FnOnce(WindowCapabilities) -> bool,
        apply: impl FnOnce(&dyn crate::host::WindowHost) -> Result<(), String>,
    ) {
        let Some(window) = self
            .windows
            .get(key)
            .and_then(|entry| entry.runtime.window())
        else {
            self.emit(RuntimeEvent::CommandFailed(format!(
                "window does not exist: {}",
                key.as_str()
            )));
            return;
        };
        if supported(window.capabilities()) {
            if let Err(error) = apply(window) {
                self.emit(RuntimeEvent::CommandFailed(format!(
                    "{name} failed for window '{}': {error}",
                    key.as_str()
                )));
            }
        } else {
            self.emit(RuntimeEvent::CommandFailed(format!(
                "{name} is unavailable for window '{}' on this backend",
                key.as_str()
            )));
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn set_visible(&mut self, key: &WindowKey, visible: bool) {
        if let Some(entry) = self.windows.get_mut(key) {
            entry.runtime.set_window_visible(visible);
        }
    }
}
