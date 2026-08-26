#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::{
    paint::{Border, ClipBehavior, Color, CornerRadii, PaintStyle, QuadStyle},
    platform::WindowConfig,
    render::RendererConfig,
    runtime::{RuntimeEvent, run_ui_with_text_engine},
    text::{FontFamily, TextColor, TextEngine, TextStyle},
    ui::{
        Align, Button, ButtonStyle, Edges, Element, LayoutStyle, Length, UiEventKind, UiTree, Wrap,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(message: &str);
}

fn button(key: &str, label: &str, accent: Color) -> Element {
    let radius = CornerRadii::all(12.0);
    let rest = QuadStyle::solid(Color::rgb(0.10, 0.14, 0.20))
        .border(Border::all(1.0, Color::rgb(0.24, 0.32, 0.44)))
        .radius(radius);
    let hovered = QuadStyle::solid(Color::rgb(0.14, 0.22, 0.30))
        .border(Border::all(1.0, accent))
        .radius(radius);
    let pressed = QuadStyle::solid(accent)
        .border(Border::all(1.0, accent))
        .radius(radius)
        .opacity(0.82);
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(rest),
            text_style(17.0, TextColor::WHITE, 600),
        )
        .hovered(hovered)
        .pressed(pressed)
        .focused(rest.border(Border::all(2.0, accent))),
    )
    .build()
}

fn text_style(size: f32, color: TextColor, weight: u16) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size * 1.25,
        color,
        weight,
        ..TextStyle::default()
    }
}

fn ui() -> UiTree {
    let panel = LayoutStyle {
        width: Length::Percent(1.0),
        max_width: Length::Px(820.0),
        ..LayoutStyle::default()
    };
    UiTree::new(
        Element::column([Element::column([
            Element::text("Argui on WebGPU")
                .text_style(text_style(48.0, TextColor::rgb(0.35, 0.85, 1.0), 700)),
            Element::text(
                "One retained Rust tree, responsive Taffy layout, Cosmic Text and WGPU — no WebView, no TypeScript UI.",
            )
            .text_style(text_style(22.0, TextColor::WHITE, 400)),
            Element::text("GPU INSTANCED · ZERO WEBVIEW")
                .text_style(text_style(14.0, TextColor::rgb(0.7, 0.95, 0.78), 700))
                .padding(Edges::symmetric(12.0, 7.0))
                .background(Color::rgb(0.08, 0.22, 0.16))
                .border(Border::all(1.0, Color::rgb(0.18, 0.5, 0.34)))
                .radius(CornerRadii::all(10.0)),
            Element::row([
                button("web-primary", "Primary", Color::rgb(0.15, 0.62, 0.92)),
                button("web-confirm", "Confirm", Color::rgb(0.22, 0.72, 0.46)),
                button("web-danger", "Delete", Color::rgb(0.92, 0.30, 0.36)),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::text("Unicode: café · العربية · שלום")
                .text_style(text_style(30.0, TextColor::rgb(1.0, 0.72, 0.38), 500)),
            Element::text("resize(browser) -> same_tree.new_layout()")
                .text_style(TextStyle {
                    family: FontFamily::Monospace,
                    ..text_style(18.0, TextColor::rgb(0.62, 0.92, 0.68), 500)
                }),
        ])
        .layout_style(panel)
        .padding(Edges::all(30.0))
        .gap(24.0)
        .background(Color::rgb(0.075, 0.095, 0.135))
        .border(Border::all(1.5, Color::rgb(0.18, 0.28, 0.4)))
        .radius(CornerRadii {
            top_left: 28.0,
            top_right: 12.0,
            bottom_right: 28.0,
            bottom_left: 12.0,
        })
        .clip(ClipBehavior::Bounds)])
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .align(Align::Center)
        .padding(Edges::symmetric(42.0, 36.0)),
    )
}

#[wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), JsValue> {
    let text_engine = TextEngine::from_embedded_fonts(
        [
            include_bytes!("../assets/fonts/NotoSans-Regular.ttf").as_slice(),
            include_bytes!("../assets/fonts/NotoSansArabic.ttf").as_slice(),
            include_bytes!("../assets/fonts/NotoSansHebrew.ttf").as_slice(),
            include_bytes!("../assets/fonts/FiraMono-Medium.ttf").as_slice(),
        ],
        "Noto Sans",
        "Noto Sans",
        "Fira Mono",
    );

    run_ui_with_text_engine(
        WindowConfig {
            title: "Argui WebGPU".into(),
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        text_engine,
        ui(),
        |event| {
            if let RuntimeEvent::Ui(ui) = &event
                && ui.kind == UiEventKind::Clicked
            {
                log(&format!("Argui clicked: {:?}", ui.key));
            }
            log(&format!("Argui: {event:?}"));
        },
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
