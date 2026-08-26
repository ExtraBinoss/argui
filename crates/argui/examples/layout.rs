use argui::{
    paint::{Border, ClipBehavior, Color, CornerRadii},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::run_ui,
    text::{FontFamily, TextColor, TextStyle},
    ui::{Align, Edges, Element, LayoutStyle, Length, UiTree},
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
    let panel_style = LayoutStyle {
        width: Length::Percent(1.0),
        max_width: Length::Px(820.0),
        ..LayoutStyle::default()
    };
    UiTree::new(
        Element::column([Element::column([
            Element::text("Argui responsive layout")
                .text_style(text_style(46.0, TextColor::rgb(0.35, 0.85, 1.0), 700)),
            Element::text(
                "This text is measured by Cosmic Text and constrained by Taffy. Resize the window: the line wrapping and every logical rectangle are recomputed from the same retained Rust tree.",
            )
            .text_style(text_style(22.0, TextColor::WHITE, 400)),
            Element::text("GPU INSTANCED · ZERO WEBVIEW")
                .text_style(text_style(14.0, TextColor::rgb(0.7, 0.95, 0.78), 700))
                .padding(Edges::symmetric(12.0, 7.0))
                .background(Color::rgb(0.08, 0.22, 0.16))
                .border(Border::all(1.0, Color::rgb(0.18, 0.5, 0.34)))
                .radius(CornerRadii::all(10.0)),
            Element::text("retained tree  →  Taffy  →  Cosmic Text  →  WGPU")
                .text_style(TextStyle {
                    family: FontFamily::Monospace,
                    ..text_style(18.0, TextColor::rgb(0.62, 0.92, 0.68), 500)
                }),
        ])
        .layout_style(panel_style)
        .padding(Edges::all(30.0))
        .gap(24.0)
        .background(Color::rgb(0.075, 0.095, 0.135))
        .border(Border::all(1.5, Color::rgb(0.18, 0.28, 0.4)))
        .radius(CornerRadii::all(22.0))
        .clip(ClipBehavior::Bounds)])
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .align(Align::Center)
        .padding(Edges::symmetric(42.0, 36.0)),
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
