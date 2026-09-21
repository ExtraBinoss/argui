//! Independently runnable virtual-list example authored in Argui DSL.

argui::include_ui!();

/// Returns the native application configuration for the list example.
#[must_use]
pub fn application_config() -> argui::platform::ApplicationConfig {
    argui::platform::ApplicationConfig::new(
        argui::platform::ApplicationIdentity::development("Argui DSL Virtual List"),
        argui::platform::WindowConfig::default(),
    )
}
