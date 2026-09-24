use argui_platform::{ApplicationConfig, PlatformError, WindowConfig};
use argui_render::RendererConfig;
use argui_text::TextEngine;
use argui_ui::UiTree;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_os = "android")]
use winit::platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp};

use crate::{
    AppModel, RuntimeError, RuntimeEvent, app::Application, event::UserEvent,
    multi::MultiApplication,
};

use super::native_event_loop;

/// Runs one native UI tree driven by batches from a JavaScript actor.
///
/// * `window` — native window configuration.
/// * `renderer` — renderer configuration.
/// * `host` — validated primitive graph initially owned by the UI thread.
/// * `assets` — decoded images and SVGs referenced by the host graph.
/// * `batches` — batches received from the JavaScript actor.
/// * `events` — callback deliveries posted back to that actor.
/// * `on_event` — observer for platform and renderer events.
///
/// # Errors
///
/// Returns an error if the native window or event loop cannot start or run.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_native_host(
    window: WindowConfig,
    renderer: RendererConfig,
    host: crate::NativeHost,
    assets: crate::NativeHostAssets,
    batches: std::sync::mpsc::Receiver<crate::NativeHostBatch>,
    events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    let event_loop = native_event_loop()?;
    run_native_host_with_event_loop(
        event_loop,
        NativeHostLaunch {
            window,
            renderer,
            text_engine: TextEngine::new(),
            host,
            assets,
            batches,
            events,
            on_event,
        },
    )
}

/// Runs a JavaScript presentation inside the application runtime, with native
/// tray, global shortcut, and close-policy controls.
///
/// `config` supplies application identity and initial windows; `renderer` configures
/// painting; `host` and `assets` provide the initial presentation; `channels`
/// exchanges changes with JavaScript; `app` handles tray and shortcut activations; `on_event` observes
/// runtime diagnostics and activations.
///
/// # Errors
/// Returns an error if the configuration, native event loop, or window fails.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_native_host_application(
    config: ApplicationConfig,
    renderer: RendererConfig,
    host: crate::NativeHost,
    assets: crate::NativeHostAssets,
    channels: NativeHostApplicationChannels,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    let NativeHostApplicationChannels {
        batches,
        events,
        requests,
    } = channels;
    let mut application = MultiApplication::new(config, renderer, app, on_event)?;
    application.install_native_host(host, assets, events);
    let event_loop = native_event_loop()?;
    let proxy = event_loop.create_proxy();
    application.set_event_proxy(proxy.clone());
    let request_proxy = proxy.clone();
    std::thread::spawn(move || {
        for batch in batches {
            if proxy.send_event(UserEvent::HostCommit(batch)).is_err() {
                return;
            }
        }
    });
    std::thread::spawn(move || {
        for request in requests {
            if request_proxy
                .send_event(UserEvent::NativeHostApplication(request))
                .is_err()
            {
                return;
            }
        }
    });
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.take().map_or(Ok(()), Err)
}

/// Channels connecting a JavaScript presentation to a native application.
#[cfg(not(target_arch = "wasm32"))]
pub struct NativeHostApplicationChannels {
    /// UI transaction batches produced by JavaScript.
    pub batches: std::sync::mpsc::Receiver<crate::NativeHostBatch>,
    /// UI callbacks delivered to JavaScript.
    pub events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
    /// Tray, shortcut, and window controls requested by JavaScript.
    pub requests: std::sync::mpsc::Receiver<crate::NativeHostApplicationRequest>,
}

/// Runs the native JavaScript presentation with embedded fonts on Android.
///
/// * `android_app` — activity handle supplied by Android NativeActivity.
/// * `window` — native window configuration.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text shaper with at least one installed Android font.
/// * `host` — validated primitive graph owned by the UI thread.
/// * `assets` — decoded images and SVGs referenced by the host graph.
/// * `batches` — batches received from the JavaScript actor.
/// * `events` — callback deliveries posted back to that actor.
/// * `on_event` — observer for platform and renderer events.
///
/// # Errors
///
/// Returns an error if Android initialization, the window, or event loop fails.
#[cfg(target_os = "android")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run_android_native_host_with_text_engine(
    android_app: AndroidApp,
    window: WindowConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    host: crate::NativeHost,
    assets: crate::NativeHostAssets,
    batches: std::sync::mpsc::Receiver<crate::NativeHostBatch>,
    events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    argui_platform::mobile::initialize_android(&android_app)
        .map_err(|error| RuntimeError::NativeHost(error.to_string()))?;
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    builder.with_android_app(android_app);
    let event_loop = builder.build().map_err(PlatformError::from)?;
    run_native_host_with_event_loop(
        event_loop,
        NativeHostLaunch {
            window,
            renderer,
            text_engine,
            host,
            assets,
            batches,
            events,
            on_event,
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeHostLaunch<F> {
    window: WindowConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    host: crate::NativeHost,
    assets: crate::NativeHostAssets,
    batches: std::sync::mpsc::Receiver<crate::NativeHostBatch>,
    events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
    on_event: F,
}

/// Connects an already configured native event loop to the JavaScript actor.
///
/// The event loop owns the UI thread; the batch forwarding thread exits when
/// the event loop closes.
///
/// * `event_loop` — platform-configured loop that owns the native window.
/// * `launch` — native host, renderer, assets, channels, fonts, and observer.
///
/// # Errors
///
/// Returns an error when the event loop fails or the native host reports a fatal error.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
fn run_native_host_with_event_loop<F: FnMut(RuntimeEvent) + 'static>(
    event_loop: EventLoop<UserEvent>,
    launch: NativeHostLaunch<F>,
) -> Result<(), RuntimeError> {
    let NativeHostLaunch {
        window,
        renderer,
        text_engine,
        host,
        assets,
        batches,
        events,
        on_event,
    } = launch;
    let initial = host.root_element().map(UiTree::new);
    let mut application =
        Application::new(window, renderer, text_engine, None, initial, None, on_event);
    application.install_native_host_assets(assets);
    application.native_host = Some(host);
    application.native_host_events = Some(events);
    let proxy = event_loop.create_proxy();
    application.set_event_proxy(proxy.clone());
    std::thread::spawn(move || {
        for batch in batches {
            if proxy.send_event(UserEvent::HostCommit(batch)).is_err() {
                break;
            }
        }
    });
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.take().map_or(Ok(()), Err)
}
