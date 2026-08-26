#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::{
    platform::WindowConfig,
    render::RendererConfig,
    runtime::run_ui_with_text_engine,
    text::{FontFamily, TextColor, TextEngine, TextStyle},
    ui::{Align, Edges, Element, LayoutStyle, Length, UiTree},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(message: &str);
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
            Element::text("Unicode: café · العربية · שלום")
                .text_style(text_style(30.0, TextColor::rgb(1.0, 0.72, 0.38), 500)),
            Element::text("resize(browser) -> same_tree.new_layout()")
                .text_style(TextStyle {
                    family: FontFamily::Monospace,
                    ..text_style(18.0, TextColor::rgb(0.62, 0.92, 0.68), 500)
                }),
        ])
        .layout_style(panel)
        .gap(24.0)])
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
        |event| log(&format!("Argui: {event:?}")),
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
