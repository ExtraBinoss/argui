use std::sync::Arc;

use argui_core::{Affine2D, Point, Rect, TextPosition};
use argui_paint::{Border, ClipChain, Color, CornerRadii, DisplayList, Fill, Quad};
use argui_text::TextLayout;
use argui_ui::{DocumentTextPoint, NodeId, TextSelectionStyle, UiTree};

use crate::LayoutOutput;

#[derive(Clone, Debug, PartialEq)]
pub struct TextRegion {
    pub node: NodeId,
    pub text_index: usize,
    pub text_len: usize,
    pub origin: Point,
    pub layout: Arc<TextLayout>,
    pub transform: Affine2D,
    pub clips: ClipChain,
    pub style: TextSelectionStyle,
    pub interaction_order: usize,
}

impl TextRegion {
    #[must_use]
    pub fn hit_position(&self, point: Point) -> Option<DocumentTextPoint> {
        self.local_point(point)
            .and_then(|point| self.layout.hit_position(point))
            .map(|position| DocumentTextPoint::new(self.node, position))
    }

    #[must_use]
    pub fn closest_position(&self, point: Point) -> DocumentTextPoint {
        let local = self.local_point_unclipped(point);
        DocumentTextPoint::new(self.node, self.layout.closest_position(local))
    }

    #[must_use]
    pub fn distance_squared(&self, point: Point) -> f32 {
        let local = self.local_point_unclipped(point);
        self.layout.lines.iter().fold(f32::INFINITY, |best, line| {
            let dx = axis_distance(
                local.x,
                line.bounds.origin.x,
                line.bounds.origin.x + line.bounds.size.width,
            );
            let dy = axis_distance(
                local.y,
                line.bounds.origin.y,
                line.bounds.origin.y + line.bounds.size.height,
            );
            best.min(dx * dx + dy * dy)
        })
    }

    #[must_use]
    pub fn visual_rects(&self, start: usize, end: usize) -> Vec<Rect> {
        self.layout
            .selection_rects(
                TextPosition::new(start, argui_core::CaretAffinity::Before),
                TextPosition::new(end, argui_core::CaretAffinity::After),
            )
            .into_iter()
            .map(|mut rect| {
                rect.origin.x += self.origin.x;
                rect.origin.y += self.origin.y;
                rect
            })
            .collect()
    }

    pub(crate) fn translate(&mut self, delta: Point) {
        self.origin.x += delta.x;
        self.origin.y += delta.y;
    }

    fn local_point(&self, point: Point) -> Option<Point> {
        self.clips
            .contains(point)
            .then(|| self.local_point_unclipped(point))
    }

    fn local_point_unclipped(&self, point: Point) -> Point {
        let point = self
            .transform
            .inverse()
            .map_or(point, |inverse| inverse.transform_point(point));
        Point::new(point.x - self.origin.x, point.y - self.origin.y)
    }
}

fn axis_distance(value: f32, start: f32, end: f32) -> f32 {
    if value < start {
        start - value
    } else if value > end {
        value - end
    } else {
        0.0
    }
}

pub(crate) fn paint(ui: &UiTree, region: &TextRegion, display_list: &mut DisplayList) {
    let Some(range) = ui.document_selection_range(region.node, region.text_len) else {
        return;
    };
    let rects = region.visual_rects(range.start, range.end);
    for bounds in &rects {
        paint_rect(display_list, region, *bounds, region.style.background, 0.0);
    }
    if ui.document_selection_handles_visible()
        && let Some(selection) = ui.document_selection()
    {
        let forward = ui
            .node_ids()
            .iter()
            .position(|node| *node == selection.anchor.node)
            .zip(
                ui.node_ids()
                    .iter()
                    .position(|node| *node == selection.focus.node),
            )
            .is_none_or(|(anchor, focus)| {
                (anchor, selection.anchor.position.index) <= (focus, selection.focus.position.index)
            });
        if region.node == selection.anchor.node
            && let Some(rect) = if forward { rects.first() } else { rects.last() }
        {
            paint_handle(display_list, region, *rect, forward);
        }
        if region.node == selection.focus.node
            && let Some(rect) = if forward { rects.last() } else { rects.first() }
        {
            paint_handle(display_list, region, *rect, !forward);
        }
    }
}

fn paint_handle(
    display_list: &mut DisplayList,
    region: &TextRegion,
    selection: Rect,
    leading: bool,
) {
    let diameter = 10.0;
    let x = if leading {
        selection.origin.x - diameter * 0.5
    } else {
        selection.origin.x + selection.size.width - diameter * 0.5
    };
    let bounds = Rect::new(
        Point::new(
            x,
            selection.origin.y + selection.size.height - diameter * 0.5,
        ),
        argui_core::Size::new(diameter, diameter),
    );
    paint_rect(
        display_list,
        region,
        bounds,
        region.style.handle,
        diameter * 0.5,
    );
}

fn paint_rect(
    display_list: &mut DisplayList,
    region: &TextRegion,
    bounds: Rect,
    color: Color,
    radius: f32,
) {
    display_list.push_quad(Quad {
        bounds,
        background: Some(Fill::Solid(color)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::all(radius),
        opacity: 1.0,
        transform: region.transform,
        clips: region.clips.clone(),
    });
}

impl LayoutOutput {
    #[must_use]
    pub fn document_selection_bounds(&self, ui: &UiTree) -> Option<Rect> {
        self.text_regions
            .iter()
            .flat_map(|region| {
                let range = ui.document_selection_range(region.node, region.text_len);
                range
                    .into_iter()
                    .flat_map(|range| region.visual_rects(range.start, range.end))
                    .map(|rect| region.transform.transform_rect(rect))
            })
            .reduce(union)
    }
}

fn union(left: Rect, right: Rect) -> Rect {
    let x = left.origin.x.min(right.origin.x);
    let y = left.origin.y.min(right.origin.y);
    let right_edge = (left.origin.x + left.size.width).max(right.origin.x + right.size.width);
    let bottom = (left.origin.y + left.size.height).max(right.origin.y + right.size.height);
    Rect::new(
        Point::new(x, y),
        argui_core::Size::new(right_edge - x, bottom - y),
    )
}
