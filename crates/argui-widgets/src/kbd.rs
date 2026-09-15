use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{AlignItems, AlignSelf, Element, JustifyContent, Role, Semantics, length, sides};

/// A visual keycap or chord. It describes a shortcut without registering it.
#[derive(Clone, Debug)]
pub struct Kbd {
    key: String,
    keys: Vec<String>,
    label: Option<String>,
    size: f32,
}

impl Kbd {
    /// Creates a keyboard hint from `keys`, the displayed key labels; `key` identifies the element.
    #[must_use]
    pub fn new(key: impl Into<String>, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            keys: keys.into_iter().map(Into::into).collect(),
            label: None,
            size: 24.0,
        }
    }

    /// Spoken description for symbolic chords, such as "Control plus K".
    #[must_use]
    /// Sets the accessible label for the key sequence.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Keycap height in logical pixels; smaller caps also scale their typography and spacing.
    #[must_use]
    /// Sets the displayed keycap size in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() {
            self.size = size.clamp(12.0, 48.0);
        }
        self
    }

    #[must_use]
    /// Builds the key hint using `theme` for its colors.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let label = self.label.unwrap_or_else(|| self.keys.join(" + "));
        Element::row(self.keys.into_iter().map(|key| {
            Element::row([Element::text(key).text_style(TextStyle {
                font_size: self.size * 0.5,
                line_height: self.size * 2.0 / 3.0,
                weight: 500,
                color: theme.muted_foreground,
                wrap: TextWrap::None,
                ..TextStyle::default()
            })])
            .min_width(length(self.size))
            .height(length(self.size))
            .padding(sides(self.size * 0.25, self.size / 12.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(self.size * 0.2))
            .semantic_hidden(true)
            .shrink(0.0)
        }))
        .keyed(self.key)
        .gap(4.0)
        .align_self(AlignSelf::START)
        .shrink(0.0)
        .semantics(Semantics::new(Role::Text).label(label))
    }
}
