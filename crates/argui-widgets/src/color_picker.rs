mod input;
mod paint;
mod values;
mod view;

use argui_core::{Color, Rect};
use argui_runtime::LayoutSnapshot;

use crate::{RangeBehavior, RangeConfig, RangeState, WidgetTheme};

/// Text representation used by the color editor. Alpha is always available.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorFormat {
    #[default]
    Hex,
    Rgb,
    Hsl,
    Hsv,
}

impl ColorFormat {
    /// All supported textual color formats.
    pub const ALL: [Self; 4] = [Self::Hex, Self::Rgb, Self::Hsl, Self::Hsv];

    #[must_use]
    /// Returns the short display label for this format.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Rgb => "RGB",
            Self::Hsl => "HSL",
            Self::Hsv => "HSV",
        }
    }
}

/// Retained editing state, including uncommitted text and captured gestures.
/// Call `layout_changed` after layout, and `update` for UI events. A true return
/// requests a rebuild; compare `color()` before/after to publish color changes.
#[derive(Clone, Debug)]
pub struct ColorPickerState {
    hsv: [f32; 3],
    alpha: f32,
    format: ColorFormat,
    enabled: bool,
    draft: Option<(usize, String, bool)>,
    pad: Option<Rect>,
    drag_start: Option<[f32; 3]>,
    hue_range: RangeState,
    alpha_range: RangeState,
}

impl ColorPickerState {
    /// Creates an enabled picker state initialized from `color`.
    #[must_use]
    pub fn new(color: Color) -> Self {
        let [r, g, b, alpha] = color.to_srgba();
        Self {
            hsv: values::rgb_to_hsv([r, g, b], 0.0),
            alpha,
            format: ColorFormat::Hex,
            enabled: true,
            draft: None,
            pad: None,
            drag_start: None,
            hue_range: RangeState::default(),
            alpha_range: RangeState::default(),
        }
    }

    #[must_use]
    /// Returns the current color represented by the retained HSV and alpha values.
    pub fn color(&self) -> Color {
        let [r, g, b] = values::hsv_to_rgb(self.hsv);
        Color::srgba(r, g, b, self.alpha)
    }

    /// Replaces the external value without discarding hue when the color is gray.
    pub fn set_color(&mut self, color: Color) {
        if self.color() == color {
            return;
        }
        let [r, g, b, alpha] = color.to_srgba();
        let previous_saturation = self.hsv[1];
        self.hsv = values::rgb_to_hsv([r, g, b], self.hsv[0]);
        if self.hsv[2] == 0.0 {
            self.hsv[1] = previous_saturation;
        }
        self.alpha = alpha;
        self.draft = None;
    }

    #[must_use]
    /// Returns the currently selected textual format.
    pub const fn format(&self) -> ColorFormat {
        self.format
    }

    /// Selects the text format and discards any incomplete text draft.
    pub fn set_format(&mut self, format: ColorFormat) {
        self.format = format;
        self.draft = None;
    }

    /// Enables or disables editing, cancelling an active drag when disabled.
    /// `enabled` controls whether picker interactions can change the color.
    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            if let Some(start) = self.drag_start.take() {
                self.hsv = start;
            }
            self.draft = None;
            self.hue_range = RangeState::default();
            self.alpha_range = RangeState::default();
        }
        self.enabled = enabled;
    }

    /// Updates the retained interaction geometry from the latest `layout` snapshot.
    /// `key` is the element-key prefix used to find this picker's subcontrols.
    pub fn layout_changed(&mut self, key: &str, layout: &LayoutSnapshot) {
        self.pad = layout.bounds(&format!("{key}::pad"));
        self.hue_range
            .layout_changed(layout, &self.range(key, false));
        self.alpha_range
            .layout_changed(layout, &self.range(key, true));
    }

    fn range(&self, key: &str, alpha: bool) -> RangeBehavior {
        let (suffix, label, value, maximum, step) = if alpha {
            ("alpha", "Opacity", self.alpha * 100.0, 100.0, 1.0)
        } else {
            ("hue", "Hue", self.hsv[0], 360.0, 1.0)
        };
        RangeBehavior::new(
            format!("{key}::{suffix}"),
            label,
            value,
            RangeConfig::new(0.0, maximum, step),
        )
        .enabled(self.enabled)
    }
}

#[derive(Clone, Debug)]
pub struct ColorPicker<'a> {
    key: String,
    label: String,
    state: &'a ColorPickerState,
}

impl<'a> ColorPicker<'a> {
    /// Creates a picker presentation with an accessible `label` and retained `state`.
    /// `key` identifies the picker and scopes its child element keys.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        state: &'a ColorPickerState,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            state,
        }
    }

    #[must_use]
    /// Builds the picker using `theme` for controls and text.
    pub fn build(self, theme: &WidgetTheme) -> argui_ui::Element {
        self.view(theme)
    }
}
