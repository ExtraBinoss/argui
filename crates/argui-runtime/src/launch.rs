use argui_platform::{ApplicationConfig, PlatformError, WindowConfig};
use argui_render::RendererConfig;
use argui_text::{TextEngine, TextScene};
use argui_ui::UiTree;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{
    AppModel, Render, RuntimeError, RuntimeEvent, app::Application, application::SingleWindowModel,
    event::UserEvent, multi::MultiApplication,
};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_application(
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch_multi(MultiApplication::new(config, renderer, app, on_event)?)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_application_with_text_engine(
    config: ApplicationConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch_multi(MultiApplication::new_with_text_engine(
        config,
        renderer,
        Some(text_engine),
        app,
        on_event,
    )?)
}

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
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl Render,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_application(config, renderer, SingleWindowModel::new(app), on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_app_with_text_engine(
    config: ApplicationConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    app: impl Render,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_application_with_text_engine(
        config,
        renderer,
        text_engine,
        SingleWindowModel::new(app),
        on_event,
    )
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
    application.fatal_error.take().map_or(Ok(()), Err)
}

#[cfg(not(target_arch = "wasm32"))]
fn launch_multi(mut application: MultiApplication) -> Result<(), RuntimeError> {
    #[cfg(all(feature = "webview", target_os = "linux"))]
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var("GDK_BACKEND").map_or(true, |backend| backend != "x11")
    {
        return crate::multi::gtk::launch(application);
    }
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(PlatformError::from)?;
    application.set_event_proxy(event_loop.create_proxy());
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.take().map_or(Ok(()), Err)
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

#[cfg(target_arch = "wasm32")]
fn launch_multi(mut application: MultiApplication) -> Result<(), RuntimeError> {
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .map_err(PlatformError::from)?;
    application.set_event_proxy(event_loop.create_proxy());
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.spawn_app(application);
    Ok(())
}
