//! Borrowed paint and text inputs for one presentation.

use argui_paint::DisplayList;
use argui_text::{PreparedText, TextEngine};

pub(super) enum FrameContent<'a> {
    None,
    Text {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
    },
    Ui {
        engine: &'a mut TextEngine,
        text: &'a PreparedText,
        display_list: &'a DisplayList,
        scale_factor: f32,
    },
    Composite {
        display_list: &'a DisplayList,
        scale_factor: f32,
    },
}
