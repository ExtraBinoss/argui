use argui::{
    paint::{Border, Color, CornerRadii},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::run_ui,
    text::{FontFamily, TextColor, TextStyle},
    ui::{AlignItems, Axes, Element, LayoutStyle, Overflow, Sides, UiTree, length, percent, sides},
};

fn text_style(size: f32, color: TextColor, weight: u16) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size * 1.25,
        color,
        weight,
        ..TextStyle::default()
    }
}

fn showcase() -> UiTree {
    let mut panel_style = LayoutStyle::default();
    panel_style.size.width = percent(1.0);
    panel_style.max_size.width = length(820.0);
    UiTree::new(
        Element::column([Element::column([
            Element::text("Argui responsive layout")
                .text_style(text_style(46.0, TextColor::srgb(0.35, 0.85, 1.0), 700)),
            Element::text(
                "This text is measured by Cosmic Text and constrained by Taffy. Resize the window: the line wrapping and every logical rectangle are recomputed from the same retained Rust tree.",
            )
            .text_style(text_style(22.0, TextColor::WHITE, 400)),
            Element::text("GPU INSTANCED · ZERO WEBVIEW")
                .text_style(text_style(14.0, TextColor::srgb(0.7, 0.95, 0.78), 700))
                .padding(sides(12.0, 7.0))
                .background(Color::srgb(0.08, 0.22, 0.16))
                .border(Border::all(1.0, Color::srgb(0.18, 0.5, 0.34)))
                .radius(CornerRadii::all(10.0)),
            Element::text("retained tree  →  Taffy  →  Cosmic Text  →  WGPU")
                .text_style(TextStyle {
                    family: FontFamily::Monospace,
                    ..text_style(18.0, TextColor::srgb(0.62, 0.92, 0.68), 500)
                }),
        ])
        .layout_style(panel_style)
        .padding(Sides::length(30.0))
        .gap(24.0)
        .background(Color::srgb(0.075, 0.095, 0.135))
        .border(Border::all(1.5, Color::srgb(0.18, 0.28, 0.4)))
        .radius(CornerRadii::all(22.0))
        .overflow(Axes { x: Overflow::Hidden, y: Overflow::Hidden })])
        .width(percent(1.0))
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .padding(sides(42.0, 36.0)),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_ui(
        WindowConfig {
            title: "Argui responsive UI".into(),
            width: 1100.0,
            height: 700.0,
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        showcase(),
        |event| println!("{event:?}"),
    )?;
    Ok(())
}
