#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! WGPU renderer boundary.

mod batch;
mod config;
mod damage;
mod effect;
mod effect_graph;
mod effect_plan;
mod error;
mod gpu_canvas;
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

pub use config::{
    DamageTracking, EffectQuality, EffectQualitySettings, RendererConfig, SurfaceAlphaMode,
};
pub use damage::{DamageMode, DamagePlan, DamageProfile, DamageRegion, DamageSnapshot};
pub use effect_graph::{EffectGraphAnalysis, EffectGraphStats, analyze_display_list};
pub use error::{RendererAttemptFailure, RendererError};
pub use gpu_canvas::{
    GpuCanvasDeviceContext, GpuCanvasDiagnostic, GpuCanvasDiagnosticKind, GpuCanvasError,
    GpuCanvasFactory, GpuCanvasFailureStage, GpuCanvasMailbox, GpuCanvasRegistration,
    GpuCanvasRegistry, GpuCanvasRegistryError, GpuCanvasRenderContext, GpuCanvasRenderer,
    GpuCanvasRequirements, GpuCanvasStats,
};
pub use offscreen::TexturePoolStats;
pub use profile::{
    AdapterProfile, GpuFrameProfile, GpuPassProfile, RenderProfile, VectorAtlasStats,
};
pub use registry::{
    EffectDamage, EffectDefinition, EffectInput, EffectParameter, EffectParameterType,
    EffectPassDefinition, EffectRegistry,
};
pub use surface::{RenderStatus, RendererDevice, SurfaceRenderer};
pub use text::TextAtlasStats;
/// Exact WGPU version used by Argui's public GPU-canvas contexts.
pub use wgpu;
