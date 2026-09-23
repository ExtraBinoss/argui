//! Desktop entry point for the embedded Solid gallery.

/// Runs the desktop gallery until its native window closes.
///
/// # Errors
/// Returns an error if gallery startup or the native event loop fails.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    argui_gallery_quickjs::run_desktop()
}
