//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod binding;
mod caret;
mod cursor;
mod effect;
mod element;
mod event;
mod focus;
mod gesture;
mod identity;
mod interaction;
mod layout_builders;
mod overlay;
mod resize;
mod responsive;
mod scroll;
mod semantics;
mod state;
mod style;
mod text_input;
mod text_selection;
mod traversal;
mod tree;
mod update;
mod virtual_list;

pub use argui_accessibility::{
    LiveRegion, Orientation, Role, SemanticAction, SemanticState, SemanticValue, Semantics,
};
pub use argui_animation::{Motion, MotionBinding, MotionState, Transition, Tween};
pub use argui_core::{Transform2D, TransformOrigin};
pub use argui_paint::{
    BlendMode, Border, BorderWidths, Color, CornerRadii, EffectArgument, EffectId, EffectInstance,
    EffectValue, Fill, Filter, GradientStop, ImageAsset, ImageFit, ImageId, ImageSampling,
    LayerMask, LayerStyle, LinearGradient, PaintStyle, ProfileDomain, QuadStyle, RadialGradient,
    Refraction, RenderObjectId, Shadow, VectorAsset, VectorId,
};
pub use binding::{BindingImpact, MotionProperty, PropertyBinding, property};
pub use caret::{
    CaretAlign, CaretAnimation, CaretFrame, CaretHeight, CaretPrimitive, CaretStyle, CaretVisual,
};
pub use cursor::CursorIcon;
pub use effect::{EffectScope, ScopedEffect};
pub use element::{Element, ElementKind, TextEditorSpec};
pub use event::{
    EventListener, EventListenerOptions, EventOwnerId, EventPhase, EventType, UiEvent, UiEventKind,
};
pub use focus::{FocusContainment, FocusRequest, FocusScope, FocusTarget, InitialFocus};
pub use gesture::{GestureArena, GestureEvent, GestureKind, GesturePhase, GestureSet};
pub use interaction::{
    HitRegion, HitShape, HitTestStyle, Interaction, InteractionUpdate, KeyboardActivation, NodeId,
    PointerEvents, WindowDragBehavior,
};
pub use overlay::{OverlayAlign, OverlayAnchor, OverlayPlacement, PlacedOverlay, PlacementSide};
pub use resize::{Resizable, ResizeAxes, ResizeConfig, ResizeEvent, ResizeState};
pub use responsive::{ContainerQuery, ContainerScopeId};
pub use scroll::{
    ScrollAxes, ScrollChaining, ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarPartStyle,
    ScrollbarRegion, ScrollbarStyle, scrollbar_at,
};
pub use state::{
    EffectPropertyKey, PropertyKey, State, StateName, StateScopeId, StateSelector, StyleCondition,
    StylePatch, StyleProperty, StylePropertyValue, StyleTransition, TransitionDirection,
    TransitionRule, VisualState, VisualStates,
};
pub use style::{
    AlignContent, AlignItems, AlignSelf, AlignmentSafety, Axes, BoxSizing, Dimension, Dimensions,
    Display, ExpandedDimension, ExpandedLengthPercentage, ExpandedLengthPercentageAuto,
    FlexDirection, FlexWrap, GridAutoFlow, GridPlacement, GridTemplateArea, GridTemplateAreas,
    GridTemplateComponent, GridTemplateRepetition, JustifyContent, JustifyItems, JustifySelf,
    LayoutStyle, LengthPercentage, LengthPercentageAuto, Line, MaxTrackSizingFunction,
    MinTrackSizingFunction, Overflow, Position, RepetitionCount, ScrollbarGutter, Sides,
    TrackSizingFunction, WritingDirection,
};
pub use style::{
    auto, evenly_sized_tracks, flex, fr, length, line, minmax, percent, repeat, sides, span, zero,
};
pub use text_input::{ClipboardRequest, TextInputFilter, TextSelection, TextSelectionRequest};
pub use text_selection::{
    DocumentTextPoint, DocumentTextSelection, SelectionCapabilities, SelectionCommand,
    SelectionGranularity, TextSelectionStyle, UserSelect,
};
pub use tree::{TreeUpdate, TreeUpdateStats, UiTree};
pub use virtual_list::{MeasurementUpdate, VariableList, VirtualList, VirtualWindow};
