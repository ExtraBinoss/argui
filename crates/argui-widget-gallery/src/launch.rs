use argui::{
    platform::{
        AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig,
    },
    render::RendererConfig,
    runtime::{RuntimeEvent, WindowRuntimeEvent, run_application_with_text_engine},
    text::TextEngine,
};
use argui_devtools::DevtoolsApp;

use crate::WidgetGallery;

type GalleryModel = DevtoolsApp<
    argui::runtime::SingleWindowModel<
        argui::widgets::TooltipHost<argui::widgets::SelectionHost<WidgetGallery>>,
    >,
>;

struct GalleryApplication {
    config: ApplicationConfig,
    renderer: RendererConfig,
    text: TextEngine,
    model: GalleryModel,
}

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf");
const APP_ICON: &[u8] = include_bytes!("../../argui/examples/assets/astra-icon-256.png");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn browser_error(message: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    fn renderer_state(state: &str);
}

fn application() -> Result<GalleryApplication, Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_ARABIC],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let icons = IconSet::single(AppIcon::from_png(APP_ICON)?);
    Ok(GalleryApplication {
        config: ApplicationConfig::new(
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
        renderer: argui_devtools::configure_renderer(
            RendererConfig::default().renderer_fallback(true).effects(
                argui_effects::registry()?
                    .with_definition(crate::pages::overlay_effects::definition())?,
            ),
        )?,
        text,
        model: DevtoolsApp::new(argui::runtime::SingleWindowModel::new(
            argui::widgets::TooltipHost::new(argui::widgets::SelectionHost::new(
                WidgetGallery::default(),
            )),
        )),
    })
}

fn handle_event(event: RuntimeEvent) {
    #[cfg(feature = "hot-reload")]
    if let RuntimeEvent::HotReloaded { generation } = &event {
        eprintln!("argui: applied hot reload generation {generation}");
    }
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
    #[cfg(not(target_arch = "wasm32"))]
    match &event {
        RuntimeEvent::RendererFallback(message) => {
            eprintln!("Argui renderer warning: {message}");
        }
        RuntimeEvent::DesktopBackdropUnavailable(reason) => {
            eprintln!("Argui desktop backdrop warning: {reason}.");
        }
        RuntimeEvent::Window {
            event: WindowRuntimeEvent::RendererFallback(message),
            ..
        } => eprintln!("Argui renderer warning: {message}"),
        RuntimeEvent::Window {
            event: WindowRuntimeEvent::DesktopBackdropUnavailable(reason),
            ..
        } => eprintln!("Argui desktop backdrop warning: {reason}."),
        _ => {}
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
        #[cfg(target_arch = "wasm32")]
        {
            renderer_state("error");
            browser_error(&error);
        }
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("{error}");
    }
}

/// Launches the native widget gallery with its DevTools inspection panel.
///
/// # Errors
///
/// Returns an error if application configuration or startup fails.
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let application = application()?;
    run_application_with_text_engine(
        application.config,
        application.renderer,
        application.text,
        application.model,
        handle_event,
    )?;
    Ok(())
}

#[cfg(target_os = "android")]
/// Launches the gallery using the supplied Android application handle.
///
/// # Errors
///
/// Returns an error if configuration or Android runtime startup fails.
pub fn launch_android(
    android_app: argui_android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    let application = application()?;
    argui_android::run_application_with_text_engine(
        android_app,
        application.config,
        application.renderer,
        application.text,
        application.model,
        handle_event,
    )?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
#[cfg_attr(coverage_nightly, coverage(off))]
/// Starts the gallery in a WebAssembly browser runtime.
///
/// # Errors
///
/// Returns a JavaScript error if browser runtime startup fails.
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        browser_error(&info.to_string());
    }));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
