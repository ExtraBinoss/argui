use std::sync::Arc;

use argui_core::{Affine2D, Point, Rect, Size, TextPosition};
use argui_paint::{Border, ClipChain, Color, CornerRadii, DisplayList, Fill, Quad};
use argui_text::TextLayout;
use argui_ui::{
    DocumentSelectionEndpoint, DocumentTextPoint, DocumentTextSelection, NodeId,
    TextSelectionHighlight, TextSelectionStyle, UiTree,
};

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
    pub highlight: TextSelectionHighlight,
    /// Number of hit regions emitted through this text's paint position.
    pub interaction_order: usize,
}

/// Viewport-space hit and paint geometry for one touch selection endpoint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionHandleGeometry {
    /// Retained text node containing this endpoint.
    pub node: NodeId,
    /// Directed endpoint that will move when this handle is dragged.
    pub endpoint: DocumentSelectionEndpoint,
    /// Center of the visible handle in viewport coordinates.
    pub center: Point,
    /// Visible handle bounds in viewport coordinates.
    pub visual_bounds: Rect,
    /// Comfortable touch target in viewport coordinates.
    pub hit_bounds: Rect,
}

impl TextRegion {
    /// Returns the text position hit by `point`, respecting transforms and clips.
    ///
    /// * `point` — position in viewport coordinates.
    ///
    /// Returns `None` when the point is clipped or outside the shaped text lines.
    #[must_use]
    pub fn hit_position(&self, point: Point) -> Option<DocumentTextPoint> {
        self.local_point(point)
            .and_then(|point| self.layout.hit_position(point))
            .map(|position| DocumentTextPoint::new(self.node, position))
    }

    /// Returns the nearest text position to `point`, without clip rejection.
    ///
    /// * `point` — position in viewport coordinates.
    #[must_use]
    pub fn closest_position(&self, point: Point) -> DocumentTextPoint {
        let local = self.local_point_unclipped(point);
        DocumentTextPoint::new(self.node, self.layout.closest_position(local))
    }

    /// Returns squared distance from `point` to the nearest shaped line bounds.
    ///
    /// * `point` — position in viewport coordinates.
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

    /// Returns selection rectangles for the half-open byte interval `start..end`.
    ///
    /// * `start` — first selected byte offset.
    /// * `end` — byte offset after the selected text.
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

impl LayoutOutput {
    /// Selectable text under the pointer, excluding later-painted hit surfaces.
    /// A transparent blocker still owns input; its paint alpha is irrelevant.
    ///
    /// * `point` — position in viewport coordinates.
    ///
    /// Returns the topmost selectable text position, if one is not occluded.
    pub fn text_at(&self, point: Point) -> Option<DocumentTextPoint> {
        self.text_regions
            .iter()
            .filter_map(|region| {
                let position = region.hit_position(point)?;
                if self
                    .hit_regions
                    .iter()
                    .skip(region.interaction_order)
                    .any(|hit| hit.contains(point))
                {
                    return None;
                }
                Some((region.interaction_order, position))
            })
            .max_by_key(|(order, _)| *order)
            .map(|(_, position)| position)
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
        paint_rect(
            display_list,
            region,
            *bounds,
            region.highlight.background.clone(),
            region.highlight.radii,
        );
    }
    if ui.document_selection_handles_visible()
        && let Some(selection) = ui.document_selection()
        && ui.has_document_selection()
    {
        for endpoint in [
            DocumentSelectionEndpoint::Anchor,
            DocumentSelectionEndpoint::Focus,
        ] {
            let point = selection_endpoint(selection, endpoint);
            if point.node == region.node
                && let Some((_, bounds)) = handle_geometry(region, endpoint, point)
            {
                paint_handle(display_list, region, bounds);
            }
        }
    }
}

/// Paints one round touch-selection handle inside its text region's transform and clips.
///
/// * `display_list` — destination display list for the handle primitive.
/// * `region` — text region supplying the handle color, transform, and clipping chain.
/// * `bounds` — local-space visible bounds of the handle.
fn paint_handle(display_list: &mut DisplayList, region: &TextRegion, bounds: Rect) {
    let diameter = bounds.size.width;
    paint_rect(
        display_list,
        region,
        bounds,
        Fill::Solid(region.style.handle),
        CornerRadii::all(diameter * 0.5),
    );
}

fn paint_rect(
    display_list: &mut DisplayList,
    region: &TextRegion,
    bounds: Rect,
    background: Fill,
    radii: CornerRadii,
) {
    display_list.push_quad(Quad {
        bounds,
        background: Some(background),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii,
        opacity: 1.0,
        transform: region.transform,
        clips: region.clips.clone(),
    });
}

impl LayoutOutput {
    /// Returns the visible touch selection handles and their hit targets.
    ///
    /// * `ui` — tree containing the current selection and its touch-handle visibility.
    ///
    /// Each endpoint receives a 44-logical-pixel square hit target centered on its painted
    /// handle. Returns no handles when the selection is empty, hidden, clipped, or has no shaped
    /// caret geometry.
    #[must_use]
    pub fn selection_handles(&self, ui: &UiTree) -> Vec<SelectionHandleGeometry> {
        if !ui.document_selection_handles_visible() || !ui.has_document_selection() {
            return Vec::new();
        }
        let Some(selection) = ui.document_selection() else {
            return Vec::new();
        };
        [
            DocumentSelectionEndpoint::Anchor,
            DocumentSelectionEndpoint::Focus,
        ]
        .into_iter()
        .filter_map(|endpoint| {
            let point = selection_endpoint(selection, endpoint);
            let region = self
                .text_regions
                .iter()
                .find(|region| region.node == point.node)?;
            handle_geometry(region, endpoint, point).map(|(geometry, _)| geometry)
        })
        .collect()
    }

    /// Finds the closest visible touch selection handle whose target contains `point`.
    ///
    /// * `ui` — tree containing the current selection and its touch-handle visibility.
    /// * `point` — pointer position in viewport coordinates.
    ///
    /// Returns the nearest matching handle, or `None` when no hit target contains the point.
    #[must_use]
    pub fn selection_handle_at(
        &self,
        ui: &UiTree,
        point: Point,
    ) -> Option<SelectionHandleGeometry> {
        self.selection_handles(ui)
            .into_iter()
            .filter(|handle| handle.hit_bounds.contains(point))
            .min_by(|left, right| {
                distance_squared(left.center, point)
                    .total_cmp(&distance_squared(right.center, point))
            })
    }

    /// Returns the viewport-space union of rectangles in the current document selection.
    ///
    /// * `ui` — tree providing the selected ranges for each text node.
    ///
    /// Returns `None` when the selection has no visible text rectangles.
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

/// Selects the requested directed endpoint from a document selection.
///
/// * `selection` — anchor and focus positions to inspect.
/// * `endpoint` — endpoint identity to return.
///
/// Returns the document text position stored at that endpoint.
fn selection_endpoint(
    selection: DocumentTextSelection,
    endpoint: DocumentSelectionEndpoint,
) -> DocumentTextPoint {
    match endpoint {
        DocumentSelectionEndpoint::Anchor => selection.anchor,
        DocumentSelectionEndpoint::Focus => selection.focus,
    }
}

/// Builds viewport hit geometry and local paint bounds for a selection endpoint.
///
/// * `region` — shaped text region containing the endpoint.
/// * `endpoint` — directed selection endpoint represented by the handle.
/// * `point` — document text position for that endpoint.
///
/// Returns `None` when the endpoint has no caret stop, matching text line, or visible clip point.
fn handle_geometry(
    region: &TextRegion,
    endpoint: DocumentSelectionEndpoint,
    point: DocumentTextPoint,
) -> Option<(SelectionHandleGeometry, Rect)> {
    let stop = region
        .layout
        .stops
        .iter()
        .filter(|stop| stop.position.index == point.position.index)
        .min_by_key(|stop| stop.position != point.position)?;
    let line = region
        .layout
        .lines
        .iter()
        .find(|line| (line.bounds.origin.y - stop.point.y).abs() < 0.01)?;
    let local_center = Point::new(
        region.origin.x + stop.point.x,
        region.origin.y + line.bounds.origin.y + line.bounds.size.height,
    );
    let center = region.transform.transform_point(local_center);
    if !region.clips.contains(center) {
        return None;
    }
    let visual_diameter = 10.0;
    let visual_local = centered_rect(local_center, visual_diameter);
    let hit_bounds = centered_rect(center, 44.0);
    Some((
        SelectionHandleGeometry {
            node: region.node,
            endpoint,
            center,
            visual_bounds: region.transform.transform_rect(visual_local),
            hit_bounds,
        },
        visual_local,
    ))
}

/// Creates a square centered at `center` with the supplied side length.
///
/// * `center` — rectangle center in the caller's coordinate space.
/// * `diameter` — width and height of the returned square.
fn centered_rect(center: Point, diameter: f32) -> Rect {
    Rect::new(
        Point::new(center.x - diameter * 0.5, center.y - diameter * 0.5),
        Size::new(diameter, diameter),
    )
}

/// Computes squared Euclidean distance without taking a square root.
///
/// * `left` — first viewport-space point.
/// * `right` — second viewport-space point.
fn distance_squared(left: Point, right: Point) -> f32 {
    let delta = Point::new(left.x - right.x, left.y - right.y);
    delta.x * delta.x + delta.y * delta.y
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
