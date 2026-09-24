//! Builds bounded virtual content along the selected scroll axis.

use crate::{Axes, Dimension, Element, Overflow, ScrollAxes};

use std::ops::Range;

use super::{Extents, VirtualItem, VirtualList, VirtualViewport};

impl VirtualList {
    /// Builds a scrollable element containing the mounted window.
    ///
    /// * `key` — stable key for the list container.
    /// * `offset` — current content offset along the list axis.
    /// * `item` — callback that builds each mounted item by index.
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
    ///
    /// # Arguments
    ///
    /// * `key` — stable key for the list container.
    /// * `offset` — current content offset along the list axis.
    /// * `pinned` — optional item index to keep mounted outside the visible window.
    /// * `item` — callback that builds a mounted item by index.
    #[must_use]
    pub fn build_pinned(
        &self,
        key: impl Into<String>,
        offset: f32,
        pinned: Option<usize>,
        mut item: impl FnMut(usize) -> Element,
    ) -> Element {
        self.build_mounted(key, offset, self.window(offset).range, pinned, &mut item)
    }

    /// Builds a presenter-supplied bounded range while native scrolling decides the next range.
    ///
    /// * `key` — stable key for the scroll container.
    /// * `offset` — current content offset along the list axis.
    /// * `range` — half-open range represented by `item`; it must lie within the list.
    /// * `item` — callback that builds each mounted item by index.
    ///
    /// # Panics
    ///
    /// Panics if `range` is reversed or extends beyond the list length.
    #[must_use]
    pub fn build_range(
        &self,
        key: impl Into<String>,
        offset: f32,
        range: Range<usize>,
        mut item: impl FnMut(usize) -> Element,
    ) -> Element {
        assert!(range.start <= range.end && range.end <= self.item_count());
        self.build_mounted(key, offset, range, None, &mut item)
    }

    /// Materializes a supplied item range and optional pinned item.
    ///
    /// * `key` — stable key for the scroll container.
    /// * `offset` — content offset along the active axis.
    /// * `range` — contiguous mounted item indices.
    /// * `pinned` — optional additional index retained outside the range.
    /// * `item` — callback that builds each mounted item.
    ///
    /// Returns a scroll container whose metadata records the actual mounted range.
    fn build_mounted(
        &self,
        key: impl Into<String>,
        offset: f32,
        range: Range<usize>,
        pinned: Option<usize>,
        item: &mut impl FnMut(usize) -> Element,
    ) -> Element {
        let window = self.window(offset);
        let pinned = pinned.filter(|index| *index < self.item_count());
        let before = pinned.filter(|index| *index < range.start);
        let after = pinned.filter(|index| *index >= range.end);
        let indices = before.into_iter().chain(range.clone()).chain(after);
        let mut children = Vec::with_capacity(range.len() + 4);
        let mut previous_end = None;
        for index in indices {
            if previous_end != Some(index) {
                children.push(spacer(
                    self.horizontal,
                    self.offset_of(index) - self.offset_of(previous_end.unwrap_or(0)),
                ));
            }
            let mut element = item(index).shrink(0.0);
            match &self.extents {
                Extents::Fixed(extent) if self.horizontal => {
                    element.style.size.width = Dimension::length(*extent)
                }
                Extents::Fixed(extent) => element.style.size.height = Dimension::length(*extent),
                Extents::Variable { state, .. } => {
                    element.virtual_item = Some(VirtualItem {
                        index,
                        viewport_extent: self.viewport_extent,
                        horizontal: self.horizontal,
                        state: state.clone(),
                    });
                }
            }
            children.push(element);
            previous_end = Some(index + 1);
        }
        children.push(spacer(
            self.horizontal,
            (window.total - self.offset_of(previous_end.unwrap_or(0))).max(0.0),
        ));
        let mut element = if self.horizontal {
            Element::row([Element::row(children).shrink(0.0)])
                .keyed(key)
                .width(Dimension::length(self.viewport_extent))
                .overflow(Axes {
                    x: Overflow::Auto,
                    y: Overflow::Hidden,
                })
                .scroll_config(self.scroll.clone().axes(ScrollAxes::Horizontal))
                .scroll_offset(argui_core::Point::new(offset, 0.0))
        } else {
            Element::column([Element::column(children).shrink(0.0)])
                .keyed(key)
                .height(Dimension::length(self.viewport_extent))
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Auto,
                })
                .scroll_config(self.scroll.clone())
                .scroll_offset(argui_core::Point::new(0.0, offset))
        };
        element.virtual_viewport = Some(VirtualViewport {
            list: self.clone(),
            mounted: range,
        });
        element
    }
}

/// Creates an inaccessible spacer along the selected scroll axis.
///
/// `horizontal` selects width rather than height; `extent` is its logical size.
/// Returns a spacer that contributes to layout without mounting model items.
fn spacer(horizontal: bool, extent: f32) -> Element {
    let spacer = Element::container([]).shrink(0.0).semantic_hidden(true);
    if horizontal {
        spacer.width(Dimension::length(extent))
    } else {
        spacer.height(Dimension::length(extent))
    }
}
