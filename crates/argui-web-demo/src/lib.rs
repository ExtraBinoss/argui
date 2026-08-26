#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::{
    platform::WindowConfig,
    render::RendererConfig,
    runtime::{RuntimeEvent, run_app_with_text_engine},
    ui::UiEventKind,
};
use argui_showcase::{StateShowcase, text_engine};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(message: &str);
}

#[wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), JsValue> {
    run_app_with_text_engine(
        WindowConfig {
            title: "Argui state showcase".into(),
            ..WindowConfig::default()
        },
        RendererConfig::default(),
        text_engine(),
        StateShowcase::default(),
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
