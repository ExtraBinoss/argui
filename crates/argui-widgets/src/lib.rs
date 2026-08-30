//! Optional, accessible widgets built from Argui engine primitives.

mod button;
mod dialog;
mod icons;
mod input;
mod select;
mod selection;
mod slider;
mod spinner;
mod tabs;
mod theme;

pub use button::{Button, ButtonStyle};
pub use dialog::{Dialog, DialogAction};
pub use icons::{TablerIcon, WidgetAssets};
pub use input::{Input, InputKind, InputStyle, TextArea};
pub use select::{Select, SelectAction, SelectOption};
pub use selection::{Checkbox, RadioGroup, RadioOption, Switch};
pub use slider::{Slider, SliderConfig, SliderState};
pub use spinner::Spinner;
pub use tabs::{Tab, Tabs};
pub use theme::{WidgetTheme, shadcn};
