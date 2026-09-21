use argui::{render::RendererConfig, runtime::run_app};
#[cfg(not(feature = "argui-live"))]
use argui_example_dsl_virtual_list::Main;

/// Launches the virtual-list example with live DSL or generated release UI.
///
/// # Errors
///
/// Returns connection, effect-registry, window, or renderer startup failures.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "argui-live")]
    let root = argui_dsl_runtime::LiveRuntime::connect(
        std::env::var("ARGUI_DEV_ADDRESS").unwrap_or_else(|_| "127.0.0.1:4777".into()),
        std::time::Duration::from_secs(10),
    )?;
    #[cfg(not(feature = "argui-live"))]
    let root = {
        let root = Main::new();
        root.set_entries(
            (0..10_000)
                .map(|index| format!("Record {:05}", index + 1))
                .collect(),
        );
        root
    };
    run_app(
        argui_example_dsl_virtual_list::application_config(),
        RendererConfig::default().effects(argui_effects::registry()?),
        root,
        |_| {},
    )?;
    Ok(())
}
