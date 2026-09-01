#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod app;
mod navigation;
mod numeric_expression;
mod pages;
mod property_slider;

pub use app::WidgetGallery;
use argui::{
    platform::{
        AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig,
    },
    render::RendererConfig,
    runtime::run_app_with_text_engine,
    text::TextEngine,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const APP_ICON: &[u8] = include_bytes!("../../argui/examples/assets/astra-icon-256.png");

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let icons = IconSet::single(AppIcon::from_png(APP_ICON)?);
    run_app_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.widgets")?,
                "Argui Widget Gallery",
                icons,
            ),
            WindowConfig {
                title: "Argui Widget Gallery".into(),
                width: 1220.0,
                height: 780.0,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default().effects(argui_effects::registry()?),
        text,
        WidgetGallery::default(),
        |_| {},
    )?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
