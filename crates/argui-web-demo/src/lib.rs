#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::{
    platform::{ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig},
    render::RendererConfig,
    runtime::run_app_with_text_engine,
};
use argui_devtools::DevtoolsHost;
use argui_showcase::{StateShowcase, text_engine};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), JsValue> {
    run_app_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.state")
                    .map_err(|error| JsValue::from_str(&error.to_string()))?,
                "Argui state showcase",
                IconSet::new(),
            ),
            WindowConfig {
                title: "Argui state showcase".into(),
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        text_engine(),
        DevtoolsHost::new(StateShowcase::default()),
        |_| {},
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
