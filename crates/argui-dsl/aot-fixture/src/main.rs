#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::{render::RendererConfig, runtime::run_app};
#[cfg(not(feature = "argui-live"))]
use argui_dsl_aot_fixture::Main;

/// Launches the generated DSL fixture for private-display visual verification.
#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "argui-live")]
    let mut root = argui_dsl_runtime::LiveRuntime::connect(
        std::env::var("ARGUI_DEV_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4777".into()),
        std::time::Duration::from_secs(10),
    )?;
    #[cfg(not(feature = "argui-live"))]
    let root = Main::new();
    root.set_translator(|id| (id == "status.ready").then(|| "Ready".to_string()));
    run_app(
        argui_dsl_aot_fixture::application_config(),
        RendererConfig::default(),
        root,
        |_| {},
    )?;
    Ok(())
}
