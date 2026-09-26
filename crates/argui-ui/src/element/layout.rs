//! Layout builders for retained elements.

use super::Element;
use crate::{
    AlignContent, AlignItems, AlignSelf, Dimension, Dimensions, Display, FlexDirection, FlexWrap,
    JustifyContent, JustifyItems, JustifySelf, LayoutInsets, LengthPercentage,
    LengthPercentageAuto, Position, PositionInsets, Sides,
};

impl Element {
    /// Sets the preferred width.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.size.width = width;
        self
    }

    /// Sets the preferred height.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.size.height = height;
        self
    }

    /// Sets the minimum width.
    #[must_use]
    pub fn min_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.min_size.width = width;
        self
    }

    /// Sets the minimum height.
    #[must_use]
    pub fn min_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.min_size.height = height;
        self
    }

    /// Sets the maximum height.
    #[must_use]
    pub fn max_height(mut self, height: LengthPercentageAuto) -> Self {
        self.style.max_size.height = height;
        self
    }

    /// Sets the maximum width.
    #[must_use]
    pub fn max_width(mut self, width: LengthPercentageAuto) -> Self {
        self.style.max_size.width = width;
        self
    }

    /// Sets the layout display mode.
    #[must_use]
    pub fn display(mut self, display: Display) -> Self {
        self.style.display = display;
        self
    }

    /// Sets the main-axis direction for flex layout.
    #[must_use]
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style.flex_direction = direction;
        self
    }

    /// Sets whether flex items wrap onto additional lines.
    #[must_use]
    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.style.flex_wrap = wrap;
        self
    }

    /// Sets cross-axis alignment for this container's items.
    #[must_use]
    pub fn align_items(mut self, alignment: AlignItems) -> Self {
        self.style.align_items = Some(alignment);
        self
    }

    /// Sets this flex item's cross-axis alignment in its parent.
    #[must_use]
    pub fn align_self(mut self, alignment: AlignSelf) -> Self {
        self.style.align_self = Some(alignment);
        self
    }

    /// Sets alignment of grid items in their cells.
    #[must_use]
    pub fn justify_items(mut self, alignment: JustifyItems) -> Self {
        self.style.justify_items = Some(alignment);
        self
    }

    /// Sets alignment of this grid item in its cell.
    #[must_use]
    pub fn justify_self(mut self, alignment: JustifySelf) -> Self {
        self.style.justify_self = Some(alignment);
        self
    }

    /// Sets how flex or grid content is distributed across the container.
    /// * `alignment` — distribution mode for the container's content.
    #[must_use]
    pub fn align_content(mut self, alignment: AlignContent) -> Self {
        self.style.align_content = Some(alignment);
        self
    }

    /// Sets how items are distributed along the main axis.
    /// * `alignment` — distribution mode along the main axis.
    #[must_use]
    pub fn justify_content(mut self, alignment: JustifyContent) -> Self {
        self.style.justify_content = Some(alignment);
        self
    }

    /// Sets the element's positioning mode.
    /// * `position` — positioning mode for the element.
    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.style.position = position;
        self
    }

    /// Positions the element absolutely with the supplied per-side inset.
    #[must_use]
    pub fn absolute(mut self, inset: Sides<LengthPercentageAuto>) -> Self {
        self.style.position = Position::Absolute;
        self.style.inset = inset;
        self.style.logical_inset = None;
        self
    }

    /// Sets pixel insets, including optional logical horizontal sides, without
    /// changing the element's positioning mode.
    ///
    /// `inset` is resolved against the element's effective writing direction.
    #[must_use]
    pub fn layout_inset(mut self, inset: PositionInsets) -> Self {
        self.style.inset = Sides {
            left: inset
                .left
                .map_or_else(LengthPercentageAuto::auto, LengthPercentageAuto::length),
            right: inset
                .right
                .map_or_else(LengthPercentageAuto::auto, LengthPercentageAuto::length),
            top: inset
                .top
                .map_or_else(LengthPercentageAuto::auto, LengthPercentageAuto::length),
            bottom: inset
                .bottom
                .map_or_else(LengthPercentageAuto::auto, LengthPercentageAuto::length),
        };
        self.style.logical_inset = Some(inset);
        self
    }

    /// Sets the focus scope established by this element.
    #[must_use]
    pub fn focus_scope(mut self, scope: crate::FocusScope) -> Self {
        self.focus_scope = Some(scope);
        self
    }

    /// Sets the element's margins.
    /// * `margin` — per-side outer spacing.
    #[must_use]
    pub fn margin(mut self, margin: Sides<LengthPercentageAuto>) -> Self {
        self.style.margin = margin;
        self.style.logical_margin = None;
        self
    }

    /// Sets pixel margins, including optional logical horizontal sides.
    ///
    /// `margin` is resolved against the element's effective writing direction.
    #[must_use]
    pub fn layout_margin(mut self, margin: LayoutInsets) -> Self {
        self.style.margin = Sides {
            left: LengthPercentageAuto::length(margin.left),
            right: LengthPercentageAuto::length(margin.right),
            top: LengthPercentageAuto::length(margin.top),
            bottom: LengthPercentageAuto::length(margin.bottom),
        };
        self.style.logical_margin = Some(margin);
        self
    }

    /// Sets the element's padding.
    #[must_use]
    pub fn padding(mut self, padding: Sides<LengthPercentage>) -> Self {
        self.style.padding = padding;
        self.style.logical_padding = None;
        self
    }

    /// Sets pixel padding, including optional logical horizontal sides.
    ///
    /// `padding` is resolved against the element's effective writing direction.
    #[must_use]
    pub fn layout_padding(mut self, padding: LayoutInsets) -> Self {
        self.style.padding = Sides {
            left: LengthPercentage::length(padding.left),
            right: LengthPercentage::length(padding.right),
            top: LengthPercentage::length(padding.top),
            bottom: LengthPercentage::length(padding.bottom),
        };
        self.style.logical_padding = Some(padding);
        self
    }

    /// Sets equal row and column gaps in logical pixels.
    /// * `gap` — spacing between rows and columns.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        let gap = LengthPercentage::length(gap);
        self.style.gap = Dimensions {
            width: gap,
            height: gap,
        };
        self
    }

    /// Sets the gap between flex rows or grid rows.
    #[must_use]
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.style.gap.height = LengthPercentage::length(gap);
        self
    }

    /// Sets the gap between flex columns or grid columns.
    #[must_use]
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.style.gap.width = LengthPercentage::length(gap);
        self
    }

    /// Sets the flex growth factor.
    /// * `grow` — flex growth factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
        self
    }

    /// Sets the flex shrink factor.
    #[must_use]
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }
}
