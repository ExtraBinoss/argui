//! Native and WebAssembly entry points for the Argui GPU Canvas Lab.

mod app;
mod gpu;
pub mod state;

use std::sync::Arc;

use app::GpuCanvasLab;
use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::{GpuCanvasDiagnosticKind, GpuCanvasRegistration, GpuCanvasRegistry, RendererConfig},
    runtime::{
        RuntimeEvent, SingleWindowModel, WindowRuntimeEvent, run_application_with_text_engine,
    },
    text::TextEngine,
};
use gpu::LabCanvasFactory;
use state::SharedLab;

const NOTO_SANS: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    fn renderer_state(state: &str);
}

/// Records renderer and GPU-canvas events for the in-app diagnostic panel.
fn handle_event(shared: &SharedLab, event: RuntimeEvent) {
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
    let message = match event {
        RuntimeEvent::GpuCanvasFailed(diagnostic)
        | RuntimeEvent::Window {
            event: WindowRuntimeEvent::GpuCanvasFailed(diagnostic),
            ..
        } => {
            debug_assert_eq!(diagnostic.kind, GpuCanvasDiagnosticKind::Failed);
            Some(diagnostic.message)
        }
        RuntimeEvent::GpuCanvasRecovered(diagnostic)
        | RuntimeEvent::Window {
            event: WindowRuntimeEvent::GpuCanvasRecovered(diagnostic),
            ..
        } => {
            debug_assert_eq!(diagnostic.kind, GpuCanvasDiagnosticKind::Recovered);
            Some(diagnostic.message)
        }
        RuntimeEvent::RendererFallback(message) => Some(format!("Renderer fallback: {message}")),
        RuntimeEvent::RendererFailed(message)
        | RuntimeEvent::LayoutFailed(message)
        | RuntimeEvent::CommandFailed(message)
        | RuntimeEvent::Window {
            event:
                WindowRuntimeEvent::RendererFailed(message) | WindowRuntimeEvent::LayoutFailed(message),
            ..
        } => {
            #[cfg(target_arch = "wasm32")]
            renderer_state("error");
            Some(message)
        }
        _ => None,
    };
    if let Some(message) = message {
        eprintln!("{message}");
        shared.set_diagnostic(message);
    }
}

/// Starts the browser build and reports initialization failures to JavaScript.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        eprintln!("{info}");
    }));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}

/// Launches the GPU Canvas Lab with one immutable canvas registration.
///
/// # Errors
///
/// Returns registry validation or platform/runtime startup failures.
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let shared = Arc::new(SharedLab::default());
    let registration = GpuCanvasRegistration::new(
        "example.gpu-canvas-lab",
        LabCanvasFactory::new(Arc::clone(&shared)),
    );
    let canvas = registration.id();
    let registry = GpuCanvasRegistry::new([registration])?;
    let renderer = RendererConfig::default()
        .gpu_canvas_cache_bytes(96 * 1024 * 1024)
        .gpu_canvases(registry);
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let event_state = Arc::clone(&shared);
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui GPU Canvas Lab"),
            WindowConfig {
                title: "Argui GPU Canvas Lab".into(),
                width: 1280.0,
                height: 780.0,
                ..WindowConfig::default()
            },
        ),
        renderer,
        text,
        SingleWindowModel::new(GpuCanvasLab::new(canvas, shared)),
        move |event| handle_event(&event_state, event),
    )?;
    Ok(())
}
