//! Retained UI description consumed by the runtime and produced by Rust or a DSL.

mod element;
mod identity;
mod interaction;
mod scroll;
mod style;
mod text_input;
mod transition;
mod tree;
mod virtual_list;
mod widget;

pub use argui_paint::{
    Border, BorderWidths, ClipBehavior, Color, CornerRadii, Fill, PaintStyle, QuadStyle,
};
pub use element::{Element, ElementKind};
pub use interaction::{
    HitRegion, Interaction, InteractionStyles, InteractionUpdate, NodeId, UiEvent, UiEventKind,
    VisualState,
};
pub use scroll::{
    ScrollAxes, ScrollConfig, ScrollPolarity, ScrollRegion, ScrollbarRegion, ScrollbarStyle,
};
pub use style::{Align, Direction, Edges, Inset, Justify, LayoutStyle, Length, Position, Wrap};
pub use text_input::{ClipboardRequest, TextInput, TextInputStyle};
pub use transition::Transition;
pub use tree::{TreeUpdate, UiTree};
pub use virtual_list::{VirtualList, VirtualWindow};
pub use widget::{Button, ButtonStyle};
