//! Astra Editor, a product-shaped Argui application example.

mod app;
mod syntax;
mod ui;
pub mod workspace;

pub use app::AstraEditor;
use argui::{
    core::ColorScheme,
    platform::{
        AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet,
        PreferenceOverrides, WindowConfig,
    },
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
const FIRA_MONO: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/FiraMono-Medium.ttf");
const APP_ICON: &[u8] = include_bytes!("../../../crates/argui/examples/assets/astra-icon-256.png");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    /// Notifies the host page that the renderer became ready or failed.
    fn renderer_state(state: &str);
}

/// Publishes renderer readiness and reports runtime failures.
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

/// Starts Astra Editor as a native application or full-page WebAssembly canvas.
///
/// # Errors
///
/// Returns an error if the application identity, icon, renderer, or window runtime
/// cannot be initialized.
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_ARABIC, NOTO_HEBREW, NOTO_EMOJI, FIRA_MONO],
        "Noto Sans",
        "Noto Sans",
        "Fira Mono",
    );
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.astra-editor")?,
        "Astra Editor",
        IconSet::single(AppIcon::from_png(APP_ICON)?),
    );
    let config = ApplicationConfig::new(
        identity,
        WindowConfig {
            title: "Astra Editor · Rust workspace".into(),
            width: 1360.0,
            height: 860.0,
            focus_on_launch: true,
            ..WindowConfig::default()
        },
    )
    .with_preferences(PreferenceOverrides {
        color_scheme: Some(ColorScheme::Light),
        ..PreferenceOverrides::default()
    });
    run_application_with_text_engine(
        config,
        RendererConfig::default()
            .renderer_fallback(true)
            .effects(argui_effects::registry()?),
        text,
        SingleWindowModel::new(argui::widgets::TooltipHost::new(
            argui::widgets::SelectionHost::new(AstraEditor::default()),
        )),
        handle_event,
    )?;
    Ok(())
}

/// Starts the editor automatically when the WebAssembly module is loaded.
///
/// # Errors
///
/// Returns a JavaScript error when application startup fails.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        eprintln!("{info}");
    }));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
