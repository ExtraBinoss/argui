//! Native tray registration and action routing for a multi-window application.

#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use std::sync::Arc;

use super::MultiApplication;
use crate::RuntimeEvent;
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use crate::{AppCommand, AppEvent, event::UserEvent};
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use argui_platform::{TrayAction, TrayEvent};

impl MultiApplication {
    /// Delivers a tray activation to the native command or application model.
    /// `event_loop` owns window commands and `event` identifies the activated item.
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn tray_event(
        &mut self,
        event_loop: &dyn crate::host::WindowFactory,
        event: TrayEvent,
    ) {
        self.emit(RuntimeEvent::Tray(event.clone()));
        if let TrayEvent::Action { action, .. } = &event
            && let Some(command) = tray_command(action.clone())
        {
            self.apply_command(event_loop, command);
            return;
        }
        let update = self.model.borrow_mut().update(&AppEvent::Tray(event));
        self.pending.borrow_mut().push(update);
        self.process_pending(event_loop);
    }

    /// Reconciles the tray with model or application configuration.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn sync_tray(&mut self) {
        let config = self
            .model
            .borrow()
            .tray()
            .or_else(|| self.config.tray.clone());
        if let Err(error) = self.apply_tray_config(config) {
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            self.emit(RuntimeEvent::TrayFailed(error));
            #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
            if !self.tray_unavailable_announced {
                self.tray_unavailable_announced = true;
                self.emit(RuntimeEvent::TrayUnavailable(error));
            }
        }
    }

    /// Applies a tray configuration and returns any native registration error.
    /// `config` replaces the current icon and menu, or removes them when absent.
    ///
    /// # Errors
    /// Returns an error if the platform rejects the configuration or has no tray backend.
    pub(super) fn apply_tray_config(
        &mut self,
        config: Option<argui_platform::TrayConfig>,
    ) -> Result<(), String> {
        let Some(config) = config else {
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            {
                self.native_tray = None;
            }
            return Ok(());
        };
        #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
        {
            if let Some(tray) = &mut self.native_tray {
                tray.sync(config, &self.config.identity.icons)
            } else {
                let Some(proxy) = self.event_proxy.clone() else {
                    return Err("native event loop is not ready".into());
                };
                let handler = Arc::new(move |event| {
                    let _ = proxy.send_event(UserEvent::Tray(event));
                });
                argui_platform::NativeTray::new(
                    &self.config.identity.id,
                    config,
                    &self.config.identity.icons,
                    handler,
                )
                .map(|tray| self.native_tray = Some(tray))
            }
        }
        #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
        {
            let _ = config;
            Err(if cfg!(target_arch = "wasm32") {
                "system tray is unavailable on the Web"
            } else {
                "rebuild Argui with the `tray` feature"
            }
            .into())
        }
    }
}

#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
fn tray_command(action: TrayAction) -> Option<AppCommand> {
    match action {
        TrayAction::Custom(_) => None,
        TrayAction::ShowWindow(key) => Some(AppCommand::ShowWindow(key)),
        TrayAction::HideWindow(key) => Some(AppCommand::HideWindow(key)),
        TrayAction::ToggleWindow(key) => Some(AppCommand::ToggleWindow(key)),
        TrayAction::FocusWindow(key) => Some(AppCommand::FocusWindow(key)),
        TrayAction::CloseWindow(key) => Some(AppCommand::CloseWindow(key)),
        TrayAction::Quit => Some(AppCommand::Quit),
    }
}
