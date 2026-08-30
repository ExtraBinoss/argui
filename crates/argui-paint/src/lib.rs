//! Renderer-independent painting primitives and ordered display lists.

mod display_list;
mod effect;
mod style;
mod vector;
mod visual;

pub use argui_core::Color;
pub use display_list::{DisplayCommand, DisplayList, DisplayListError};
pub use effect::{
    BlendMode, EffectArgument, EffectId, EffectInstance, EffectValue, Filter, LayerMask,
    LayerStyle, ProfileDomain, Refraction, RenderObjectId, Shadow,
};
pub use style::{
    Border, BorderWidths, CornerRadii, Fill, ImagePrimitive, PaintStyle, Quad, QuadStyle,
};
pub use vector::{VectorAsset, VectorId, VectorPrimitive, VectorVertex};
pub use visual::{
    ClipChain, ClipRegion, GradientError, GradientStop, GradientStops, ImageAsset, ImageAssetError,
    ImageFit, ImageId, ImageSampling, LinearGradient, RadialGradient,
};
