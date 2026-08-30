#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod batch;
mod config;
mod effect;
mod effect_graph;
mod effect_plan;
mod error;
mod gpu_profile;
mod image;
mod offscreen;
mod profile;
mod quad;
mod registry;
mod surface;
mod target;
mod text;
mod upload;
mod vector;

pub use config::{EffectQuality, EffectQualitySettings, RendererConfig, SurfaceAlphaMode};
pub use effect_graph::EffectGraphStats;
pub use error::RendererError;
pub use offscreen::TexturePoolStats;
pub use profile::{AdapterProfile, GpuFrameProfile, GpuPassProfile, RenderProfile};
pub use registry::{
    EffectDefinition, EffectInput, EffectParameter, EffectParameterType, EffectPassDefinition,
    EffectRegistry,
};
pub use surface::{RenderStatus, RendererDevice, SurfaceRenderer};
