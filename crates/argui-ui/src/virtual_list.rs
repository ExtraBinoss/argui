use std::ops::Range;

use crate::{Element, Length, ScrollConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualWindow {
    pub range: Range<usize>,
    pub before: f32,
    pub after: f32,
    pub total: f32,
}

#[derive(Clone, Debug, PartialEq)]
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
                enabled: true,
                axes: crate::ScrollAxes::Vertical,
                polarity: crate::ScrollPolarity::Normal,
                chaining: crate::ScrollChaining::Auto,
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
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    #[must_use]
    pub fn window(&self, offset: f32) -> VirtualWindow {
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
        .semantic_hidden(true)
}

/// A virtual list whose rows can report their real height after layout.
///
/// Prefix sums are maintained in a Fenwick tree, so finding the visible range
/// and correcting an anchor after a measurement both cost `O(log n)`.
#[derive(Clone, Debug, PartialEq)]
pub struct VariableList {
    item_count: usize,
    estimated_extent: f32,
    viewport_extent: f32,
    overscan: usize,
    extents: Vec<f32>,
    measured: Vec<bool>,
    prefix: Fenwick,
    scroll: ScrollConfig,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeasurementUpdate {
    pub changed: bool,
    pub corrected_offset: f32,
}

impl VariableList {
    #[must_use]
    pub fn new(item_count: usize, estimated_extent: f32, viewport_extent: f32) -> Self {
        let estimate = sanitize_extent(estimated_extent);
        Self {
            item_count,
            estimated_extent: estimate,
            viewport_extent: viewport_extent.max(0.0),
            overscan: 3,
            extents: vec![estimate; item_count],
            measured: vec![false; item_count],
            prefix: Fenwick::uniform(item_count, estimate),
            scroll: ScrollConfig {
                enabled: true,
                axes: crate::ScrollAxes::Vertical,
                polarity: crate::ScrollPolarity::Normal,
                chaining: crate::ScrollChaining::Auto,
                line_size: estimate,
                multiplier: 1.0,
                scrollbar: None,
            },
        }
    }

    #[must_use]
    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    #[must_use]
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    #[must_use]
    pub fn item_extent(&self, index: usize) -> Option<f32> {
        self.extents.get(index).copied()
    }

    #[must_use]
    pub fn is_measured(&self, index: usize) -> bool {
        self.measured.get(index).copied().unwrap_or(false)
    }

    #[must_use]
    pub fn total_extent(&self) -> f32 {
        self.prefix.total()
    }

    #[must_use]
    pub fn window(&self, offset: f32) -> VirtualWindow {
        let total = self.total_extent();
        let offset = offset.clamp(0.0, (total - self.viewport_extent).max(0.0));
        let first = self.prefix.lower_bound(offset);
        let last = self
            .prefix
            .lower_bound((offset + self.viewport_extent).min(total))
            .saturating_add(1)
            .min(self.item_count);
        let start = first.saturating_sub(self.overscan);
        let end = last.saturating_add(self.overscan).min(self.item_count);
        let before = self.prefix.sum(start);
        let after = (total - self.prefix.sum(end)).max(0.0);
        VirtualWindow {
            range: start..end,
            before,
            after,
            total,
        }
    }

    /// Records a measured row while preserving the row currently under the
    /// viewport's top edge. The returned offset can be applied immediately.
    pub fn measure(&mut self, index: usize, extent: f32, current_offset: f32) -> MeasurementUpdate {
        let Some(previous) = self.extents.get(index).copied() else {
            return MeasurementUpdate {
                corrected_offset: current_offset,
                ..MeasurementUpdate::default()
            };
        };
        let next = sanitize_extent(extent);
        let anchor = self.prefix.lower_bound(current_offset);
        let within = current_offset - self.prefix.sum(anchor);
        let changed = (next - previous).abs() > 0.01 || !self.measured[index];
        if changed {
            self.extents[index] = next;
            self.measured[index] = true;
            self.prefix.add(index, next - previous);
        }
        let max = (self.total_extent() - self.viewport_extent).max(0.0);
        MeasurementUpdate {
            changed,
            corrected_offset: (self.prefix.sum(anchor) + within).clamp(0.0, max),
        }
    }

    #[must_use]
    pub fn build(
        &self,
        key: impl Into<String>,
        offset: f32,
        mut item: impl FnMut(usize) -> Element,
    ) -> Element {
        let window = self.window(offset);
        let mut children = Vec::with_capacity(window.range.len() + 2);
        children.push(spacer(window.before));
        children.extend(window.range.map(|index| {
            item(index)
                .height(Length::Px(self.extents[index]))
                .shrink(0.0)
        }));
        children.push(spacer(window.after));
        Element::column([Element::column(children).shrink(0.0)])
            .keyed(key)
            .height(Length::Px(self.viewport_extent))
            .scrollable(self.scroll.clone())
    }

    #[must_use]
    pub const fn estimated_extent(&self) -> f32 {
        self.estimated_extent
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Fenwick {
    tree: Vec<f32>,
}

impl Fenwick {
    fn uniform(len: usize, value: f32) -> Self {
        let mut result = Self {
            tree: vec![0.0; len + 1],
        };
        for index in 0..len {
            result.add(index, value);
        }
        result
    }

    fn add(&mut self, index: usize, delta: f32) {
        let mut cursor = index + 1;
        while cursor < self.tree.len() {
            self.tree[cursor] += delta;
            cursor += cursor & cursor.wrapping_neg();
        }
    }

    fn sum(&self, end: usize) -> f32 {
        let mut cursor = end.min(self.tree.len().saturating_sub(1));
        let mut sum = 0.0;
        while cursor != 0 {
            sum += self.tree[cursor];
            cursor &= cursor - 1;
        }
        sum
    }

    fn total(&self) -> f32 {
        self.sum(self.tree.len().saturating_sub(1))
    }

    fn lower_bound(&self, target: f32) -> usize {
        let len = self.tree.len().saturating_sub(1);
        if len == 0 || target <= 0.0 {
            return 0;
        }
        let mut index = 0;
        let mut accumulated = 0.0;
        let mut bit = len.next_power_of_two();
        while bit != 0 {
            let next = index + bit;
            if next <= len && accumulated + self.tree[next] <= target {
                index = next;
                accumulated += self.tree[next];
            }
            bit >>= 1;
        }
        index.min(len.saturating_sub(1))
    }
}

fn sanitize_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(f32::EPSILON)
    } else {
        1.0
    }
}
