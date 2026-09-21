use argui::{render::RendererConfig, runtime::run_app};
#[cfg(not(feature = "argui-live"))]
use argui_example_dsl_live_demo::Main;

/// Runs the generated release demo or its hot-reload development runtime.
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
    run_app(
        argui_example_dsl_live_demo::application_config(),
        RendererConfig::default(),
        root,
        |_| {},
    )?;
    Ok(())
}
