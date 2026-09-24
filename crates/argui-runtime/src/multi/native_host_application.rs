//! Application controls received from a native JavaScript presentation.

use argui_platform::{GlobalShortcut, WindowKey};

use super::MultiApplication;
use crate::{AppCommand, NativeHostApplicationRequest, NativeWindowInfo};

impl MultiApplication {
    /// Applies one application request on the UI thread and replies to its caller.
    /// `event_loop` owns native windows; `request` contains the change and reply channel.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn native_host_application_request(
        &mut self,
        event_loop: &dyn crate::host::WindowFactory,
        request: NativeHostApplicationRequest,
    ) {
        match request {
            NativeHostApplicationRequest::SetTray(config, reply) => {
                let result = config
                    .as_ref()
                    .map_or(Ok(()), |config| {
                        config.validate().map_err(|error| error.to_string())
                    })
                    .and_then(|()| self.apply_tray_config(config.clone()));
                if result.is_ok() {
                    self.config.tray = config;
                }
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::SetGlobalShortcuts(shortcuts, reply) => {
                let _ = reply.send(self.replace_global_shortcuts(shortcuts));
            }
            NativeHostApplicationRequest::SetCloseBehavior(behavior, reply) => {
                let result = if let Some(entry) = self.windows.get_mut(&WindowKey::main()) {
                    entry.spec.window.close_behavior = behavior;
                    Ok(())
                } else {
                    Err("main window is unavailable".into())
                };
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::FocusWindow(reply) => {
                let result = if self.windows.contains_key(&WindowKey::main()) {
                    self.apply_command(event_loop, AppCommand::FocusWindow(WindowKey::main()));
                    Ok(())
                } else {
                    Err("main window is unavailable".into())
                };
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::OpenWindow(spec, reply) => {
                let key = spec.key.clone();
                let result = if self.windows.contains_key(&key) {
                    self.apply_command(event_loop, AppCommand::FocusWindow(key));
                    Ok(())
                } else {
                    self.open_window(event_loop, spec);
                    self.windows
                        .contains_key(&key)
                        .then_some(())
                        .ok_or_else(|| format!("could not open window '{}'", key.as_str()))
                };
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::SendWindowMessage(window, message, reply) => {
                let result = if self.windows.contains_key(&window) {
                    let update = self
                        .model
                        .borrow_mut()
                        .update(&crate::AppEvent::HostMessage { window, message });
                    self.pending.borrow_mut().push(update);
                    self.process_pending(event_loop);
                    Ok(())
                } else {
                    Err(format!("window '{}' is unavailable", window.as_str()))
                };
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::GetWindowInfo(key, reply) => {
                let result = self
                    .windows
                    .get(&key)
                    .and_then(|entry| entry.runtime.window().map(|window| (entry, window)))
                    .map(|(entry, window)| {
                        let (width, height) = window.logical_size();
                        NativeWindowInfo {
                            window: key.clone(),
                            title: entry.spec.window.title.clone(),
                            width,
                            height,
                            visible: window.is_visible(),
                            decorations: entry.spec.window.decorations,
                            transparent: entry.spec.window.transparent,
                            backdrop: entry.spec.window.desktop_backdrop,
                            backdrop_available: entry.runtime.desktop_backdrop_available(),
                        }
                    })
                    .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()));
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::SetWindowTitle(key, title, reply) => {
                let result = self
                    .windows
                    .get_mut(&key)
                    .and_then(|entry| {
                        entry.runtime.window().map(|window| {
                            window.set_title(&title);
                            entry.spec.window.title = title;
                        })
                    })
                    .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()));
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::SetWindowSize(key, width, height, reply) => {
                let result = if !width.is_finite()
                    || !height.is_finite()
                    || !(1.0..=16384.0).contains(&width)
                    || !(1.0..=16384.0).contains(&height)
                {
                    Err("window width and height must be finite values from 1 to 16384".into())
                } else {
                    self.windows
                        .get_mut(&key)
                        .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()))
                        .and_then(|entry| {
                            let window = entry.runtime.window().ok_or_else(|| {
                                format!("window '{}' is unavailable", key.as_str())
                            })?;
                            window.request_inner_size(width, height)?;
                            entry.spec.window.width = width;
                            entry.spec.window.height = height;
                            Ok(())
                        })
                };
                let _ = reply.send(result);
            }
            NativeHostApplicationRequest::SetWindowDecorations(key, decorations, reply) => {
                let result =
                    self.windows
                        .get_mut(&key)
                        .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()))
                        .and_then(|entry| {
                            let window = entry.runtime.window().ok_or_else(|| {
                                format!("window '{}' is unavailable", key.as_str())
                            })?;
                            window.set_decorations(decorations);
                            entry.spec.window.decorations = decorations;
                            Ok(())
                        });
                let _ = reply.send(result);
            }
        }
    }

    /// Replaces native shortcut registrations while retaining the previous set on failure.
    /// `shortcuts` is the complete desired set, including an empty set to disable registration.
    ///
    /// # Errors
    /// Returns a validation, portal, or native registration error.
    fn replace_global_shortcuts(&mut self, shortcuts: Vec<GlobalShortcut>) -> Result<(), String> {
        let mut next = self.config.clone();
        next.global_shortcuts = shortcuts.clone();
        next.validate().map_err(|error| error.to_string())?;
        if self.config.global_shortcuts == shortcuts {
            return Ok(());
        }
        #[cfg(all(
            feature = "global-shortcuts",
            any(target_os = "linux", target_os = "windows", target_os = "macos")
        ))]
        {
            let previous = self.config.global_shortcuts.clone();
            self.native_global_shortcuts = None;
            let result = if shortcuts.is_empty() {
                Ok(None)
            } else {
                self.create_shortcut_owner(&shortcuts).map(Some)
            };
            match result {
                Ok(owner) => {
                    self.native_global_shortcuts = owner;
                    self.config.global_shortcuts = shortcuts;
                    Ok(())
                }
                Err(error) => {
                    if !previous.is_empty() {
                        self.native_global_shortcuts = self.create_shortcut_owner(&previous).ok();
                    }
                    Err(error)
                }
            }
        }
        #[cfg(not(all(
            feature = "global-shortcuts",
            any(target_os = "linux", target_os = "windows", target_os = "macos")
        )))]
        {
            let _ = shortcuts;
            Err("global shortcuts require the `global-shortcuts` feature on Linux, Windows, or macOS".into())
        }
    }

    /// Creates a platform shortcut owner connected to this event loop.
    /// `shortcuts` contains the already validated registrations.
    ///
    /// # Errors
    /// Returns a portal setup or platform registration failure.
    #[cfg(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    ))]
    fn create_shortcut_owner(
        &self,
        shortcuts: &[GlobalShortcut],
    ) -> Result<argui_platform::NativeGlobalShortcuts, String> {
        #[cfg(target_os = "linux")]
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            argui_platform::prepare_wayland_global_shortcuts(
                self.config.identity.linux_application_id(),
            )?;
        }
        let proxy = self
            .event_proxy
            .clone()
            .ok_or("native event loop is not ready")?;
        let handler = std::sync::Arc::new(move |event| {
            let _ = proxy.send_event(crate::event::UserEvent::GlobalShortcut(event));
        });
        let proxy = self
            .event_proxy
            .clone()
            .ok_or("native event loop is not ready")?;
        let on_error = std::sync::Arc::new(move |error| {
            let _ = proxy.send_event(crate::event::UserEvent::GlobalShortcutsFailed(error));
        });
        argui_platform::NativeGlobalShortcuts::new(
            self.config.identity.linux_application_id(),
            shortcuts,
            handler,
            on_error,
        )
    }
}
