use crate::{Element, WritingDirection};

impl Element {
    #[must_use]
    pub fn writing_direction(mut self, direction: WritingDirection) -> Self {
        self.style.writing_direction = direction;
        self
    }

    /// Applies a writing direction throughout this subtree, including mounted entities and portals.
    #[must_use]
    pub fn direction_scope(mut self, direction: WritingDirection) -> Self {
        self.direction_scope = Some(direction);
        self
    }
}
