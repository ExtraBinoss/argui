use argui_ui::{Element, Position, Sides, length, percent};

/// Reserves width / ratio in normal flow and fits its content to that box.
#[derive(Clone, Debug)]
pub struct AspectRatio {
    key: String,
    ratio: f32,
    content: Element,
}

impl AspectRatio {
    /// The ratio must be finite and strictly positive.
    #[must_use]
    /// Creates a container with the requested width-to-height ratio.
    /// `key` identifies the element, `ratio` is width divided by height, and `content` is fitted inside it.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is not finite and strictly positive.
    pub fn new(key: impl Into<String>, ratio: f32, content: Element) -> Self {
        assert!(
            ratio.is_finite() && ratio > 0.0,
            "aspect ratio must be finite and positive"
        );
        Self {
            key: key.into(),
            ratio,
            content,
        }
    }

    #[must_use]
    /// Builds the aspect-ratio container.
    pub fn build(self) -> Element {
        Element::container([self
            .content
            .absolute(Sides::length(0.0))
            .width(percent(1.0))
            .height(percent(1.0))])
        .keyed(self.key)
        .width(percent(1.0))
        .min_width(length(0.0))
        .shrink(0.0)
        .aspect_ratio(self.ratio)
        .position(Position::Relative)
    }
}
