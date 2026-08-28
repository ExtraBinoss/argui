use std::sync::Arc;

use crate::{ApplicationId, IconSet, TrayConfig, TrayEvent};

#[cfg(target_os = "linux")]
#[path = "native_tray/linux.rs"]
mod platform;
#[cfg(any(target_os = "windows", target_os = "macos"))]
#[path = "native_tray/tray_icon.rs"]
mod platform;

pub type TrayEventHandler = Arc<dyn Fn(TrayEvent) + Send + Sync>;

pub struct NativeTray(platform::PlatformTray);

impl NativeTray {
    pub fn new(
        application_id: &ApplicationId,
        config: TrayConfig,
        fallback_icons: &IconSet,
        handler: TrayEventHandler,
    ) -> Result<Self, String> {
        config.validate().map_err(|error| error.to_string())?;
        platform::PlatformTray::new(application_id, config, fallback_icons, handler).map(Self)
    }

    pub fn sync(&mut self, config: TrayConfig, fallback_icons: &IconSet) -> Result<(), String> {
        config.validate().map_err(|error| error.to_string())?;
        self.0.sync(config, fallback_icons)
    }
}
