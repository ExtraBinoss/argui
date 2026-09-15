//! iOS static-library bootstrap for an Argui application.
//!
//! The application supplies one Rust launch function and links the resulting
//! static library into its Xcode target. Winit owns the UIKit window internally;
//! application views remain entirely in Argui and WGPU.

/// Whether the current compilation target is iOS.
#[must_use]
pub fn is_ios() -> bool {
    cfg!(target_os = "ios")
}

/// Export an Objective-C-callable function that starts an Argui application.
///
/// The launch function takes no arguments and returns `Result<(), E>`, where
/// `E` implements `Display`. Pass the generated function name to Xcode's `main`.
///
/// # Panics
/// The generated entry point panics when the launch function returns an error.
#[macro_export]
macro_rules! ios_main {
    ($name:ident, $launch:path) => {
        #[cfg(target_os = "ios")]
        #[allow(unsafe_code)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() {
            if let Err(error) = $launch() {
                panic!("Argui iOS launch failed: {error}");
            }
        }
    };
}

#[cfg(target_os = "ios")]
pub use argui_runtime::{
    run, run_app, run_app_with_text_engine, run_application, run_application_with_text_engine,
    run_ui, run_ui_with_text_engine, run_with_text, run_with_text_engine,
};
