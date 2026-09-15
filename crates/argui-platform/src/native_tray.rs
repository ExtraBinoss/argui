use std::sync::Arc;

use crate::{ApplicationId, IconSet, TrayConfig, TrayEvent};

#[cfg(target_os = "linux")]
#[path = "native_tray/linux.rs"]
mod platform;
#[cfg(any(target_os = "windows", target_os = "macos"))]
#[path = "native_tray/tray_icon.rs"]
mod platform;

/// Thread-safe callback receiving native tray events.
pub type TrayEventHandler = Arc<dyn Fn(TrayEvent) + Send + Sync>;

/// Native system-tray icon and menu owner.
pub struct NativeTray(platform::PlatformTray);

impl NativeTray {
    /// Creates a native tray icon and connects its menu events to `handler`.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or the platform cannot create the tray.
    /// `application_id`, `config`, and `fallback_icons` define the native icon/menu; `handler` receives user events.
    pub fn new(
        application_id: &ApplicationId,
        config: TrayConfig,
        fallback_icons: &IconSet,
        handler: TrayEventHandler,
    ) -> Result<Self, String> {
        config.validate().map_err(|error| error.to_string())?;
        platform::PlatformTray::new(application_id, config, fallback_icons, handler).map(Self)
    }

    /// Applies an updated tray configuration.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or the platform update fails.
    /// `config` is the replacement tray state; `fallback_icons` supplies icons when no tray icon is set.
    pub fn sync(&mut self, config: TrayConfig, fallback_icons: &IconSet) -> Result<(), String> {
        config.validate().map_err(|error| error.to_string())?;
        self.0.sync(config, fallback_icons)
    }
}
