use argui::{render::RendererConfig, runtime::run_app};
mod service;
#[cfg(not(feature = "argui-live"))]
use argui_example_widget_gallery_dsl::Main;
use service::GalleryHost;

/// Launches the DSL gallery in AOT or live development mode.
///
/// # Errors
///
/// Returns connection, window, or renderer startup failures.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "argui-live")]
    let root = argui_dsl_runtime::LiveRuntime::connect(
        std::env::var("ARGUI_DEV_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4777".into()),
        std::time::Duration::from_secs(10),
    )?;
    #[cfg(not(feature = "argui-live"))]
    let root = Main::new();
    let root = GalleryHost::new(root)?;
    run_app(
        argui_example_widget_gallery_dsl::application_config(),
        RendererConfig::default().effects(argui_effects::registry()?),
        root,
        |_| {},
    )?;
    Ok(())
}
