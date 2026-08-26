use std::ops::Range;

use crate::{Element, Length, ScrollConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualWindow {
    pub range: Range<usize>,
    pub before: f32,
    pub after: f32,
    pub total: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualList {
    pub item_count: usize,
    pub item_extent: f32,
    pub viewport_extent: f32,
    pub overscan: usize,
    pub scroll: ScrollConfig,
}

impl VirtualList {
    #[must_use]
    pub const fn new(item_count: usize, item_extent: f32, viewport_extent: f32) -> Self {
        Self {
            item_count,
            item_extent,
            viewport_extent,
            overscan: 3,
            scroll: ScrollConfig {
                axes: crate::ScrollAxes::Vertical,
                polarity: crate::ScrollPolarity::Normal,
                line_size: item_extent,
                multiplier: 1.0,
                scrollbar: None,
            },
        }
    }

    #[must_use]
    pub const fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    #[must_use]
    pub const fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    #[must_use]
    pub fn window(self, offset: f32) -> VirtualWindow {
        let extent = self.item_extent.max(f32::EPSILON);
        let total = extent * self.item_count as f32;
        let offset = offset.clamp(0.0, (total - self.viewport_extent).max(0.0));
        let first = (offset / extent).floor() as usize;
        let visible = (self.viewport_extent / extent).ceil() as usize + 1;
        let chunk = visible.max(1);
        let anchor = first / chunk * chunk;
        let start = anchor.saturating_sub(self.overscan);
        let end = anchor
            .saturating_add(chunk)
            .saturating_add(visible)
            .saturating_add(self.overscan)
            .min(self.item_count);
        let before = start as f32 * extent;
        let after = (self.item_count - end) as f32 * extent;
        VirtualWindow {
            range: start..end,
            before,
            after,
            total,
        }
    }

    #[must_use]
    pub fn build(
        self,
        key: impl Into<String>,
        offset: f32,
        mut item: impl FnMut(usize) -> Element,
    ) -> Element {
        let window = self.window(offset);
        let mut children = Vec::with_capacity(window.range.len() + 2);
        children.push(spacer(window.before));
        children.extend(
            window
                .range
                .map(|index| item(index).height(Length::Px(self.item_extent)).shrink(0.0)),
        );
        children.push(spacer(window.after));
        Element::column([Element::column(children).shrink(0.0)])
            .keyed(key)
            .height(Length::Px(self.viewport_extent))
            .scrollable(self.scroll)
    }
}

fn spacer(height: f32) -> Element {
    Element::container([])
        .height(Length::Px(height))
        .shrink(0.0)
}
