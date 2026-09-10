use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::{Axes, Dimension, Element, Overflow, ScrollConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualWindow {
    pub range: Range<usize>,
    pub before: f32,
    pub after: f32,
    pub total: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VirtualAlignment {
    Start,
    Center,
    End,
    #[default]
    Nearest,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeasurementUpdate {
    pub changed: bool,
    pub corrected_offset: f32,
}

#[derive(Clone, Debug)]
pub struct VirtualItem {
    index: usize,
    viewport_extent: f32,
    state: Rc<RefCell<VariableExtents>>,
}

impl PartialEq for VirtualItem {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.viewport_extent == other.viewport_extent
            && Rc::ptr_eq(&self.state, &other.state)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualList {
    item_count: usize,
    viewport_extent: f32,
    overscan: usize,
    scroll: ScrollConfig,
    extents: Extents,
}

#[derive(Clone, Debug, PartialEq)]
enum Extents {
    Fixed(f32),
    Variable {
        estimate: f32,
        state: Rc<RefCell<VariableExtents>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct VariableExtents {
    values: Vec<f32>,
    measured: Vec<bool>,
    prefix: Fenwick,
}

impl VirtualList {
    #[must_use]
    pub fn fixed(item_count: usize, item_extent: f32, viewport_extent: f32) -> Self {
        Self::from_extents(
            item_count,
            viewport_extent,
            Extents::Fixed(sanitize_extent(item_extent)),
        )
    }

    #[must_use]
    pub fn variable(item_count: usize, estimated_extent: f32, viewport_extent: f32) -> Self {
        let estimate = sanitize_extent(estimated_extent);
        Self::from_extents(
            item_count,
            viewport_extent,
            Extents::Variable {
                estimate,
                state: Rc::new(RefCell::new(VariableExtents {
                    values: vec![estimate; item_count],
                    measured: vec![false; item_count],
                    prefix: Fenwick::uniform(item_count, estimate),
                })),
            },
        )
    }

    fn from_extents(item_count: usize, viewport_extent: f32, extents: Extents) -> Self {
        let line_size = extents.extent(0).unwrap_or(1.0);
        Self {
            item_count,
            viewport_extent: viewport_extent.max(0.0),
            overscan: 3,
            scroll: ScrollConfig::default().line_size(line_size),
            extents,
        }
    }

    #[must_use]
    pub fn item_count(&self) -> usize {
        match &self.extents {
            Extents::Fixed(_) => self.item_count,
            Extents::Variable { state, .. } => state.borrow().values.len(),
        }
    }

    #[must_use]
    pub const fn viewport_extent(&self) -> f32 {
        self.viewport_extent
    }

    /// Resizes a viewport while retaining shared variable measurements.
    #[must_use]
    pub fn with_viewport(mut self, extent: f32) -> Self {
        self.viewport_extent = extent.max(0.0);
        self
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
    pub fn item_extent(&self, index: usize) -> Option<f32> {
        (index < self.item_count())
            .then(|| self.extents.extent(index))
            .flatten()
    }

    #[must_use]
    pub fn is_measured(&self, index: usize) -> bool {
        match &self.extents {
            Extents::Fixed(_) => index < self.item_count(),
            Extents::Variable { state, .. } => {
                state.borrow().measured.get(index).copied().unwrap_or(false)
            }
        }
    }

    #[must_use]
    pub fn total_extent(&self) -> f32 {
        self.extents.total(self.item_count())
    }

    #[must_use]
    pub fn estimated_extent(&self) -> f32 {
        match self.extents {
            Extents::Fixed(extent) => extent,
            Extents::Variable { estimate, .. } => estimate,
        }
    }

    #[must_use]
    pub fn offset_of(&self, index: usize) -> f32 {
        self.extents.prefix(index.min(self.item_count()))
    }

    #[must_use]
    pub fn item_at_offset(&self, offset: f32) -> Option<usize> {
        let item_count = self.item_count();
        (item_count != 0).then(|| {
            self.extents
                .lower_bound(offset.max(0.0), item_count)
                .min(item_count - 1)
        })
    }

    #[must_use]
    pub fn visible_range(&self, offset: f32) -> Range<usize> {
        let start = self.item_at_offset(offset).unwrap_or(0);
        let end = self
            .item_at_offset(offset + self.viewport_extent)
            .map_or(0, |index| index.saturating_add(1))
            .min(self.item_count());
        start..end.max(start).min(self.item_count())
    }

    #[must_use]
    pub fn window(&self, offset: f32) -> VirtualWindow {
        let total = self.total_extent();
        let item_count = self.item_count();
        let offset = offset.clamp(0.0, (total - self.viewport_extent).max(0.0));
        let first = self.extents.lower_bound(offset, item_count).min(item_count);
        let (start, end) = match self.extents {
            Extents::Fixed(extent) => {
                let visible = (self.viewport_extent / extent).ceil() as usize + 1;
                let chunk = visible.max(1);
                let anchor = first / chunk * chunk;
                (
                    anchor.saturating_sub(self.overscan),
                    anchor
                        .saturating_add(chunk)
                        .saturating_add(visible)
                        .saturating_add(self.overscan)
                        .min(item_count),
                )
            }
            Extents::Variable { .. } => {
                let last = self
                    .extents
                    .lower_bound((offset + self.viewport_extent).min(total), item_count)
                    .saturating_add(1)
                    .min(item_count);
                (
                    first.saturating_sub(self.overscan),
                    last.saturating_add(self.overscan).min(item_count),
                )
            }
        };
        let before = self.offset_of(start);
        VirtualWindow {
            range: start..end,
            before,
            after: (total - self.offset_of(end)).max(0.0),
            total,
        }
    }

    #[must_use]
    pub fn scroll_to(&self, index: usize, align: VirtualAlignment, current_offset: f32) -> f32 {
        let item_count = self.item_count();
        if item_count == 0 {
            return 0.0;
        }
        let index = index.min(item_count - 1);
        let start = self.offset_of(index);
        let end = start + self.item_extent(index).unwrap_or_default();
        let viewport_end = current_offset + self.viewport_extent;
        let target = match align {
            VirtualAlignment::Start => start,
            VirtualAlignment::Center => (start + end - self.viewport_extent) * 0.5,
            VirtualAlignment::End => end - self.viewport_extent,
            VirtualAlignment::Nearest if start < current_offset => start,
            VirtualAlignment::Nearest if end > viewport_end => end - self.viewport_extent,
            VirtualAlignment::Nearest => current_offset,
        };
        target.clamp(0.0, (self.total_extent() - self.viewport_extent).max(0.0))
    }

    pub fn measure(&mut self, index: usize, extent: f32, offset: f32) -> MeasurementUpdate {
        let Extents::Variable { state, .. } = &self.extents else {
            return MeasurementUpdate {
                corrected_offset: offset,
                ..MeasurementUpdate::default()
            };
        };
        measure_variable(
            &mut state.borrow_mut(),
            index,
            extent,
            offset,
            self.viewport_extent,
        )
    }

    pub fn insert(&mut self, index: usize, count: usize) {
        let item_count = self.item_count();
        let index = index.min(item_count);
        match &mut self.extents {
            Extents::Fixed(_) => {}
            Extents::Variable { estimate, state } => {
                let mut state = state.borrow_mut();
                state
                    .values
                    .splice(index..index, std::iter::repeat_n(*estimate, count));
                state
                    .measured
                    .splice(index..index, std::iter::repeat_n(false, count));
                state.prefix = Fenwick::from_values(&state.values);
            }
        }
        self.item_count = item_count.saturating_add(count);
    }

    pub fn remove(&mut self, range: Range<usize>) {
        let item_count = self.item_count();
        let start = range.start.min(item_count);
        let end = range.end.max(start).min(item_count);
        match &mut self.extents {
            Extents::Fixed(_) => {}
            Extents::Variable { state, .. } => {
                let mut state = state.borrow_mut();
                state.values.drain(start..end);
                state.measured.drain(start..end);
                state.prefix = Fenwick::from_values(&state.values);
            }
        }
        self.item_count = item_count - (end - start);
    }

    /// Reorder retained measurements. New entries (`None`) start at the estimate.
    /// Call when data changes, before rebuilding mounted rows; all handles share the update.
    pub fn remap(&mut self, order: impl IntoIterator<Item = Option<usize>>) {
        match &mut self.extents {
            Extents::Fixed(_) => self.item_count = order.into_iter().count(),
            Extents::Variable { estimate, state } => {
                let mut state = state.borrow_mut();
                let mut values = Vec::new();
                let mut measured = Vec::new();
                for index in order {
                    values.push(
                        index
                            .and_then(|index| state.values.get(index))
                            .copied()
                            .unwrap_or(*estimate),
                    );
                    measured.push(
                        index
                            .and_then(|index| state.measured.get(index))
                            .copied()
                            .unwrap_or(false),
                    );
                }
                self.item_count = values.len();
                state.prefix = Fenwick::from_values(&values);
                state.values = values;
                state.measured = measured;
            }
        }
    }

    #[must_use]
    pub fn build(
        &self,
        key: impl Into<String>,
        offset: f32,
        item: impl FnMut(usize) -> Element,
    ) -> Element {
        self.build_pinned(key, offset, None, item)
    }

    /// Keeps one active or edited row mounted outside the visible window.
    /// Only the window and the pinned row are visited; gaps retain their measured extent.
    #[must_use]
    pub fn build_pinned(
        &self,
        key: impl Into<String>,
        offset: f32,
        pinned: Option<usize>,
        mut item: impl FnMut(usize) -> Element,
    ) -> Element {
        let window = self.window(offset);
        let pinned = pinned.filter(|index| *index < self.item_count());
        let before = pinned.filter(|index| *index < window.range.start);
        let after = pinned.filter(|index| *index >= window.range.end);
        let indices = before.into_iter().chain(window.range.clone()).chain(after);
        let mut children = Vec::with_capacity(window.range.len() + 4);
        let mut previous_end = None;
        for index in indices {
            if previous_end != Some(index) {
                children.push(spacer(
                    self.offset_of(index) - self.offset_of(previous_end.unwrap_or(0)),
                ));
            }
            let mut element = item(index).shrink(0.0);
            match &self.extents {
                Extents::Fixed(extent) => element.style.size.height = Dimension::length(*extent),
                Extents::Variable { state, .. } => {
                    element.virtual_item = Some(VirtualItem {
                        index,
                        viewport_extent: self.viewport_extent,
                        state: state.clone(),
                    });
                }
            }
            children.push(element);
            previous_end = Some(index + 1);
        }
        children.push(spacer(
            (window.total - self.offset_of(previous_end.unwrap_or(0))).max(0.0),
        ));
        Element::column([Element::column(children).shrink(0.0)])
            .keyed(key)
            .height(Dimension::length(self.viewport_extent))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(self.scroll.clone())
    }
}

impl Extents {
    fn extent(&self, index: usize) -> Option<f32> {
        match self {
            Self::Fixed(extent) => Some(*extent),
            Self::Variable { state, .. } => state.borrow().values.get(index).copied(),
        }
    }

    fn prefix(&self, end: usize) -> f32 {
        match self {
            Self::Fixed(extent) => *extent * end as f32,
            Self::Variable { state, .. } => state.borrow().prefix.sum(end),
        }
    }

    fn total(&self, count: usize) -> f32 {
        self.prefix(count)
    }

    fn lower_bound(&self, offset: f32, count: usize) -> usize {
        match self {
            Self::Fixed(extent) => (offset / *extent).floor() as usize,
            Self::Variable { state, .. } => state.borrow().prefix.lower_bound(offset, count),
        }
    }
}

impl VirtualItem {
    #[must_use]
    pub fn measure_layout(&self, extent: f32, offset: f32) -> MeasurementUpdate {
        measure_variable(
            &mut self.state.borrow_mut(),
            self.index,
            extent,
            offset,
            self.viewport_extent,
        )
    }
}

fn measure_variable(
    state: &mut VariableExtents,
    index: usize,
    extent: f32,
    offset: f32,
    viewport_extent: f32,
) -> MeasurementUpdate {
    let Some(previous) = state.values.get(index).copied() else {
        return MeasurementUpdate {
            corrected_offset: offset,
            ..MeasurementUpdate::default()
        };
    };
    let anchor = state.prefix.lower_bound(offset, state.values.len());
    let within = offset - state.prefix.sum(anchor);
    let next = sanitize_extent(extent);
    let changed = (next - previous).abs() > 0.01 || !state.measured[index];
    if changed {
        state.values[index] = next;
        state.measured[index] = true;
        state.prefix.add(index, next - previous);
    }
    let maximum = (state.prefix.total() - viewport_extent).max(0.0);
    MeasurementUpdate {
        changed,
        corrected_offset: (state.prefix.sum(anchor) + within).clamp(0.0, maximum),
    }
}

fn spacer(height: f32) -> Element {
    Element::container([])
        .height(Dimension::length(height))
        .shrink(0.0)
        .semantic_hidden(true)
}

#[derive(Clone, Debug, PartialEq)]
struct Fenwick {
    tree: Vec<f32>,
}

impl Fenwick {
    fn uniform(len: usize, value: f32) -> Self {
        Self::from_values(&vec![value; len])
    }

    fn from_values(values: &[f32]) -> Self {
        let mut result = Self {
            tree: vec![0.0; values.len() + 1],
        };
        for (index, value) in values.iter().copied().enumerate() {
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

    fn lower_bound(&self, target: f32, count: usize) -> usize {
        if count == 0 || target <= 0.0 {
            return 0;
        }
        let mut index = 0;
        let mut accumulated = 0.0;
        let mut bit = count.next_power_of_two();
        while bit != 0 {
            let next = index + bit;
            if next <= count && accumulated + self.tree[next] <= target {
                index = next;
                accumulated += self.tree[next];
            }
            bit >>= 1;
        }
        index.min(count.saturating_sub(1))
    }
}

fn sanitize_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(f32::EPSILON)
    } else {
        1.0
    }
}
