//! Renderer-independent painting primitives and ordered display lists.

mod display_list;
mod effect;
mod gpu_canvas;
mod style;
mod vector;
mod visual;

pub use argui_core::{Color, ColorInterpolation};
pub use display_list::{DisplayCommand, DisplayList, DisplayListError};
pub use effect::{
    BlendMode, EffectArgument, EffectId, EffectInstance, EffectValue, Filter, LayerMask,
    LayerStyle, ProfileDomain, Refraction, RenderObjectId, Shadow,
};
pub use gpu_canvas::{GpuCanvasId, GpuCanvasPrimitive};
pub use style::{
    Border, BorderWidths, CornerRadii, Fill, ImagePrimitive, PaintStyle, Quad, QuadStyle,
};
pub use vector::{VectorAsset, VectorId, VectorPrimitive};
pub use visual::{
    BilinearGradient, ClipChain, ClipRegion, GradientError, GradientStop, GradientStops,
    ImageAsset, ImageAssetError, ImageFit, ImageId, ImageSampling, LinearGradient, RadialGradient,
};
