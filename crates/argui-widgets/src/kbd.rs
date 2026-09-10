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
}

impl Kbd {
    #[must_use]
    pub fn new(key: impl Into<String>, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            keys: keys.into_iter().map(Into::into).collect(),
            label: None,
        }
    }

    /// Spoken description for symbolic chords, such as "Control plus K".
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let label = self.label.unwrap_or_else(|| self.keys.join(" + "));
        Element::row(self.keys.into_iter().map(|key| {
            Element::row([Element::text(key).text_style(TextStyle {
                font_size: 12.0,
                line_height: 16.0,
                weight: 500,
                color: theme.muted_foreground,
                wrap: TextWrap::None,
                ..TextStyle::default()
            })])
            .min_width(length(24.0))
            .height(length(24.0))
            .padding(sides(6.0, 2.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(5.0))
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
