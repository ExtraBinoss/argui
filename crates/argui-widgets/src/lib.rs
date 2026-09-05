//! Optional, accessible widgets built from Argui engine primitives.

#[cfg(feature = "button")]
mod button;
#[cfg(feature = "button")]
mod button_behavior;
#[cfg(feature = "dialog")]
mod dialog;
#[cfg(feature = "dialog")]
mod dialog_behavior;
#[cfg(feature = "icons")]
mod icons;
#[cfg(feature = "input")]
mod input;
#[cfg(feature = "popover")]
mod popover;
#[cfg(feature = "popover")]
mod popover_behavior;
#[cfg(any(feature = "text-selection", feature = "popover", feature = "select"))]
mod presence;
#[cfg(feature = "radio-group")]
mod radio_group_behavior;
#[cfg(feature = "range")]
mod range;
#[cfg(feature = "select")]
mod select;
#[cfg(feature = "select")]
mod select_behavior;
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
mod selection;
#[cfg(feature = "slider")]
mod slider;
#[cfg(feature = "spinner")]
mod spinner;
#[cfg(feature = "tabs")]
mod tabs;
#[cfg(feature = "tabs")]
mod tabs_behavior;
#[cfg(feature = "input")]
mod text_field_behavior;
#[cfg(feature = "text-selection")]
mod text_selection;
mod theme;
#[cfg(any(feature = "text-selection", feature = "popover", feature = "select"))]
pub use presence::Presence;
#[cfg(feature = "split-pane")]
mod split_pane;
#[cfg(feature = "split-pane")]
pub use split_pane::{SplitAxis, SplitPane};
#[cfg(feature = "vlist")]
mod vlist;
#[cfg(feature = "vlist")]
pub use vlist::VList;
#[cfg(feature = "tree-view")]
mod tree_view;
#[cfg(feature = "tree-view")]
pub use tree_view::{TreeAction, TreeNode, TreeView, TreeViewCache};
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
mod toggle_behavior;

#[cfg(feature = "button")]
pub use button::{Button, ButtonStyle};
#[cfg(feature = "button")]
pub use button_behavior::{BUTTON_BUSY, BUTTON_SCOPE, ButtonAction, ButtonBehavior, ButtonPart};
#[cfg(feature = "dialog")]
pub use dialog::Dialog;
#[cfg(feature = "dialog")]
pub use dialog_behavior::{DIALOG_OPEN, DIALOG_SCOPE, DialogAction, DialogBehavior, DialogPart};
#[cfg(feature = "icons")]
pub use icons::{TablerIcon, WidgetAssets};
#[cfg(feature = "input")]
pub use input::{Input, InputKind, InputStyle, TextArea};
#[cfg(feature = "popover")]
pub use popover::Popover;
#[cfg(feature = "popover")]
pub use popover_behavior::{
    POPOVER_OPEN, POPOVER_SCOPE, PopoverAction, PopoverBehavior, PopoverPart,
};
#[cfg(feature = "radio-group")]
pub use radio_group_behavior::{RadioGroupAction, RadioGroupBehavior, RadioGroupPart};
#[cfg(feature = "range")]
pub use range::{
    RANGE_SCOPE, RangeAction, RangeAxis, RangeBehavior, RangeConfig, RangeDetents, RangeDirection,
    RangePart, RangeState,
};
#[cfg(feature = "select")]
pub use select::{Select, SelectOption};
#[cfg(feature = "select")]
pub use select_behavior::{
    SELECT_HIGHLIGHTED, SELECT_OPEN, SELECT_SCOPE, SELECT_SELECTED, SelectAction, SelectBehavior,
    SelectPart,
};
#[cfg(feature = "checkbox")]
pub use selection::Checkbox;
#[cfg(feature = "switch")]
pub use selection::Switch;
#[cfg(feature = "radio-group")]
pub use selection::{RadioGroup, RadioOption};
#[cfg(feature = "slider")]
pub use slider::Slider;
#[cfg(feature = "spinner")]
pub use spinner::Spinner;
#[cfg(feature = "tabs")]
pub use tabs::{Tab, Tabs};
#[cfg(feature = "tabs")]
pub use tabs_behavior::{TAB_SELECTED, TABS_SCOPE, TabsAction, TabsBehavior, TabsPart};
#[cfg(feature = "input")]
pub use text_field_behavior::{
    TEXT_FIELD_INVALID, TEXT_FIELD_READ_ONLY, TEXT_FIELD_SCOPE, TextFieldBehavior, TextFieldPart,
};
#[cfg(feature = "text-selection")]
pub use text_selection::{SelectionHost, TextSelectionToolbar};
pub use theme::{WidgetTheme, shadcn};
#[cfg(any(feature = "checkbox", feature = "switch", feature = "radio-group"))]
pub use toggle_behavior::{TOGGLE_CHECKED, TOGGLE_SCOPE, ToggleAction, ToggleBehavior, TogglePart};
