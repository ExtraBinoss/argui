use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig, WindowLevel},
    render::RendererConfig,
    runtime::{RuntimeEvent, run_application_with_text_engine},
    text::TextEngine,
};
use argui_showcase::spotlight::{
    SPOTLIGHT_WINDOW_HEIGHT, SPOTLIGHT_WINDOW_WIDTH, SpotlightShowcase,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui Spotlight"),
            WindowConfig {
                title: "Argui Spotlight".into(),
                width: SPOTLIGHT_WINDOW_WIDTH,
                height: SPOTLIGHT_WINDOW_HEIGHT,
                decorations: false,
                resizable: true,
                transparent: true,
                native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
                level: WindowLevel::AlwaysOnTop,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        text,
        SpotlightShowcase::default(),
        |event| {
            if matches!(
                event,
                RuntimeEvent::CommandFailed(_) | RuntimeEvent::RendererFailed(_)
            ) {
                eprintln!("{event:?}");
            }
        },
    )?;
    Ok(())
}
