//! WebAssembly entry points for the Argui state showcase.
#![cfg_attr(
    all(coverage_nightly, target_arch = "wasm32"),
    feature(coverage_attribute)
)]

use argui::runtime::SingleWindowModel;
use argui::{
    platform::{ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig},
    render::{RendererConfig, RendererError},
};
use argui_devtools::DevtoolsApp;
use argui_showcase::StateShowcase;

#[cfg(target_arch = "wasm32")]
use argui::runtime::run_application_with_text_engine;
#[cfg(target_arch = "wasm32")]
use argui_showcase::text_engine;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Builds the browser demo's application identity and main window.
///
/// # Errors
///
/// Returns an error if the application identity cannot be constructed.
pub fn application_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    Ok(ApplicationConfig::new(
        ApplicationIdentity::new(
            ApplicationId::new("dev.argui.state")?,
            "Argui state showcase",
            IconSet::new(),
        ),
        WindowConfig {
            title: "Argui state showcase".into(),
            ..WindowConfig::default()
        },
    ))
}

/// Adds the DevTools effects to the browser renderer.
///
/// # Errors
///
/// Returns a renderer error if DevTools effects cannot be configured.
pub fn renderer_config() -> Result<RendererConfig, RendererError> {
    argui_devtools::configure_renderer(RendererConfig::default())
}

/// Creates the state showcase inside the DevTools app.
/// Returns the showcase wrapped in a single-window model and DevTools host.
pub fn devtools_app() -> DevtoolsApp<SingleWindowModel<StateShowcase>> {
    DevtoolsApp::new(SingleWindowModel::new(StateShowcase::default()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
/// Starts the demo in the browser's WebAssembly runtime.
///
/// # Errors
///
/// Returns a JavaScript error if app configuration or runtime startup fails.
pub fn start() -> Result<(), JsValue> {
    run_application_with_text_engine(
        application_config().map_err(|error| JsValue::from_str(&error.to_string()))?,
        renderer_config().map_err(|error| JsValue::from_str(&error.to_string()))?,
        text_engine(),
        devtools_app(),
        |_| {},
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
