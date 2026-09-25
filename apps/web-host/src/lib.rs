//! Browser bridge between Solid or React presentation operations and Argui WASM.

pub mod delivery;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::ArguiWebHost;
