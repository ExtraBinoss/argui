//! Gallery WebAssembly host using the same browser bridge as generated Argui apps.

#[path = "../../../../crates/argui-cli/assets/hosts/web/src/delivery.rs"]
pub mod delivery;

#[cfg(target_arch = "wasm32")]
#[path = "../../../../crates/argui-cli/assets/hosts/web/src/web.rs"]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::ArguiWebHost;
