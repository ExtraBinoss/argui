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
    runtime::run_application_with_text_engine,
    text::TextEngine,
};
use argui_devtools::DevtoolsApp;

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const APP_ICON: &[u8] = include_bytes!("../../argui/examples/assets/astra-icon-256.png");

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let icons = IconSet::single(AppIcon::from_png(APP_ICON)?);
    run_application_with_text_engine(
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
                desktop_backdrop: cfg!(feature = "desktop-backdrop")
                    .then_some(argui::core::BackdropMaterial::Sidebar),
                ..WindowConfig::default()
            },
        ),
        argui_devtools::configure_renderer(
            RendererConfig::default().effects(
                argui_effects::registry()?
                    .with_definition(pages::scroll_effects::definition())?
                    .with_definition(pages::overlay_effects::definition())?,
            ),
        )?,
        text,
        DevtoolsApp::new(argui::runtime::SingleWindowModel::new(
            argui::widgets::TooltipHost::new(argui::widgets::SelectionHost::new(
                WidgetGallery::default(),
            )),
        )),
        |event| {
            use argui::runtime::{RuntimeEvent, WindowRuntimeEvent};
            let error = match event {
                RuntimeEvent::RendererFailed(message)
                | RuntimeEvent::LayoutFailed(message)
                | RuntimeEvent::CommandFailed(message) => Some(message),
                RuntimeEvent::Window {
                    event:
                        WindowRuntimeEvent::RendererFailed(message)
                        | WindowRuntimeEvent::LayoutFailed(message),
                    ..
                } => Some(message),
                _ => None,
            };
            if let Some(error) = error {
                #[cfg(target_arch = "wasm32")]
                browser_error(&error);
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("{error}");
            }
        },
    )?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn browser_error(message: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| browser_error(&info.to_string())));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
