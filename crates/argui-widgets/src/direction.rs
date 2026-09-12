use argui_ui::{Element, WritingDirection};

/// Direction shared by nested layout and anchored portal placement. Pass the same direction
/// to controlled keyboard models through their `rtl` option.
#[derive(Clone, Debug)]
pub struct Direction {
    pub direction: WritingDirection,
    pub content: Element,
}

impl Direction {
    #[must_use]
    pub const fn new(direction: WritingDirection, content: Element) -> Self {
        Self { direction, content }
    }

    #[must_use]
    pub fn build(self) -> Element {
        Element::container([self.content]).direction_scope(self.direction)
    }
}
