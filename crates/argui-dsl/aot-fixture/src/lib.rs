//! Release dependency and generated-Rust compilation fixture.

argui::include_ui!();

/// Returns the native application configuration used by the DSL fixture.
///
/// The returned configuration provides the fixture's development identity and
/// one default main window.
#[must_use]
pub fn application_config() -> argui::platform::ApplicationConfig {
    argui::platform::ApplicationConfig::new(
        argui::platform::ApplicationIdentity::development("Argui DSL fixture"),
        argui::platform::WindowConfig::default(),
    )
}
