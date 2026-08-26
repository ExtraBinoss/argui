use argui::{
    platform::{PlatformEvent, WindowConfig},
    render::RendererConfig,
    runtime::{RuntimeEvent, run},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WindowConfig {
        title: "Argui — Window step".into(),
        width: 960.0,
        height: 640.0,
        decorations: true,
        resizable: true,
        transparent: false,
        append_to_document: true,
    };

    run(config, RendererConfig::default(), |event| {
        if !matches!(
            event,
            RuntimeEvent::Platform(PlatformEvent::RedrawRequested)
        ) {
            println!("{event:?}");
        }
    })?;
    Ok(())
}
