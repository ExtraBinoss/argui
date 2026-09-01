//! Optional, accessible widgets built from Argui engine primitives.

mod button;
mod button_behavior;
mod dialog;
mod dialog_behavior;
mod icons;
mod input;
mod radio_group_behavior;
mod range;
mod select;
mod select_behavior;
mod selection;
mod slider;
mod spinner;
mod tabs;
mod tabs_behavior;
mod text_field_behavior;
mod theme;
mod toggle_behavior;

pub use button::{Button, ButtonStyle};
pub use button_behavior::{BUTTON_BUSY, BUTTON_SCOPE, ButtonAction, ButtonBehavior, ButtonPart};
pub use dialog::Dialog;
pub use dialog_behavior::{DIALOG_OPEN, DIALOG_SCOPE, DialogAction, DialogBehavior, DialogPart};
pub use icons::{TablerIcon, WidgetAssets};
pub use input::{Input, InputKind, InputStyle, TextArea};
pub use radio_group_behavior::{RadioGroupAction, RadioGroupBehavior, RadioGroupPart};
pub use range::{
    RANGE_SCOPE, RangeAction, RangeAxis, RangeBehavior, RangeConfig, RangeDetents, RangeDirection,
    RangePart, RangeState,
};
pub use select::{Select, SelectOption};
pub use select_behavior::{
    SELECT_HIGHLIGHTED, SELECT_OPEN, SELECT_SCOPE, SELECT_SELECTED, SelectAction, SelectBehavior,
    SelectPart,
};
pub use selection::{Checkbox, RadioGroup, RadioOption, Switch};
pub use slider::Slider;
pub use spinner::Spinner;
pub use tabs::{Tab, Tabs};
pub use tabs_behavior::{TAB_SELECTED, TABS_SCOPE, TabsAction, TabsBehavior, TabsPart};
pub use text_field_behavior::{
    TEXT_FIELD_INVALID, TEXT_FIELD_READ_ONLY, TEXT_FIELD_SCOPE, TextFieldBehavior, TextFieldPart,
};
pub use theme::{WidgetTheme, shadcn};
pub use toggle_behavior::{TOGGLE_CHECKED, TOGGLE_SCOPE, ToggleAction, ToggleBehavior, TogglePart};
