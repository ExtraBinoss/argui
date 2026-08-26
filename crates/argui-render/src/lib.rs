#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod batch;
mod config;
mod error;
mod quad;
mod surface;
mod text;

pub use config::RendererConfig;
pub use error::RendererError;
pub use surface::{RenderStatus, SurfaceRenderer};
