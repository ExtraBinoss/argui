//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod action;
mod activation;
mod editing;
pub use action::{
    ActionBinding, ActionError, ActionId, ActionInvocation, ActionScope, ActionState, Shortcut,
};
pub use editing::UiCommand;
mod binding;
mod caret;
mod cursor;
mod desktop_backdrop;
pub use desktop_backdrop::{DesktopBackdrop, DesktopBackdropState};
mod custom;
pub use custom::{
    CustomConstraints, CustomDescription, CustomElement, CustomLayoutContext, CustomMeasurement,
    CustomPaintContext, CustomPhaseStats, CustomState,
};
mod effect;
mod element;
mod native_content;
pub use native_content::NativeContent;
mod event;
mod focus;
pub use argui_accessibility::FocusPolicy;
mod gesture;
mod identity;
mod interaction;
mod layout_builders;
mod overlay;
mod responsive;
mod scroll;
mod scroll_config;
mod scroll_effect;
mod scroll_gesture;
mod scroll_physics;
mod scroll_request;
mod semantics;
mod state;
mod style;
mod text_input;
mod text_selection;
mod traversal;
mod tree;
mod update;
mod virtual_list;

pub use activation::{ActivationSource, ClickEvent};
pub use argui_accessibility::{
    CheckedState, GridPosition, LiveRegion, Orientation, PopupKind, Role, SemanticAction,
    SemanticState, SemanticValue, Semantics, SortDirection,
};
pub use argui_animation::{Motion, MotionBinding, MotionState, Transition, Tween};
pub use argui_core::{Insets, Transform2D, TransformOrigin};
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
    ColorHandlerValue, ColorValueFormat, ContinuousValuePhase, EventFilter, EventHandler,
    EventHandlerId, EventListener, EventListenerOptions, EventOwnerId, EventPhase, EventType,
    FromHandlerValue, HandlerValue, RangeHandlerValue, SplitHandlerValue, UiEvent, UiEventKind,
    ValueHandler,
};
pub use focus::{FocusContainment, FocusRequest, FocusScope, FocusTarget, InitialFocus};
pub use gesture::{
    GestureArena, GestureCapture, GestureDelivery, GestureEvent, GestureKind, GesturePhase,
    GestureSet, PanAxis, PanGesture, PinchGesture, RotationGesture, TapGesture,
};
pub use interaction::{
    HitRegion, HitShape, HitTestStyle, Interaction, InteractionUpdate, KeyboardActivation, NodeId,
    PointerEvents, WindowDragBehavior,
};
pub use overlay::{
    AnchorPortal, AnchorWidth, CollisionPolicy, DismissPolicy, FloatingPlacement, OverlaySurface,
    PlacedOverlay, Placement, Portal, PortalTarget, ViewportAlign, ViewportPlacement, WindowLayer,
};
pub use responsive::{ContainerQuery, ContainerScopeId};
pub use scroll::{ScrollRegion, ScrollbarAxis, ScrollbarGeometry, ScrollbarRegion, scrollbar_at};
pub use scroll_config::{
    ElasticScroll, InertialScroll, OverscrollBehavior, ScrollAnchoring, ScrollAxes, ScrollConfig,
    ScrollPhysics, ScrollPolarity, ScrollPropagation, ScrollbarPartStyle, ScrollbarStyle,
    ScrollbarVisibility,
};
pub use scroll_effect::{ScrollEffect, ScrollMetric, ScrollMetrics};
pub use scroll_gesture::ScrollGesture;
pub use scroll_request::{ScrollAlignment, ScrollBehavior, ScrollRequest, ScrollTarget};
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
pub use text_input::{
    ClipboardRequest, HistoryConfig, TextInputFilter, TextPrivacy, TextSelection,
    TextSelectionRequest,
};
pub use text_selection::{
    DocumentSelectionEndpoint, DocumentTextPoint, DocumentTextSelection, SelectionCapabilities,
    SelectionCommand, SelectionGranularity, TextSelectionHighlight, TextSelectionStyle, UserSelect,
};
pub use tree::{TreeUpdate, TreeUpdateStats, UiTree};
pub use virtual_list::{
    MeasurementUpdate, VirtualAlignment, VirtualItem, VirtualList, VirtualWindow,
};

mod semantic_relations;
pub use semantic_relations::{
    SemanticBindings, SemanticDiagnostic, SemanticReferenceError, SemanticTarget,
};
