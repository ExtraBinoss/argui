use argui::{platform::WindowConfig, render::RendererConfig, runtime::run_app_with_text_engine};
use argui_showcase::{StateShowcase, text_engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_app_with_text_engine(
        WindowConfig {
            title: "Argui state showcase".into(),
            width: 1050.0,
            height: 680.0,
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        text_engine(),
        StateShowcase::default(),
        |event| println!("{event:?}"),
    )?;
    Ok(())
}
