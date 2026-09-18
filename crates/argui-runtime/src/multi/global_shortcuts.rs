#[cfg(all(
    feature = "global-shortcuts",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
use std::sync::Arc;

#[cfg(all(
    feature = "global-shortcuts",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
use argui_platform::GlobalShortcutEvent;

use super::MultiApplication;
#[cfg(all(
    feature = "global-shortcuts",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
use crate::AppEvent;
use crate::RuntimeEvent;
#[cfg(all(
    feature = "global-shortcuts",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
use crate::event::UserEvent;

impl MultiApplication {
    /// Creates the configured platform shortcut owner once the event proxy is ready.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn sync_global_shortcuts(&mut self) {
        if self.config.global_shortcuts.is_empty() {
            return;
        }

        #[cfg(all(
            feature = "global-shortcuts",
            any(target_os = "linux", target_os = "windows", target_os = "macos")
        ))]
        {
            if self.native_global_shortcuts.is_some() {
                return;
            }
            #[cfg(target_os = "linux")]
            if let Some(error) = self.global_shortcut_setup_error.take() {
                self.emit(RuntimeEvent::GlobalShortcutsFailed(error));
                return;
            }
            let Some(proxy) = self.event_proxy.clone() else {
                return;
            };
            let handler = Arc::new(move |event| {
                let _ = proxy.send_event(UserEvent::GlobalShortcut(event));
            });
            let Some(proxy) = self.event_proxy.clone() else {
                return;
            };
            let on_error = Arc::new(move |error| {
                let _ = proxy.send_event(UserEvent::GlobalShortcutsFailed(error));
            });
            match argui_platform::NativeGlobalShortcuts::new(
                self.config.identity.linux_application_id(),
                &self.config.global_shortcuts,
                handler,
                on_error,
            ) {
                Ok(shortcuts) => self.native_global_shortcuts = Some(shortcuts),
                Err(error) => self.emit(RuntimeEvent::GlobalShortcutsFailed(error)),
            }
        }

        #[cfg(not(all(
            feature = "global-shortcuts",
            any(target_os = "linux", target_os = "windows", target_os = "macos")
        )))]
        if !self.global_shortcuts_unavailable_announced {
            self.global_shortcuts_unavailable_announced = true;
            self.emit(RuntimeEvent::GlobalShortcutsUnavailable(
                if cfg!(feature = "global-shortcuts") {
                    "global shortcuts are available only on Windows, macOS, and Linux"
                } else {
                    "rebuild Argui with the `global-shortcuts` feature"
                }
                .into(),
            ));
        }
    }

    /// Delivers one native shortcut transition and applies its model commands.
    #[cfg(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    ))]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn global_shortcut_event(
        &mut self,
        event_loop: &dyn crate::host::WindowFactory,
        event: GlobalShortcutEvent,
    ) {
        self.pending_activation_token = event.activation_token.clone();
        self.emit(RuntimeEvent::GlobalShortcut(event.clone()));
        let update = self
            .model
            .borrow_mut()
            .update(&AppEvent::GlobalShortcut(event));
        self.pending.borrow_mut().push(update);
        self.process_pending(event_loop);
        self.pending_activation_token = None;
    }
}
