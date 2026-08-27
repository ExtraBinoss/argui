use argui_core::{Affine2D, Point, Rect, ScrollDelta};
use argui_paint::{ClipChain, QuadStyle};

use crate::NodeId;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollAxes {
    Horizontal,
    #[default]
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollPolarity {
    #[default]
    Normal,
    Inverted,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollChaining {
    #[default]
    Auto,
    Contain,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollConfig {
    pub enabled: bool,
    pub axes: ScrollAxes,
    pub polarity: ScrollPolarity,
    pub chaining: ScrollChaining,
    pub line_size: f32,
    pub multiplier: f32,
    pub scrollbar: Option<ScrollbarStyle>,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            axes: ScrollAxes::Vertical,
            polarity: ScrollPolarity::Normal,
            chaining: ScrollChaining::Auto,
            line_size: 40.0,
            multiplier: 1.0,
            scrollbar: None,
        }
    }
}

impl ScrollConfig {
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn axes(mut self, axes: ScrollAxes) -> Self {
        self.axes = axes;
        self
    }

    #[must_use]
    pub const fn polarity(mut self, polarity: ScrollPolarity) -> Self {
        self.polarity = polarity;
        self
    }

    #[must_use]
    pub const fn chaining(mut self, chaining: ScrollChaining) -> Self {
        self.chaining = chaining;
        self
    }

    #[must_use]
    pub const fn line_size(mut self, line_size: f32) -> Self {
        self.line_size = line_size;
        self
    }

    #[must_use]
    pub const fn multiplier(mut self, multiplier: f32) -> Self {
        self.multiplier = multiplier;
        self
    }

    #[must_use]
    pub fn scrollbar(mut self, scrollbar: ScrollbarStyle) -> Self {
        self.scrollbar = Some(scrollbar);
        self
    }

    fn logical_delta(&self, delta: ScrollDelta) -> Point {
        let mut delta = match delta {
            ScrollDelta::Lines(point) => {
                Point::new(point.x * self.line_size, point.y * self.line_size)
            }
            ScrollDelta::Pixels(point) => point,
        };
        let polarity = match self.polarity {
            ScrollPolarity::Normal => -1.0,
            ScrollPolarity::Inverted => 1.0,
        } * self.multiplier;
        delta.x *= polarity;
        delta.y *= polarity;
        match self.axes {
            ScrollAxes::Horizontal => delta.y = 0.0,
            ScrollAxes::Vertical => delta.x = 0.0,
            ScrollAxes::Both => {}
        }
        delta
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarStyle {
    pub width: f32,
    pub inset: f32,
    pub min_thumb: f32,
    pub track: QuadStyle,
    pub thumb: QuadStyle,
}

impl ScrollbarStyle {
    #[must_use]
    pub const fn new(track: QuadStyle, thumb: QuadStyle) -> Self {
        Self {
            width: 10.0,
            inset: 4.0,
            min_thumb: 28.0,
            track,
            thumb,
        }
    }

    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn inset(mut self, inset: f32) -> Self {
        self.inset = inset;
        self
    }

    #[must_use]
    pub const fn min_thumb(mut self, min_thumb: f32) -> Self {
        self.min_thumb = min_thumb;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarRegion {
    pub track: Rect,
    pub thumb: Rect,
    pub style: ScrollbarStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub clip: Rect,
    pub transform: Affine2D,
    pub clips: ClipChain,
    pub max_offset: Point,
    pub config: ScrollConfig,
    pub scrollbar: Option<ScrollbarRegion>,
}

impl ScrollRegion {
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.local_point(point)
            .is_some_and(|local| self.bounds.contains(local) && self.clip.contains(local))
            && self.clips.contains(point)
    }

    #[must_use]
    pub fn scrollbar_contains(&self, point: Point) -> bool {
        self.clips.contains(point)
            && self.scrollbar.as_ref().is_some_and(|scrollbar| {
                self.local_point(point).is_some_and(|local| {
                    self.clip.contains(local) && scrollbar.track.contains(local)
                })
            })
    }

    #[must_use]
    pub fn local_point(&self, point: Point) -> Option<Point> {
        self.transform
            .inverse()
            .map(|inverse| inverse.transform_point(point))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollChange {
    pub node: NodeId,
    pub delta: Point,
    pub offset: Point,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScrollOutcome {
    Changed(ScrollChange),
    Consumed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollDrag {
    node: NodeId,
    grab: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScrollState {
    offsets: Vec<(NodeId, Point)>,
    drag: Option<ScrollDrag>,
}

impl ScrollState {
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub fn offset(&self, node: NodeId) -> Point {
        self.offsets
            .iter()
            .find_map(|(candidate, offset)| (*candidate == node).then_some(*offset))
            .unwrap_or_default()
    }

    pub fn set_offset(&mut self, node: NodeId, offset: Point) -> bool {
        let offset = Point::new(offset.x.max(0.0), offset.y.max(0.0));
        if let Some((_, stored)) = self
            .offsets
            .iter_mut()
            .find(|(candidate, _)| *candidate == node)
        {
            if *stored == offset {
                return false;
            }
            *stored = offset;
        } else {
            self.offsets.push((node, offset));
        }
        true
    }

    pub fn scroll(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> Option<ScrollOutcome> {
        for region in regions.iter().rev().filter(|region| region.contains(point)) {
            if !region.config.enabled {
                continue;
            }
            let requested = region.config.logical_delta(delta);
            let previous = self.offset(region.node);
            let offset = Point::new(
                (previous.x + requested.x).clamp(0.0, region.max_offset.x),
                (previous.y + requested.y).clamp(0.0, region.max_offset.y),
            );
            let applied = Point::new(offset.x - previous.x, offset.y - previous.y);
            if applied == Point::default() {
                if region.config.chaining == ScrollChaining::Contain {
                    return Some(ScrollOutcome::Consumed);
                }
                continue;
            }
            if let Some((_, stored)) = self
                .offsets
                .iter_mut()
                .find(|(candidate, _)| *candidate == region.node)
            {
                *stored = offset;
            } else {
                self.offsets.push((region.node, offset));
            }
            return Some(ScrollOutcome::Changed(ScrollChange {
                node: region.node,
                delta: applied,
                offset,
            }));
        }
        None
    }

    pub fn scrollbar_pressed(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<Option<ScrollChange>> {
        for region in regions.iter().rev() {
            if !region.config.enabled {
                continue;
            }
            let Some(scrollbar) = region.scrollbar.as_ref() else {
                continue;
            };
            let Some(point) = region.local_point(point) else {
                continue;
            };
            if !region.clip.contains(point)
                || !region
                    .clips
                    .contains(region.transform.transform_point(point))
                || !scrollbar.track.contains(point)
            {
                continue;
            }
            let grab = if scrollbar.thumb.contains(point) {
                point.y - scrollbar.thumb.origin.y
            } else {
                scrollbar.thumb.size.height * 0.5
            };
            self.drag = Some(ScrollDrag {
                node: region.node,
                grab,
            });
            return Some(self.drag_to(point, regions));
        }
        None
    }

    pub fn scrollbar_dragged(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<ScrollChange> {
        self.drag?;
        self.drag_to(point, regions)
    }

    pub fn scrollbar_released(&mut self) -> bool {
        self.drag.take().is_some()
    }

    fn drag_to(&mut self, point: Point, regions: &[ScrollRegion]) -> Option<ScrollChange> {
        let drag = self.drag?;
        let region = regions.iter().find(|region| region.node == drag.node)?;
        let scrollbar = region.scrollbar.as_ref()?;
        let point = region.local_point(point)?;
        let travel = scrollbar.track.size.height - scrollbar.thumb.size.height;
        if travel <= 0.0 || region.max_offset.y <= 0.0 {
            return None;
        }
        let thumb_y = (point.y - drag.grab - scrollbar.track.origin.y).clamp(0.0, travel);
        let previous = self.offset(region.node);
        let offset = Point::new(previous.x, thumb_y / travel * region.max_offset.y);
        let applied = Point::new(0.0, offset.y - previous.y);
        if applied.y == 0.0 {
            return None;
        }
        if let Some((_, stored)) = self
            .offsets
            .iter_mut()
            .find(|(candidate, _)| *candidate == region.node)
        {
            *stored = offset;
        } else {
            self.offsets.push((region.node, offset));
        }
        Some(ScrollChange {
            node: region.node,
            delta: applied,
            offset,
        })
    }

    pub fn retain(&mut self, ids: &[NodeId]) {
        self.offsets.retain(|(node, _)| ids.contains(node));
        if self.drag.is_some_and(|drag| !ids.contains(&drag.node)) {
            self.drag = None;
        }
    }
}
