//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod effect;
mod element;
mod identity;
mod interaction;
mod overlay;
mod scroll;
mod style;
mod text_input;
mod transition;
mod tree;
mod virtual_list;
mod widget;

pub use argui_core::{Transform2D, TransformOrigin};
pub use argui_paint::{
    BlendMode, Border, BorderWidths, ClipBehavior, Color, CornerRadii, CustomEffect, Fill, Filter,
    GradientStop, ImageAsset, ImageFit, ImageId, ImageSampling, LayerMask, LayerStyle,
    LinearGradient, PaintStyle, QuadStyle, RadialGradient, Refraction, ShaderEffectId, Shadow,
    VectorAsset, VectorId,
};
pub use effect::{EffectScope, ScopedEffect};
pub use element::{Element, ElementKind};
pub use interaction::{
    HitRegion, Interaction, InteractionStyles, InteractionUpdate, NodeId, UiEvent, UiEventKind,
    VisualState,
};
pub use overlay::{OverlayAlign, OverlayPlacement, PlacedOverlay, PlacementSide};
pub use scroll::{
    ScrollAxes, ScrollChaining, ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarRegion,
    ScrollbarStyle,
};
pub use style::{Align, Direction, Edges, Inset, Justify, LayoutStyle, Length, Position, Wrap};
pub use text_input::{ClipboardRequest, TextInput, TextInputStyle};
pub use transition::Transition;
pub use tree::{TreeUpdate, UiTree};
pub use virtual_list::{VirtualList, VirtualWindow};
pub use widget::{Button, ButtonStyle};
