use argui_text::{TextAlign, TextStyle, TextWrap};
use argui_ui::{AlignItems, Element, Orientation, Role, Semantics, auto, length, percent};

use crate::WidgetTheme;

/// A one-pixel divider, decorative unless explicitly exposed to accessibility.
#[derive(Clone, Debug)]
pub struct Separator {
    key: String,
    orientation: Orientation,
    decorative: bool,
    label: Option<String>,
}

impl Separator {
    /// Creates a non-decorative separator identified by `key`.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            orientation: Orientation::Horizontal,
            decorative: true,
            label: None,
        }
    }

    /// Vertical separators need a definite height on their parent.
    #[must_use]
    /// Sets the separator axis.
    /// `orientation` determines the line direction.
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Centers a label between equal-length rules. Empty labels render a plain rule.
    #[must_use]
    /// Sets the accessible label for a non-decorative separator.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into()).filter(|label| !label.trim().is_empty());
        self
    }

    #[must_use]
    /// Sets whether the separator is hidden from accessibility semantics.
    /// `decorative` is true when it should not be announced.
    pub const fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    #[must_use]
    /// Builds the separator using `theme` for its line color.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let rule = || {
            Element::container([])
                .background(theme.border)
                .semantic_hidden(true)
        };
        let mut element = if let Some(label) = &self.label {
            let line = match self.orientation {
                Orientation::Horizontal => rule().width(length(0.0)).height(length(1.0)),
                Orientation::Vertical => rule().width(length(1.0)).height(length(0.0)),
            }
            .grow(1.0);
            let label = Element::text(label.clone())
                .keyed(format!("{}::label", self.key))
                .text_style(TextStyle {
                    font_size: 13.0,
                    line_height: 18.0,
                    color: theme.muted_foreground,
                    wrap: TextWrap::None,
                    ..TextStyle::default()
                })
                .text_align(TextAlign::Center)
                .min_width(length(0.0));
            let children = [line.clone(), label, line];
            match self.orientation {
                Orientation::Horizontal => {
                    Element::row(children).width(percent(1.0)).height(auto())
                }
                Orientation::Vertical => {
                    Element::column(children).height(percent(1.0)).width(auto())
                }
            }
            .align_items(AlignItems::CENTER)
            .gap(12.0)
        } else {
            match self.orientation {
                Orientation::Horizontal => rule().width(percent(1.0)).height(length(1.0)),
                Orientation::Vertical => rule().width(length(1.0)).height(percent(1.0)),
            }
        };
        element = element.keyed(self.key).shrink(0.0).semantic_hidden(false);
        if self.decorative {
            // A visible label remains readable; only the rules are decorative.
            element.semantic_hidden(self.label.is_none())
        } else {
            let mut semantics = Semantics::new(Role::Separator).orientation(self.orientation);
            if let Some(label) = self.label {
                semantics = semantics.label(label);
                for child in &mut element.children {
                    child.semantic_hidden = true;
                }
            }
            element.semantics(semantics)
        }
    }
}
