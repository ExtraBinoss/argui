//! Argui command line entry point.

/// Dispatches command line arguments and reports a concise error on failure.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(error) = argui_cli::run(std::env::args().skip(1).collect()) {
        eprintln!("argui: {error}");
        std::process::exit(1);
    }
}

/// Keeps the workspace WebAssembly compile gate valid; the CLI runs on desktop.
#[cfg(target_arch = "wasm32")]
fn main() {}
