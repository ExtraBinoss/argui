use argui_text::{TextAlign, TextOverflow};

use crate::{
    Axes, BoxSizing, Dimension, Dimensions, Element, ElementKind, GridAutoFlow, GridPlacement,
    GridTemplateAreas, GridTemplateComponent, LengthPercentage, LengthPercentageAuto, Line,
    Overflow, ScrollbarGutter, Sides, TrackSizingFunction,
};

impl Element {
    /// Sets overflow behavior on the horizontal and vertical axes.
    #[must_use]
    pub fn overflow(mut self, overflow: Axes<Overflow>) -> Self {
        self.style.overflow = overflow;
        self
    }

    /// Sets the reserved space for scrollbars.
    /// * `gutter` — scrollbar gutter policy.
    #[must_use]
    pub fn scrollbar_gutter(mut self, gutter: ScrollbarGutter) -> Self {
        self.style.scrollbar_gutter = gutter;
        self
    }

    /// Sets scrollbar width in logical pixels; negative widths are clamped to zero.
    #[must_use]
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.style.scrollbar_width = width.max(0.0);
        self
    }

    /// Sets how declared dimensions account for padding and border.
    /// * `sizing` — box sizing model.
    #[must_use]
    pub fn box_sizing(mut self, sizing: BoxSizing) -> Self {
        self.style.box_sizing = sizing;
        self
    }

    /// Sets the inset on each side of an absolutely positioned element.
    #[must_use]
    pub fn inset(mut self, inset: Sides<LengthPercentageAuto>) -> Self {
        self.style.inset = inset;
        self
    }

    /// Sets the preferred width and height.
    /// * `size` — dimensions to use for the element.
    #[must_use]
    pub fn size(mut self, size: Dimensions<Dimension>) -> Self {
        self.style.size = size;
        self
    }

    /// Sets the minimum width and height.
    /// * `size` — minimum dimensions.
    #[must_use]
    pub fn min_size(mut self, size: Dimensions<LengthPercentageAuto>) -> Self {
        self.style.min_size = size;
        self
    }

    /// Sets the maximum width and height.
    /// * `size` — maximum dimensions.
    #[must_use]
    pub fn max_size(mut self, size: Dimensions<LengthPercentageAuto>) -> Self {
        self.style.max_size = size;
        self
    }

    /// Sets the preferred width-to-height ratio.
    #[must_use]
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.style.aspect_ratio = Some(ratio);
        self
    }

    /// Sets the row and column gaps.
    #[must_use]
    pub fn gaps(mut self, gaps: Dimensions<LengthPercentage>) -> Self {
        self.style.gap = gaps;
        self
    }

    /// Sets the flex item's initial main-axis size.
    /// * `basis` — initial main-axis dimension.
    #[must_use]
    pub fn flex_basis(mut self, basis: Dimension) -> Self {
        self.style.flex_basis = basis;
        self
    }

    /// Sets the explicit grid row track definitions.
    ///
    /// * `tracks` — row track sizing and optional names.
    #[must_use]
    pub fn grid_template_rows(
        mut self,
        tracks: impl IntoIterator<Item = GridTemplateComponent<String>>,
    ) -> Self {
        self.style.grid_template_rows = tracks.into_iter().collect();
        self
    }

    /// Sets the explicit grid column track definitions.
    ///
    /// * `tracks` — column track sizing and optional names.
    #[must_use]
    pub fn grid_template_columns(
        mut self,
        tracks: impl IntoIterator<Item = GridTemplateComponent<String>>,
    ) -> Self {
        self.style.grid_template_columns = tracks.into_iter().collect();
        self
    }

    /// Sets the sizing functions used for implicit grid rows.
    /// * `tracks` — sizing functions for implicitly created rows.
    #[must_use]
    pub fn grid_auto_rows(mut self, tracks: impl IntoIterator<Item = TrackSizingFunction>) -> Self {
        self.style.grid_auto_rows = tracks.into_iter().collect();
        self
    }

    /// Sets the sizing functions used for implicit grid columns.
    /// * `tracks` — sizing functions for implicitly created columns.
    #[must_use]
    pub fn grid_auto_columns(
        mut self,
        tracks: impl IntoIterator<Item = TrackSizingFunction>,
    ) -> Self {
        self.style.grid_auto_columns = tracks.into_iter().collect();
        self
    }

    /// Sets the grid auto-placement order.
    /// * `flow` — auto-placement direction and density.
    #[must_use]
    pub fn grid_auto_flow(mut self, flow: GridAutoFlow) -> Self {
        self.style.grid_auto_flow = flow;
        self
    }

    /// Sets the named grid area definitions.
    /// * `areas` — named area matrix for the grid.
    #[must_use]
    pub fn grid_template_areas(mut self, areas: GridTemplateAreas<String>) -> Self {
        self.style.grid_template_areas = Some(areas);
        self
    }

    /// Sets this item's grid row placement.
    #[must_use]
    pub fn grid_row(mut self, row: Line<GridPlacement<String>>) -> Self {
        self.style.grid_row = row;
        self
    }

    /// Sets this item's grid column placement.
    #[must_use]
    pub fn grid_column(mut self, column: Line<GridPlacement<String>>) -> Self {
        self.style.grid_column = column;
        self
    }

    /// Sets text alignment on text and text-editor elements.
    /// * `align` — alignment applied to text content.
    #[must_use]
    pub fn text_align(mut self, align: TextAlign) -> Self {
        match &mut self.kind {
            ElementKind::Text { style, .. } => style.align = align,
            ElementKind::TextEditor {
                text,
                placeholder_text,
                ..
            } => {
                text.align = align;
                placeholder_text.align = align;
            }
            ElementKind::Custom(_)
            | ElementKind::Container
            | ElementKind::Image { .. }
            | ElementKind::Vector { .. } => {}
        }
        self
    }

    /// Sets overflow handling on text and text-editor elements.
    #[must_use]
    pub fn text_overflow(mut self, overflow: TextOverflow) -> Self {
        match &mut self.kind {
            ElementKind::Text { style, .. } => style.overflow = overflow,
            ElementKind::TextEditor {
                text,
                placeholder_text,
                ..
            } => {
                text.overflow = overflow;
                placeholder_text.overflow = overflow;
            }
            ElementKind::Custom(_)
            | ElementKind::Container
            | ElementKind::Image { .. }
            | ElementKind::Vector { .. } => {}
        }
        self
    }
}
