use argui_text::{TextAlign, TextOverflow};

use crate::{
    Axes, BoxSizing, Dimension, Dimensions, Element, ElementKind, GridAutoFlow, GridPlacement,
    GridTemplateAreas, GridTemplateComponent, LengthPercentage, LengthPercentageAuto, Line,
    Overflow, ScrollbarGutter, Sides, TrackSizingFunction,
};

impl Element {
    #[must_use]
    pub fn overflow(mut self, overflow: Axes<Overflow>) -> Self {
        self.style.overflow = overflow;
        self
    }

    #[must_use]
    pub fn scrollbar_gutter(mut self, gutter: ScrollbarGutter) -> Self {
        self.style.scrollbar_gutter = gutter;
        self
    }

    #[must_use]
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.style.scrollbar_width = width.max(0.0);
        self
    }

    #[must_use]
    pub fn box_sizing(mut self, sizing: BoxSizing) -> Self {
        self.style.box_sizing = sizing;
        self
    }

    #[must_use]
    pub fn inset(mut self, inset: Sides<LengthPercentageAuto>) -> Self {
        self.style.inset = inset;
        self
    }

    #[must_use]
    pub fn size(mut self, size: Dimensions<Dimension>) -> Self {
        self.style.size = size;
        self
    }

    #[must_use]
    pub fn min_size(mut self, size: Dimensions<LengthPercentageAuto>) -> Self {
        self.style.min_size = size;
        self
    }

    #[must_use]
    pub fn max_size(mut self, size: Dimensions<LengthPercentageAuto>) -> Self {
        self.style.max_size = size;
        self
    }

    #[must_use]
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.style.aspect_ratio = Some(ratio);
        self
    }

    #[must_use]
    pub fn gaps(mut self, gaps: Dimensions<LengthPercentage>) -> Self {
        self.style.gap = gaps;
        self
    }

    #[must_use]
    pub fn flex_basis(mut self, basis: Dimension) -> Self {
        self.style.flex_basis = basis;
        self
    }

    #[must_use]
    pub fn grid_template_rows(
        mut self,
        tracks: impl IntoIterator<Item = GridTemplateComponent<String>>,
    ) -> Self {
        self.style.grid_template_rows = tracks.into_iter().collect();
        self
    }

    #[must_use]
    pub fn grid_template_columns(
        mut self,
        tracks: impl IntoIterator<Item = GridTemplateComponent<String>>,
    ) -> Self {
        self.style.grid_template_columns = tracks.into_iter().collect();
        self
    }

    #[must_use]
    pub fn grid_auto_rows(mut self, tracks: impl IntoIterator<Item = TrackSizingFunction>) -> Self {
        self.style.grid_auto_rows = tracks.into_iter().collect();
        self
    }

    #[must_use]
    pub fn grid_auto_columns(
        mut self,
        tracks: impl IntoIterator<Item = TrackSizingFunction>,
    ) -> Self {
        self.style.grid_auto_columns = tracks.into_iter().collect();
        self
    }

    #[must_use]
    pub fn grid_auto_flow(mut self, flow: GridAutoFlow) -> Self {
        self.style.grid_auto_flow = flow;
        self
    }

    #[must_use]
    pub fn grid_template_areas(mut self, areas: GridTemplateAreas<String>) -> Self {
        self.style.grid_template_areas = Some(areas);
        self
    }

    #[must_use]
    pub fn grid_row(mut self, row: Line<GridPlacement<String>>) -> Self {
        self.style.grid_row = row;
        self
    }

    #[must_use]
    pub fn grid_column(mut self, column: Line<GridPlacement<String>>) -> Self {
        self.style.grid_column = column;
        self
    }

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
            ElementKind::Container | ElementKind::Image { .. } | ElementKind::Vector { .. } => {}
        }
        self
    }

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
            ElementKind::Container | ElementKind::Image { .. } | ElementKind::Vector { .. } => {}
        }
        self
    }
}
