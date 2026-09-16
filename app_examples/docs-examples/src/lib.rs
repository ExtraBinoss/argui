pub mod examples;

use argui::{
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::RendererConfig,
    runtime::{
        Render, RuntimeEvent, SingleWindowModel, WindowRuntimeEvent,
        run_application_with_text_engine,
    },
    text::TextEngine,
};

const NOTO_SANS: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    fn renderer_state(state: &str);
}

fn events(event: RuntimeEvent) {
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
    if matches!(
        event,
        RuntimeEvent::RendererFailed(_)
            | RuntimeEvent::LayoutFailed(_)
            | RuntimeEvent::CommandFailed(_)
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::RendererFailed(_) | WindowRuntimeEvent::LayoutFailed(_),
                ..
            }
    ) {
        #[cfg(target_arch = "wasm32")]
        renderer_state("error");
    }
}

fn launch<M: Render + 'static>(title: &str, model: M) -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development(title),
            WindowConfig {
                title: title.into(),
                width: 820.0,
                height: 520.0,
                ..WindowConfig::default()
            },
        ),
        RendererConfig::default(),
        text,
        SingleWindowModel::new(model),
        events,
    )?;
    Ok(())
}

pub fn launch_example(example: &str) -> Result<(), Box<dyn std::error::Error>> {
    match example {
        "installation" => launch("Install Argui", examples::installation::Example),
        "first-window" => launch("First window", examples::first_window::Example),
        "elements" => launch("Compose elements", examples::elements::Example),
        "counter" => launch("Counter", examples::counter::Example::default()),
        "layout" => launch("Responsive layout", examples::layout::Example::default()),
        "events" => launch("Events", examples::events::Example::default()),
        "styling" => launch("Styling", examples::styling::Example::default()),
        "accessibility" => launch("Accessibility", examples::accessibility::Example::default()),
        "tasks" => launch("Tasks", examples::tasks::Example::default()),
        "data" => launch("Data", examples::data::Example::default()),
        "overlays" => launch("Overlays", examples::overlays::Example::default()),
        "platform-support" => launch("Platform support", examples::platform_support::Example),
        "platform-roadmap" => launch(
            "Platform roadmap",
            examples::platform_roadmap::Example::default(),
        ),
        "animation" => launch("Animation", examples::animation::Example::default()),
        "i18n" => launch("Internationalization", examples::i18n::Example::default()),
        "mental-model" => launch("Mental model", examples::mental_model::Example),
        "project-structure" => launch("Project structure", examples::project_structure::Example),
        "clean-code" => launch("Clean code", examples::clean_code::Example::default()),
        "custom-elements" => launch("Custom elements", examples::custom_elements::Example),
        _ => Err(format!("unknown documentation example: {example}").into()),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run(example: &str) -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        eprintln!("{info}");
    }));
    launch_example(example).map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
