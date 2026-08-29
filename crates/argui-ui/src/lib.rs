//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod binding;
mod cursor;
mod effect;
mod element;
mod focus;
mod gesture;
mod identity;
mod interaction;
mod overlay;
mod scroll;
mod semantics;
mod style;
mod text_input;
mod traversal;
mod tree;
mod update;
mod virtual_list;
mod widget;

pub use argui_accessibility::{
    LiveRegion, Orientation, Role, SemanticAction, SemanticState, SemanticValue, Semantics,
};
pub use argui_animation::{Motion, MotionBinding, MotionState, Tween};
pub use argui_core::{Transform2D, TransformOrigin};
pub use argui_paint::{
    BlendMode, Border, BorderWidths, ClipBehavior, Color, CornerRadii, EffectArgument, EffectId,
    EffectInstance, EffectValue, Fill, Filter, GradientStop, ImageAsset, ImageFit, ImageId,
    ImageSampling, LayerMask, LayerStyle, LinearGradient, PaintStyle, ProfileDomain, QuadStyle,
    RadialGradient, Refraction, RenderObjectId, Shadow, VectorAsset, VectorId,
};
pub use binding::{BindingImpact, MotionProperty, PropertyBinding, property};
pub use cursor::CursorIcon;
pub use effect::{EffectScope, ScopedEffect};
pub use element::{Element, ElementKind};
pub use focus::{FocusContainment, FocusRequest, FocusScope, FocusTarget, InitialFocus};
pub use gesture::{GestureArena, GestureEvent, GestureKind, GesturePhase, GestureSet};
pub use interaction::{
    HitRegion, Interaction, InteractionStyles, InteractionUpdate, KeyboardActivation, NodeId,
    UiEvent, UiEventKind, VisualState,
};
pub use overlay::{OverlayAlign, OverlayAnchor, OverlayPlacement, PlacedOverlay, PlacementSide};
pub use scroll::{
    ScrollAxes, ScrollChaining, ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarRegion,
    ScrollbarStyle,
};
pub use style::{Align, Direction, Edges, Inset, Justify, LayoutStyle, Length, Position, Wrap};
pub use text_input::{ClipboardRequest, TextInput, TextInputStyle};
pub use tree::{TreeUpdate, TreeUpdateStats, UiTree};
pub use virtual_list::{MeasurementUpdate, VariableList, VirtualList, VirtualWindow};
pub use widget::{Button, ButtonStyle};
