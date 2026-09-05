use argui_ui::{Element, ScrollConfig, VirtualList};

use crate::WidgetTheme;

/// Widget presentation around the engine's virtual window calculations.
#[derive(Clone, Debug)]
pub struct VList {
    key: String,
    pub viewport: f32,
    pub offset: f32,
    pub row_height: f32,
}

impl VList {
    #[must_use]
    pub fn new(key: impl Into<String>, row_height: f32, viewport: f32, offset: f32) -> Self {
        assert!(row_height.is_finite() && row_height > 0.0);
        Self { key: key.into(), row_height, viewport: viewport.max(0.0), offset: offset.max(0.0) }
    }

    #[must_use]
    pub fn config(&self, count: usize) -> VirtualList {
        VirtualList::fixed(count, self.row_height, self.viewport).overscan(8)
    }

    #[must_use]
    pub fn build(&self, count: usize, theme: &WidgetTheme, row: impl FnMut(usize) -> Element) -> Element {
        self.config(count)
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
            .build(&self.key, self.offset, row)
    }
}
