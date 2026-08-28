#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod batch;
mod config;
mod effect;
mod effect_graph;
mod effect_plan;
mod error;
mod image;
mod offscreen;
mod profile;
mod quad;
mod shader;
mod surface;
mod target;
mod text;
mod upload;
mod vector;

pub use config::RendererConfig;
pub use effect_graph::EffectGraphStats;
pub use error::RendererError;
pub use offscreen::TexturePoolStats;
pub use profile::RenderProfile;
pub use shader::EffectShader;
pub use surface::{RenderStatus, RendererDevice, SurfaceRenderer};
