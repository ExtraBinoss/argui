#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use argui::runtime::SingleWindowModel;
use argui::{
    platform::{ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig},
    render::RendererConfig,
    runtime::run_application_with_text_engine,
};
use argui_devtools::DevtoolsApp;
use argui_showcase::{StateShowcase, text_engine};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), JsValue> {
    run_application_with_text_engine(
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
        argui_devtools::configure_renderer(RendererConfig::default())
            .map_err(|error| JsValue::from_str(&error.to_string()))?,
        text_engine(),
        DevtoolsApp::new(SingleWindowModel::new(StateShowcase::default())),
        |_| {},
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
