use argui::{
    core::{Point, Rect, Size},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::{RuntimeEvent, run_with_text},
    text::{FontFamily, TextBlock, TextColor, TextScene},
};

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

fn demo_scene() -> TextScene {
    TextScene::new()
        .with(
            TextBlock::new("Argui text renderer", bounds(48.0, 38.0, 850.0, 70.0))
                .size(48.0)
                .weight(700)
                .color(TextColor::srgb(0.35, 0.85, 1.0)),
        )
        .with(
            TextBlock::new(
                "Cosmic Text shaping · Swash rasterization · instanced WGPU glyphs",
                bounds(50.0, 108.0, 900.0, 42.0),
            )
            .size(20.0)
            .color(TextColor::srgb(0.72, 0.76, 0.84)),
        )
        .with(
            TextBlock::new(
                "14 px — précis, léger, café, déjà vu  •  20 px — readable UI text  •  32 px — Display",
                bounds(50.0, 174.0, 900.0, 95.0),
            )
            .size(20.0)
            .color(TextColor::srgb(0.94, 0.95, 0.98)),
        )
        .with(
            TextBlock::new(
                "Unicode & BiDi: العربية جميلة — שלום עולם — Rust 🦀  ✨ 🚀",
                bounds(50.0, 280.0, 900.0, 70.0),
            )
            .size(28.0)
            .color(TextColor::srgb(1.0, 0.72, 0.38)),
        )
        .with(
            TextBlock::new(
                "Monospace 18 px:  fn render(text: &TextScene) -> Frame",
                bounds(50.0, 372.0, 900.0, 48.0),
            )
            .size(18.0)
            .family(FontFamily::Monospace)
            .color(TextColor::srgb(0.62, 0.92, 0.68)),
        )
        .with(
            TextBlock::new(
                "Clipped text proof — this sentence wraps inside a deliberately small rectangle and everything below its lower edge is discarded by the GPU shader. The same bounds will later come directly from Taffy.",
                bounds(50.0, 458.0, 520.0, 92.0),
            )
            .size(22.0)
            .line_height(28.0)
            .color(TextColor::srgb(0.92, 0.56, 0.82)),
        )
        .with(
            TextBlock::new(
                "64",
                bounds(700.0, 430.0, 180.0, 110.0),
            )
            .size(64.0)
            .weight(700)
            .family(FontFamily::Serif)
            .color(TextColor::srgba(0.72, 0.62, 1.0, 0.95)),
        )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = WindowConfig {
        title: "Argui — GPU text demo".into(),
        width: 1000.0,
        height: 650.0,
        ..WindowConfig::default()
    };

    run_with_text(window, RendererConfig::default(), demo_scene(), |event| {
        if !matches!(event, RuntimeEvent::Platform(_)) {
            println!("{event:?}");
        }
    })?;
    Ok(())
}
