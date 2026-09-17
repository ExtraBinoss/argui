use argui_ui::{Element, ScrollConfig, ValueHandler, VirtualList};

use crate::WidgetTheme;

/// Widget presentation around the engine's virtual window calculations.
#[derive(Clone, Debug)]
pub struct VList {
    key: String,
    variable: Option<VirtualList>,
    pub viewport: f32,
    pub offset: f32,
    pub row_height: f32,
    pub effects: Vec<argui_ui::ScrollEffect>,
    pub propagation: argui_ui::ScrollPropagation,
    select_handlers: Vec<ValueHandler<String>>,
    activate_handlers: Vec<ValueHandler<String>>,
}

impl VList {
    /// Creates a fixed-height virtual list with viewport and initial scroll offset.
    ///
    /// # Panics
    ///
    /// Panics if `row_height` is non-finite or not positive.
    /// `key` identifies the viewport; `viewport` is its height and `offset` its initial scroll position.
    #[must_use]
    pub fn new(key: impl Into<String>, row_height: f32, viewport: f32, offset: f32) -> Self {
        assert!(row_height.is_finite() && row_height > 0.0);
        Self {
            key: key.into(),
            variable: None,
            row_height,
            viewport: viewport.max(0.0),
            offset: offset.max(0.0),
            effects: Vec::new(),
            propagation: argui_ui::ScrollPropagation::Chain,
            select_handlers: Vec::new(),
            activate_handlers: Vec::new(),
        }
    }

    /// Keep `config` in application state; clones share measured row heights.
    /// `offset` is the initial vertical scroll position.
    /// `key` identifies the viewport.
    #[must_use]
    pub fn variable(key: impl Into<String>, config: &VirtualList, offset: f32) -> Self {
        let mut list = Self::new(
            key,
            config.estimated_extent(),
            config.viewport_extent(),
            offset,
        );
        list.variable = Some(config.clone());
        list
    }

    /// Adds a handler that receives the stable id selected in [`Self::build_list`].
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<String>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    /// Adds a handler that receives the stable id activated in [`Self::build_list`].
    #[must_use]
    pub fn on_activate(mut self, handler: ValueHandler<String>) -> Self {
        self.activate_handlers.push(handler);
        self
    }

    /// Builds selectable rows with the same behavior as a non-virtual List.
    /// `collection` supplies items, `state` supplies controlled selection, `multiple` enables multi-selection, `theme` styles rows, and `row` builds them.
    #[must_use]
    pub fn build_list(
        &self,
        collection: &crate::Collection,
        state: &crate::ListState,
        multiple: bool,
        theme: &WidgetTheme,
        mut row: impl FnMut(usize) -> Element,
    ) -> Element {
        let mut list = crate::List::new(&self.key, collection).selection(state, multiple);
        for handler in &self.select_handlers {
            list = list.on_select(*handler);
        }
        for handler in &self.activate_handlers {
            list = list.on_activate(*handler);
        }
        let pinned = state
            .active
            .as_deref()
            .and_then(|id| collection.index_of(id));
        list.root(
            self.config(collection.len())
                .scroll_config(self.scroll(theme))
                .build_pinned(&self.key, self.offset, pinned, |index| {
                    list.row(index, row(index), theme)
                })
                .scrollbar_gutter(argui_ui::ScrollbarGutter::Stable),
        )
    }

    #[must_use]
    /// Returns the virtual-list configuration for `count` rows.
    ///
    /// # Panics
    ///
    /// Panics when a retained variable-height configuration has a different item count.
    pub fn config(&self, count: usize) -> VirtualList {
        if let Some(config) = &self.variable {
            assert_eq!(
                count,
                config.item_count(),
                "variable list count must match its retained configuration"
            );
            config.clone().with_viewport(self.viewport).overscan(8)
        } else {
            VirtualList::fixed(count, self.row_height, self.viewport).overscan(8)
        }
    }

    /// Returns whether scrolling from `previous` to `next` needs a new mounted row window.
    /// `count` is the current row count; offsets inside the same overscanned chunk return false.
    #[must_use]
    pub fn window_changed(&self, count: usize, previous: f32, next: f32) -> bool {
        let config = self.config(count);
        config.window(previous).range != config.window(next).range
    }

    #[must_use]
    /// Adds one scroll effect to the viewport.
    pub fn effect(mut self, effect: argui_ui::ScrollEffect) -> Self {
        self.effects.push(effect);
        self
    }

    #[must_use]
    /// Adds each supplied `effects` scroll effect to the viewport.
    pub fn effects(mut self, effects: impl IntoIterator<Item = argui_ui::ScrollEffect>) -> Self {
        self.effects.extend(effects);
        self
    }

    /// Controls whether unused scroll input can reach an enclosing viewport.
    /// `propagation` selects the scroll chaining policy.
    #[must_use]
    pub fn propagation(mut self, propagation: argui_ui::ScrollPropagation) -> Self {
        self.propagation = propagation;
        self
    }

    #[must_use]
    /// Builds a virtual viewport for `count` rows using `theme`, rendering each through `row`.
    pub fn build(
        &self,
        count: usize,
        theme: &WidgetTheme,
        row: impl FnMut(usize) -> Element,
    ) -> Element {
        self.build_pinned(count, theme, None, row)
    }

    /// Keeps the active row mounted when it leaves the virtual window.
    /// `pinned` is the optional row index to retain outside the visible window.
    /// `count` is the number of rows and `theme` supplies scroll styling.
    #[must_use]
    pub fn build_pinned(
        &self,
        count: usize,
        theme: &WidgetTheme,
        pinned: Option<usize>,
        row: impl FnMut(usize) -> Element,
    ) -> Element {
        self.config(count)
            .scroll_config(self.scroll(theme))
            .build_pinned(&self.key, self.offset, pinned, row)
            .scrollbar_gutter(argui_ui::ScrollbarGutter::Stable)
    }

    fn scroll(&self, theme: &WidgetTheme) -> ScrollConfig {
        let mut scroll = ScrollConfig::default()
            .propagation(self.propagation)
            .scrollbar(theme.scrollbar.clone());
        scroll.effects.clone_from(&self.effects);
        scroll
    }

    /// Scrolls a non-virtual header and virtual rows in one viewport.
    /// `header_extent` is the measured height of the header, including its spacing.
    /// `header` is the fixed element above rows; `row` builds each virtual row.
    /// `count` is the number of virtual rows and `theme` supplies scroll styling.
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
