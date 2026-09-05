use argui_ui::{Element, ScrollConfig, VirtualList};

use crate::WidgetTheme;

/// Widget presentation around the engine's virtual window calculations.
#[derive(Clone, Debug)]
pub struct VList {
    key: String,
    pub viewport: f32,
    pub offset: f32,
    pub row_height: f32,
    pub effects: Vec<argui_ui::ScrollEffect>,
    pub propagation: argui_ui::ScrollPropagation,
}

impl VList {
    #[must_use]
    pub fn new(key: impl Into<String>, row_height: f32, viewport: f32, offset: f32) -> Self {
        assert!(row_height.is_finite() && row_height > 0.0);
        Self {
            key: key.into(),
            row_height,
            viewport: viewport.max(0.0),
            offset: offset.max(0.0),
            effects: Vec::new(),
            propagation: argui_ui::ScrollPropagation::Chain,
        }
    }

    #[must_use]
    pub fn config(&self, count: usize) -> VirtualList {
        VirtualList::fixed(count, self.row_height, self.viewport).overscan(8)
    }

    #[must_use]
    pub fn effect(mut self, effect: argui_ui::ScrollEffect) -> Self {
        self.effects.push(effect);
        self
    }

    #[must_use]
    pub fn effects(mut self, effects: impl IntoIterator<Item = argui_ui::ScrollEffect>) -> Self {
        self.effects.extend(effects);
        self
    }

    /// Controls whether unused scroll input can reach an enclosing viewport.
    #[must_use]
    pub fn propagation(mut self, propagation: argui_ui::ScrollPropagation) -> Self {
        self.propagation = propagation;
        self
    }

    #[must_use]
    pub fn build(
        &self,
        count: usize,
        theme: &WidgetTheme,
        row: impl FnMut(usize) -> Element,
    ) -> Element {
        let mut scroll = ScrollConfig::default()
            .propagation(self.propagation)
            .scrollbar(theme.scrollbar.clone());
        scroll.effects.clone_from(&self.effects);
        self.config(count)
            .scroll_config(scroll)
            .build(&self.key, self.offset, row)
            .scrollbar_gutter(argui_ui::ScrollbarGutter::Stable)
    }

    /// Scrolls a non-virtual header and virtual rows in one viewport.
    /// `header_extent` is the measured height of the header, including its spacing.
    #[must_use]
    pub fn build_with_header(
        &self,
        count: usize,
        theme: &WidgetTheme,
        header: Element,
        header_extent: f32,
        row: impl FnMut(usize) -> Element,
    ) -> Element {
        let mut list = self.clone();
        list.offset = (self.offset - header_extent.max(0.0)).max(0.0);
        let mut root = list.build(count, theme, row);
        root.children.insert(0, header.shrink(0.0));
        root
    }
}
