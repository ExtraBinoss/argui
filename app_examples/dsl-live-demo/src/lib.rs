//! Minimal AOT and live development example for the Argui DSL.

argui::include_ui!();

/// Returns the native window configuration for the live DSL demo.
#[must_use]
pub fn application_config() -> argui::platform::ApplicationConfig {
    argui::platform::ApplicationConfig::new(
        argui::platform::ApplicationIdentity::development("Argui Live Studio"),
        argui::platform::WindowConfig::default(),
    )
}
