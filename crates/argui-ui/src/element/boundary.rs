use crate::{Axes, Element, Overflow};

impl Element {
    /// Isolates one content subtree from its ancestors' intrinsic sizing and
    /// baseline calculations. Give the wrapper a height or external constraints:
    /// content contributes no intrinsic size; padding and borders still apply.
    ///
    /// The content uses the wrapper's normal layout style, padding and border.
    /// Both axes must remain clipped or scrollable. UI ancestry, events,
    /// semantics and inherited styles are preserved across the layout boundary.
    #[must_use]
    pub fn layout_boundary(content: Self) -> Self {
        let mut element = Self::column([content]);
        element.layout_boundary = true;
        element.style.overflow = Axes {
            x: Overflow::Clip,
            y: Overflow::Clip,
        };
        element
    }
}
