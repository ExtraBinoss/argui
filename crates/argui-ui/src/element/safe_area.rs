use argui_core::Insets;

use crate::{Element, Sides, length};

impl Element {
    /// Wraps this element in padding for the supplied logical-pixel safe area.
    /// Copies the element background to the wrapper so it also paints behind
    /// system bars when the host draws edge to edge.
    ///
    /// * `insets` — safe-area distances from the top, right, bottom, and left edges.
    #[must_use]
    pub fn safe_area(self, insets: Insets) -> Self {
        let insets = Insets::new(
            insets.top.max(0.0),
            insets.right.max(0.0),
            insets.bottom.max(0.0),
            insets.left.max(0.0),
        );
        if insets == Insets::ZERO {
            return self;
        }
        let background = self.paint.quad.background.clone();
        let mut wrapper = Self::container([self]);
        wrapper.paint.quad.background = background;
        wrapper.padding(Sides {
            top: length(insets.top),
            right: length(insets.right),
            bottom: length(insets.bottom),
            left: length(insets.left),
        })
    }
}
