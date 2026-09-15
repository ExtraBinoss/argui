use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii, ImageFit, ImageId};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{AlignItems, Element, JustifyContent, Role, Semantics, length, percent};

/// An already-loaded image or a controlled fallback, with one accessible name.
#[derive(Clone, Debug)]
pub struct Avatar {
    key: String,
    label: String,
    fallback: String,
    image: Option<ImageId>,
    size: f32,
}

impl Avatar {
    /// Creates an avatar with an accessible label and fallback text.
    /// `key` identifies the element, `label` is its accessible name, and `fallback` is shown without an image.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        fallback: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            fallback: fallback.into(),
            image: None,
            size: 40.0,
        }
    }

    /// Supply `None` while loading or after an image error. The application owns loading.
    #[must_use]
    pub const fn image(mut self, image: Option<ImageId>) -> Self {
        self.image = image;
        self
    }

    #[must_use]
    /// Sets the avatar's square size in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        self.size = if size.is_finite() {
            size.max(1.0)
        } else {
            40.0
        };
        self
    }

    #[must_use]
    /// Builds the avatar using `theme` for its fallback and border colors.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let child = self.image.map_or_else(
            || {
                Element::text(self.fallback).text_style(TextStyle {
                    font_size: self.size * 0.35,
                    line_height: self.size * 0.5,
                    weight: 600,
                    color: theme.foreground,
                    wrap: TextWrap::None,
                    ..TextStyle::default()
                })
            },
            |image| {
                Element::image(image)
                    .image_fit(ImageFit::Cover)
                    .width(percent(1.0))
                    .height(percent(1.0))
            },
        );
        Element::row([child.semantic_hidden(true)])
            .keyed(self.key)
            .semantics(Semantics::new(Role::Image).label(self.label))
            .width(length(self.size))
            .height(length(self.size))
            .shrink(0.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .clip(CornerRadii::all(self.size * 0.5))
    }
}
