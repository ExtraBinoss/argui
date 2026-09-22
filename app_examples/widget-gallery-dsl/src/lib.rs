//! Independently runnable Argui widget gallery implemented in DSL.

// The live client receives its UI from argui dev and never uses the generated AOT tree.
#[cfg(not(argui_dev_client))]
argui::include_ui!();

/// Returns the native application configuration for the DSL gallery.
#[must_use]
pub fn application_config() -> argui::platform::ApplicationConfig {
    argui::platform::ApplicationConfig::new(
        argui::platform::ApplicationIdentity::development("Argui Widget Gallery DSL"),
        argui::platform::WindowConfig::default(),
    )
}
