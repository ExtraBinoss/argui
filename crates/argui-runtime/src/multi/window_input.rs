use argui_platform::{WindowInputRegion, WindowKey};

use super::MultiApplication;

impl MultiApplication {
    /// Installs a pointer policy and retains it for DPI, zoom, and size changes.
    /// `key` selects the window and `region` uses Argui UI logical coordinates.
    ///
    /// # Errors
    /// Returns an error for an unknown window, unsupported policy, invalid
    /// geometry, or a platform failure. The previous policy remains retained.
    pub(super) fn set_window_input_region(
        &mut self,
        key: &WindowKey,
        region: WindowInputRegion,
    ) -> Result<(), String> {
        let entry = self
            .windows
            .get_mut(key)
            .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()))?;
        let window = entry
            .runtime
            .window()
            .ok_or_else(|| format!("window '{}' is unavailable", key.as_str()))?;
        let capabilities = window.capabilities();
        if matches!(region, WindowInputRegion::PassThrough) && !capabilities.mouse_passthrough {
            return Err("mouse passthrough is unavailable on this window backend".into());
        }
        if matches!(region, WindowInputRegion::Exclude(_)) && !capabilities.input_regions {
            return Err("partial window input regions require X11 on this backend".into());
        }
        window.set_input_region(
            &region,
            window.native_scale_factor() * f64::from(self.ui_zoom_factor),
        )?;
        entry.input_region = region;
        Ok(())
    }

    /// Reapplies one retained input policy after native geometry or scale changes.
    /// `key` selects the window that changed.
    pub(super) fn refresh_window_input_region(&mut self, key: &WindowKey) {
        let Some(region) = self
            .windows
            .get(key)
            .map(|entry| entry.input_region.clone())
        else {
            return;
        };
        if let Err(error) = self.set_window_input_region(key, region) {
            self.emit(crate::RuntimeEvent::CommandFailed(error));
        }
    }
}
