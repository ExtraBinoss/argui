use argui_ui::{Element, WritingDirection};

/// Direction shared by nested layout and anchored portal placement. Pass the same direction
/// to controlled keyboard models through their `rtl` option.
#[derive(Clone, Debug)]
pub struct Direction {
    pub direction: WritingDirection,
    pub content: Element,
}

impl Direction {
    /// Wraps `content` in the requested writing direction.
    #[must_use]
    pub const fn new(direction: WritingDirection, content: Element) -> Self {
        Self { direction, content }
    }

    #[must_use]
    /// Builds the directional container.
    pub fn build(self) -> Element {
        Element::container([self.content]).direction_scope(self.direction)
    }
}
