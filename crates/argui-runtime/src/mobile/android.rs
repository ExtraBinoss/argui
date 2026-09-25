//! Android `NativeActivity` bootstrap for an Argui application.
//!
//! The application owns the final `cdylib` and supplies its model and assets.
//! This module passes Android's activity handle to Winit and runs the regular
//! Argui runtime. No Java or Kotlin UI is required.

/// Whether the current compilation target is Android.
#[must_use]
pub fn is_android() -> bool {
    cfg!(target_os = "android")
}

/// Generate the unmangled `android_main` symbol required by `NativeActivity`.
///
/// The supplied function receives `AndroidApp` and returns `Result<(), E>`,
/// where `E` implements `Display`. Android's glue catches a panic at this FFI
/// boundary and reports it to logcat.
///
/// # Panics
/// The generated entry point panics when the launch function returns an error.
#[macro_export]
macro_rules! android_main {
    ($launch:path) => {
        #[cfg(target_os = "android")]
        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        fn android_main(app: $crate::mobile::android::AndroidApp) {
            if let Err(error) = $launch(app) {
                panic!("Argui Android launch failed: {error}");
            }
        }
    };
}

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
use {
    crate::{AppModel, RuntimeError, RuntimeEvent},
    argui_platform::ApplicationConfig,
    argui_render::RendererConfig,
    argui_text::TextEngine,
};

/// Runs an Argui model from Android's `android_main` callback.
///
/// # Arguments
/// * `android_app` — activity handle supplied by Android.
/// * `config` — platform identity, window, tray, and preference settings.
/// * `renderer` — GPU renderer configuration.
/// * `app` — application model.
/// * `on_event` — callback for runtime events.
///
/// # Errors
/// Returns an error if runtime setup or application execution fails.
#[cfg(target_os = "android")]
pub fn run_application(
    android_app: AndroidApp,
    config: ApplicationConfig,
    renderer: RendererConfig,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    crate::run_android_application(android_app, config, renderer, app, on_event)
}

/// Runs an Argui model with an application-owned text engine on Android.
///
/// # Arguments
/// * `android_app` — activity handle supplied by Android.
/// * `config` — platform identity, window, tray, and preference settings.
/// * `renderer` — GPU renderer configuration.
/// * `text_engine` — text engine used for shaping and rasterization.
/// * `app` — application model.
/// * `on_event` — callback for runtime events.
///
/// # Errors
/// Returns an error if runtime setup or application execution fails.
#[cfg(target_os = "android")]
pub fn run_application_with_text_engine(
    android_app: AndroidApp,
    config: ApplicationConfig,
    renderer: RendererConfig,
    text_engine: TextEngine,
    app: impl AppModel,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    crate::run_android_application_with_text_engine(
        android_app,
        config,
        renderer,
        text_engine,
        app,
        on_event,
    )
}
