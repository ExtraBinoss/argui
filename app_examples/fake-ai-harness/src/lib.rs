mod app;
pub mod fake_model;

use app::AiHarness;
use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::RendererConfig,
    runtime::{
        RuntimeEvent, SingleWindowModel, WindowRuntimeEvent, run_application_with_text_engine,
    },
    text::TextEngine,
};

const NOTO_SANS: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSansArabic.ttf");
const NOTO_HEBREW: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSansHebrew.ttf");
const NOTO_EMOJI: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoEmoji-Regular.ttf");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    fn renderer_state(state: &str);
}

fn handle_event(event: RuntimeEvent) {
    #[cfg(target_arch = "wasm32")]
    if matches!(
        &event,
        RuntimeEvent::RendererReady
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::RendererReady,
                ..
            }
    ) {
        renderer_state("ready");
    }
    let error = match event {
        RuntimeEvent::RendererFailed(message)
        | RuntimeEvent::LayoutFailed(message)
        | RuntimeEvent::CommandFailed(message) => Some(message),
        RuntimeEvent::Window {
            event:
                WindowRuntimeEvent::RendererFailed(message) | WindowRuntimeEvent::LayoutFailed(message),
            ..
        } => Some(message),
        _ => None,
    };
    if let Some(error) = error {
        eprintln!("{error}");
        #[cfg(target_arch = "wasm32")]
        renderer_state("error");
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        eprintln!("{info}");
    }));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_ARABIC, NOTO_HEBREW, NOTO_EMOJI],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui AI Harness"),
            WindowConfig {
                title: "Argui AI Harness · 1,000 tok/s".into(),
                width: 1320.0,
                height: 820.0,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        text,
        SingleWindowModel::new(argui::widgets::TooltipHost::new(
            argui::widgets::SelectionHost::new(AiHarness::default()),
        )),
        handle_event,
    )?;
    Ok(())
}
