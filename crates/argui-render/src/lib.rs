#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod config;
mod error;
mod surface;

pub use config::RendererConfig;
pub use error::RendererError;
pub use surface::{RenderStatus, SurfaceRenderer};
