//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod identity;
mod interaction;
mod style;
mod tree;
mod widget;

pub use argui_paint::{
    Border, BorderWidths, ClipBehavior, Color, CornerRadii, Fill, PaintStyle, QuadStyle,
};
pub use interaction::{
    HitRegion, Interaction, InteractionStyles, InteractionUpdate, NodeId, UiEvent, UiEventKind,
    VisualState,
};
pub use style::{Align, Direction, Edges, Justify, LayoutStyle, Length, Wrap};
pub use tree::{Element, ElementKind, TreeUpdate, UiTree};
pub use widget::{Button, ButtonStyle};
