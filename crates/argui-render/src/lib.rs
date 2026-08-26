#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod batch;
mod config;
mod effect;
mod effect_graph;
mod error;
mod offscreen;
mod quad;
mod surface;
mod text;

pub use config::RendererConfig;
pub use effect_graph::EffectGraphStats;
pub use error::RendererError;
pub use offscreen::TexturePoolStats;
pub use surface::{RenderStatus, SurfaceRenderer};
