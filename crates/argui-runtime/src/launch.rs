use argui_platform::{PlatformError, WindowConfig};
use argui_render::RendererConfig;
use argui_text::{TextEngine, TextScene};
use argui_ui::UiTree;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{RuntimeError, RuntimeEvent, UiApp, app::Application, event::UserEvent};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run(
    window: WindowConfig,
    renderer: RendererConfig,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch(Application::new(
        window,
        renderer,
        TextEngine::new(),
        None,
        None,
        None,
        on_event,
    ))
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_with_text(
    window: WindowConfig,
    renderer: RendererConfig,
    scene: TextScene,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_with_text_engine(window, renderer, TextEngine::new(), scene, on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_with_text_engine(
    window: WindowConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    scene: TextScene,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch(Application::new(
        window,
        renderer,
        text_engine,
        Some(scene),
        None,
        None,
        on_event,
    ))
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_ui(
    window: WindowConfig,
    renderer: RendererConfig,
    ui: UiTree,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_ui_with_text_engine(window, renderer, TextEngine::new(), ui, on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_ui_with_text_engine(
    window: WindowConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    ui: UiTree,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch(Application::new(
        window,
        renderer,
        text_engine,
        None,
        Some(ui),
        None,
        on_event,
    ))
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_app(
    window: WindowConfig,
    renderer: RendererConfig,
    app: impl UiApp,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_app_with_text_engine(window, renderer, TextEngine::new(), app, on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_app_with_text_engine(
    window: WindowConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    app: impl UiApp,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    let ui = UiTree::new(app.view());
    launch(Application::new(
        window,
        renderer,
        text_engine,
        None,
        Some(ui),
        Some(Box::new(app)),
        on_event,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn launch(mut application: Application) -> Result<(), RuntimeError> {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(PlatformError::from)?;
    application.set_event_proxy(event_loop.create_proxy());
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.map_or(Ok(()), Err)
}

#[cfg(target_arch = "wasm32")]
fn launch(mut application: Application) -> Result<(), RuntimeError> {
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(PlatformError::from)?;
    application.set_event_proxy(event_loop.create_proxy());
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.spawn_app(application);
    Ok(())
}
