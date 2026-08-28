use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::RendererConfig,
    runtime::run_app,
};
use argui_perf_showcase::PerfShowcase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui retained performance laboratory"),
            WindowConfig {
                title: "Argui retained performance laboratory".into(),
                width: 1100.0,
                height: 820.0,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        PerfShowcase::default(),
        |_| {},
    )?;
    Ok(())
}
