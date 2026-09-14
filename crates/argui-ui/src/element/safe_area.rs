use argui_core::Insets;

use crate::{Element, Sides, length};

impl Element {
    /// Wrap this element in padding for the supplied logical-pixel safe area.
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
        Self::container([self]).padding(Sides {
            top: length(insets.top),
            right: length(insets.right),
            bottom: length(insets.bottom),
            left: length(insets.left),
        })
    }
}
