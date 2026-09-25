use argui_platform::{ApplicationConfig, PlatformError, WindowConfig};
use argui_render::RendererConfig;
use argui_text::{TextEngine, TextScene};
use argui_ui::UiTree;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_os = "android")]
use winit::platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp};

use crate::{
    AppModel, Render, RuntimeError, RuntimeEvent, app::Application, application::SingleWindowModel,
    event::UserEvent, multi::MultiApplication,
};

#[cfg(not(target_arch = "wasm32"))]
mod native_host;
#[cfg(target_arch = "wasm32")]
mod web_host;
#[cfg(target_os = "android")]
pub use native_host::run_android_native_host_with_text_engine;
#[cfg(not(target_arch = "wasm32"))]
pub use native_host::{
    NativeHostApplicationChannels, run_native_host, run_native_host_application,
};
#[cfg(target_arch = "wasm32")]
pub use web_host::WebHostHandle;

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a multi-window application with the default text engine.
///
/// # Arguments
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration shared by the application.
/// * `app` — application model that supplies windows and handles events.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the platform event loop or application host cannot be initialized or run.
pub fn run_application(
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    launch_multi(MultiApplication::new(config, renderer, app, on_event)?)
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a multi-window application using the supplied text engine.
///
/// # Arguments
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration shared by the application.
/// * `text_engine` — text engine used to shape and render application text.
/// * `app` — application model that supplies windows and handles events.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the application host or platform event loop cannot be initialized or run.
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

/// Run an application from Android's `android_main` entry point.
///
/// # Arguments
/// * `android_app` — Android activity handle provided by the platform entry point.
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration.
/// * `app` — application model that supplies windows and handles events.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the event loop or application host cannot be initialized or run.
#[cfg(target_os = "android")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_android_application(
    android_app: AndroidApp,
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_android_application_with_text_engine(
        android_app,
        config,
        renderer,
        TextEngine::new(),
        app,
        on_event,
    )
}

/// Run an application with embedded fonts from Android's `android_main` entry point.
///
/// # Arguments
/// * `android_app` — Android activity handle provided by the platform entry point.
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text engine used by the application.
/// * `app` — application model that supplies windows and handles events.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the event loop or application host cannot be initialized or run.
#[cfg(target_os = "android")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_android_application_with_text_engine(
    android_app: AndroidApp,
    config: ApplicationConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    let application =
        MultiApplication::new_with_text_engine(config, renderer, Some(text_engine), app, on_event)?;
    launch_android(android_app, application)
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a window using the platform's default text engine.
///
/// # Arguments
/// * `window` — configuration for the window to create.
/// * `renderer` — GPU renderer configuration.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the event loop or window cannot be initialized or run.
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
/// Runs a window that displays only a text scene using the default text engine.
///
/// # Arguments
/// * `window` — configuration for the window to create.
/// * `renderer` — GPU renderer configuration.
/// * `scene` — text scene to display.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the platform event loop or window cannot be initialized or run.
pub fn run_with_text(
    window: WindowConfig,
    renderer: RendererConfig,
    scene: TextScene,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_with_text_engine(window, renderer, TextEngine::new(), scene, on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a window that displays a text scene with the supplied text engine.
///
/// # Arguments
/// * `window` — configuration for the window to create.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text engine used to shape and render the scene.
/// * `scene` — text scene to display.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the platform event loop or window cannot be initialized or run.
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
/// Runs a window displaying a UI tree with the default text engine.
///
/// # Arguments
/// * `window` — configuration for the window to create.
/// * `renderer` — GPU renderer configuration.
/// * `ui` — UI tree to display.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the platform event loop or window cannot be initialized or run.
pub fn run_ui(
    window: WindowConfig,
    renderer: RendererConfig,
    ui: UiTree,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_ui_with_text_engine(window, renderer, TextEngine::new(), ui, on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a window displaying a UI tree with the supplied text engine.
///
/// # Arguments
/// * `window` — configuration for the window to create.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text engine used to shape and render UI text.
/// * `ui` — UI tree to display.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the platform event loop or window cannot be initialized or run.
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
/// Runs a single-window render model using the default text engine.
///
/// # Arguments
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration.
/// * `app` — model that renders the window contents.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the application host or platform event loop cannot be initialized or run.
pub fn run_app(
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl Render,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    run_application(config, renderer, SingleWindowModel::new(app), on_event)
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs a single-window render model using the supplied text engine.
///
/// # Arguments
/// * `config` — application and window configuration.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text engine used to render text in the window.
/// * `app` — model that renders the window contents.
/// * `on_event` — callback for runtime events emitted by the host.
///
/// # Errors
/// Returns an error if the application host or platform event loop cannot be initialized or run.
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
    let event_loop = native_event_loop()?;
    let proxy = event_loop.create_proxy();
    application.set_event_proxy(proxy);
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.take().map_or(Ok(()), Err)
}

#[cfg(not(target_arch = "wasm32"))]
fn launch_multi(application: MultiApplication) -> Result<(), RuntimeError> {
    #[cfg(all(feature = "webview", target_os = "linux"))]
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var("DISPLAY")
            .ok()
            .is_none_or(|display| display.is_empty())
        && std::env::var("GDK_BACKEND").map_or(true, |backend| backend != "x11")
    {
        return crate::multi::gtk::launch(application);
    }
    let event_loop = native_event_loop()?;
    run_event_loop(event_loop, application)
}

/// Builds a native event loop, choosing X11 when a display is available.
///
/// Wayland remains the fallback on Linux without `DISPLAY`; other platforms
/// retain winit's native backend selection.
///
/// # Errors
/// Returns a platform error if the selected event loop cannot be created.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
fn native_event_loop() -> Result<EventLoop<UserEvent>, PlatformError> {
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "linux")]
    if std::env::var("DISPLAY").is_ok_and(|display| !display.is_empty()) {
        use winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_x11();
    }
    builder.build().map_err(PlatformError::from)
}

#[cfg(target_os = "android")]
fn launch_android(
    android_app: AndroidApp,
    application: MultiApplication,
) -> Result<(), RuntimeError> {
    argui_platform::mobile::initialize_android(&android_app)
        .map_err(|error| RuntimeError::NativeHost(error.to_string()))?;
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    builder.with_android_app(android_app);
    let event_loop = builder.build().map_err(PlatformError::from)?;
    run_event_loop(event_loop, application)
}

#[cfg(not(target_arch = "wasm32"))]
fn run_event_loop(
    event_loop: EventLoop<UserEvent>,
    mut application: MultiApplication,
) -> Result<(), RuntimeError> {
    let proxy = event_loop.create_proxy();
    application.set_event_proxy(proxy);
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
