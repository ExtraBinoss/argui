use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::ScrollConfig;

mod build;
mod fenwick;
use fenwick::Fenwick;

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

/// One measured item extent reported to a virtual-list presenter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualMeasurement {
    /// Zero-based index of the measured item.
    pub index: usize,
    /// Item width or height along the list axis, in logical pixels.
    pub extent: f32,
}

/// Native window state carried by a mounted scroll container.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualViewport {
    /// List estimates and retained measurements used for window calculation.
    pub list: VirtualList,
    /// Half-open item range currently mounted by the presenter.
    pub mounted: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct VirtualItem {
    index: usize,
    viewport_extent: f32,
    horizontal: bool,
    state: Rc<RefCell<VariableExtents>>,
}

impl PartialEq for VirtualItem {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.viewport_extent == other.viewport_extent
            && self.horizontal == other.horizontal
            && Rc::ptr_eq(&self.state, &other.state)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VirtualList {
    item_count: usize,
    viewport_extent: f32,
    horizontal: bool,
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
    /// Creates a list whose items share one fixed extent.
    ///
    /// # Arguments
    ///
    /// * `item_count` — initial number of items.
    /// * `item_extent` — extent of each item in logical pixels.
    /// * `viewport_extent` — viewport extent along the list axis.
    #[must_use]
    pub fn fixed(item_count: usize, item_extent: f32, viewport_extent: f32) -> Self {
        Self::from_extents(
            item_count,
            viewport_extent,
            Extents::Fixed(sanitize_extent(item_extent)),
        )
    }

    /// Creates a list with variable item extents initialized to one estimate.
    ///
    /// The finite `item_count` can grow with [`Self::insert`]. Measurement
    /// storage is proportional to that count, while mounted elements remain
    /// bounded by the visible window and overscan.
    ///
    /// # Arguments
    ///
    /// * `item_count` — initial number of items.
    /// * `estimated_extent` — initial extent used until an item is measured.
    /// * `viewport_extent` — viewport extent along the list axis.
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
            horizontal: false,
            overscan: 3,
            scroll: ScrollConfig::default().line_size(line_size),
            extents,
        }
    }

    /// Selects horizontal scrolling and width measurements for this list.
    ///
    /// Returns the list with its existing item extents and viewport extent
    /// interpreted along the horizontal axis.
    #[must_use]
    pub const fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Returns whether item extents are measured along the horizontal axis.
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        self.horizontal
    }

    /// Returns whether mounted items update their measured extents.
    #[must_use]
    pub const fn is_variable(&self) -> bool {
        matches!(self.extents, Extents::Variable { .. })
    }

    /// Copies variable measurements into an independent list for atomic host edits.
    ///
    /// Returns a list whose later insertions and removals cannot mutate this list.
    #[must_use]
    pub fn detached(&self) -> Self {
        let mut next = self.clone();
        if let Extents::Variable { state, .. } = &mut next.extents {
            let measurements = state.borrow().clone();
            *state = Rc::new(RefCell::new(measurements));
        }
        next
    }

    /// Returns the current number of items in the list.
    #[must_use]
    pub fn item_count(&self) -> usize {
        match &self.extents {
            Extents::Fixed(_) => self.item_count,
            Extents::Variable { state, .. } => state.borrow().values.len(),
        }
    }

    /// Returns the viewport extent along the list axis.
    #[must_use]
    pub const fn viewport_extent(&self) -> f32 {
        self.viewport_extent
    }

    /// Resizes the viewport while retaining shared variable measurements.
    /// `extent` is the new viewport extent in logical pixels.
    #[must_use]
    pub fn with_viewport(mut self, extent: f32) -> Self {
        self.viewport_extent = extent.max(0.0);
        self
    }

    /// Sets the number of extra items kept around the visible window.
    /// `overscan` is the number of items retained outside the visible range.
    #[must_use]
    pub const fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Sets the scrolling configuration used by a built list.
    /// `scroll` configures scroll behavior for the generated container.
    #[must_use]
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    /// Returns the extent of an item, or `None` when its index is out of range.
    /// `index` is the zero-based item index.
    #[must_use]
    pub fn item_extent(&self, index: usize) -> Option<f32> {
        (index < self.item_count())
            .then(|| self.extents.extent(index))
            .flatten()
    }

    /// Returns whether an item has a measured extent.
    /// `index` is the zero-based item index.
    #[must_use]
    pub fn is_measured(&self, index: usize) -> bool {
        match &self.extents {
            Extents::Fixed(_) => index < self.item_count(),
            Extents::Variable { state, .. } => {
                state.borrow().measured.get(index).copied().unwrap_or(false)
            }
        }
    }

    /// Returns the total estimated or measured extent of all items.
    #[must_use]
    pub fn total_extent(&self) -> f32 {
        self.extents.total(self.item_count())
    }

    /// Returns the configured fixed extent or initial variable-item estimate.
    #[must_use]
    pub fn estimated_extent(&self) -> f32 {
        match self.extents {
            Extents::Fixed(extent) => extent,
            Extents::Variable { estimate, .. } => estimate,
        }
    }

    /// Returns the content offset at the beginning of an item index.
    /// `index` is clamped to the current item count.
    #[must_use]
    pub fn offset_of(&self, index: usize) -> f32 {
        self.extents.prefix(index.min(self.item_count()))
    }

    /// Returns the item containing an offset, or `None` for an empty list.
    /// `offset` is the current content offset in logical pixels.
    #[must_use]
    pub fn item_at_offset(&self, offset: f32) -> Option<usize> {
        let item_count = self.item_count();
        (item_count != 0).then(|| {
            self.extents
                .lower_bound(offset.max(0.0), item_count)
                .min(item_count - 1)
        })
    }

    /// Returns the half-open item range intersecting the viewport at an offset.
    /// `offset` is the current content offset in logical pixels.
    #[must_use]
    pub fn visible_range(&self, offset: f32) -> Range<usize> {
        let start = self.item_at_offset(offset).unwrap_or(0);
        let end = self
            .item_at_offset(offset + self.viewport_extent)
            .map_or(0, |index| index.saturating_add(1))
            .min(self.item_count());
        start..end.max(start).min(self.item_count())
    }

    /// Computes the mounted item window and extents before and after it.
    /// `offset` is the current content offset in logical pixels.
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
            Extents::Variable { estimate, .. } => {
                let last = self
                    .extents
                    .lower_bound((offset + self.viewport_extent).min(total), item_count)
                    .saturating_add(1)
                    .min(item_count);
                // Keep a stable mounted window while the pointer moves through nearby
                // variable rows. Remounting at every row boundary forces fresh text
                // shaping during a fast scroll; an estimate-sized chunk amortizes that
                // work while `last` still guarantees every visible row is present.
                let visible = (self.viewport_extent / estimate).ceil() as usize;
                let chunk = visible.saturating_add(1).max(1);
                let anchor = first / chunk * chunk;
                (
                    anchor.saturating_sub(self.overscan),
                    last.max(anchor.saturating_add(chunk).saturating_add(visible))
                        .min(item_count),
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

    /// Computes a clamped offset that reveals `index` using `align`, relative to
    /// `current_offset`; offsets are in logical pixels.
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

    /// Records a measured item extent and corrects the scroll offset to preserve its anchor.
    ///
    /// * `index` — zero-based item index that was measured.
    /// * `extent` — measured item extent in logical pixels.
    /// * `offset` — current content offset before correction.
    ///
    /// Returns whether the measurement changed and the corrected offset.
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

    /// Inserts estimated items at an index and updates the list length.
    ///
    /// * `index` — insertion position, clamped to the current list length.
    /// * `count` — number of items to insert.
    pub fn insert(&mut self, index: usize, count: usize) {
        let item_count = self.item_count();
        let index = index.min(item_count);
        match &mut self.extents {
            Extents::Fixed(_) => {}
            Extents::Variable { estimate, state } => {
                let mut state = state.borrow_mut();
                if index == item_count {
                    for _ in 0..count {
                        state.values.push(*estimate);
                        state.measured.push(false);
                        state.prefix.push(*estimate);
                    }
                } else {
                    state
                        .values
                        .splice(index..index, std::iter::repeat_n(*estimate, count));
                    state
                        .measured
                        .splice(index..index, std::iter::repeat_n(false, count));
                    state.prefix = Fenwick::from_values(&state.values);
                }
            }
        }
        self.item_count = item_count.saturating_add(count);
    }

    /// Removes a range of items and their retained measurements.
    ///
    /// * `range` — half-open range of item indices to remove.
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
    ///
    /// * `order` — old item index for each new position, or `None` for a new item.
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
    /// Returns whether this item's measured extent is its width.
    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        self.horizontal
    }

    /// Returns this item's zero-based index in its virtual list.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Records the measured extent for this mounted item.
    ///
    /// * `extent` — measured extent in logical pixels.
    /// * `offset` — current content offset, corrected to preserve the anchor.
    ///
    /// Returns whether the shared measurement changed and the corrected offset.
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

fn sanitize_extent(extent: f32) -> f32 {
    if extent.is_finite() {
        extent.max(f32::EPSILON)
    } else {
        1.0
    }
}
