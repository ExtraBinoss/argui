use argui_core::{Point, Rect};
use argui_ui::{FocusTarget, NodeId, ScrollAlignment, ScrollRequest, ScrollTarget};

#[derive(Clone, Copy, Debug)]
pub(super) struct ScrollTrack {
    pub(super) node: NodeId,
    pub(super) from: Point,
    pub(super) to: Point,
}

pub(super) fn scroll_tracks(
    ui: &argui_ui::UiTree,
    layout: &argui_layout::LayoutOutput,
    request: &ScrollRequest,
) -> Vec<ScrollTrack> {
    match &request.target {
        ScrollTarget::Offset { container, offset } => region_for(ui, layout, container)
            .map(|region| ScrollTrack {
                node: region.node,
                from: ui.scroll_offset(region.node),
                to: clamp_point(*offset, region.max_offset),
            })
            .into_iter()
            .collect(),
        ScrollTarget::Rect { container, rect } => region_for(ui, layout, container)
            .map(|region| reveal_track(ui, region, *rect, request))
            .into_iter()
            .collect(),
        ScrollTarget::Element(target) => element_tracks(ui, layout, target, request),
    }
}

fn element_tracks(
    ui: &argui_ui::UiTree,
    layout: &argui_layout::LayoutOutput,
    target: &FocusTarget,
    request: &ScrollRequest,
) -> Vec<ScrollTrack> {
    let Some(node) = ui.resolve_node(target) else {
        return Vec::new();
    };
    let Some(mut rect) = layout
        .nodes
        .iter()
        .find(|candidate| candidate.node == node)
        .map(|candidate| candidate.bounds)
    else {
        return Vec::new();
    };
    let mut tracks = Vec::new();
    let mut cursor = Some(node);
    while let Some(current) = cursor {
        if let Some(region) = layout
            .scroll_regions
            .iter()
            .find(|region| region.node == current)
        {
            let track = reveal_track(ui, region, rect, request);
            rect.origin.x -= track.to.x - track.from.x;
            rect.origin.y -= track.to.y - track.from.y;
            tracks.push(track);
        }
        if layout.portals.iter().any(|portal| portal.node == current) {
            break;
        }
        cursor = ui.parent_of(current);
    }
    tracks
}

fn region_for<'a>(
    ui: &argui_ui::UiTree,
    layout: &'a argui_layout::LayoutOutput,
    target: &FocusTarget,
) -> Option<&'a argui_ui::ScrollRegion> {
    let node = ui.resolve_node(target)?;
    layout
        .scroll_regions
        .iter()
        .find(|region| region.node == node)
}

fn reveal_track(
    ui: &argui_ui::UiTree,
    region: &argui_ui::ScrollRegion,
    target: Rect,
    request: &ScrollRequest,
) -> ScrollTrack {
    let from = ui.scroll_offset(region.node);
    let to = Point::new(
        align_axis(
            from.x,
            region.bounds.origin.x,
            region.bounds.origin.x + region.bounds.size.width,
            target.origin.x - request.margin.left,
            target.origin.x + target.size.width + request.margin.right,
            request.x,
        ),
        align_axis(
            from.y,
            region.bounds.origin.y,
            region.bounds.origin.y + region.bounds.size.height,
            target.origin.y - request.margin.top,
            target.origin.y + target.size.height + request.margin.bottom,
            request.y,
        ),
    );
    ScrollTrack {
        node: region.node,
        from,
        to: clamp_point(to, region.max_offset),
    }
}

pub(super) fn align_axis(
    current: f32,
    viewport_start: f32,
    viewport_end: f32,
    target_start: f32,
    target_end: f32,
    alignment: ScrollAlignment,
) -> f32 {
    current
        + match alignment {
            ScrollAlignment::Start => target_start - viewport_start,
            ScrollAlignment::Center => {
                (target_start + target_end - viewport_start - viewport_end) * 0.5
            }
            ScrollAlignment::End => target_end - viewport_end,
            ScrollAlignment::Nearest if target_start < viewport_start => {
                target_start - viewport_start
            }
            ScrollAlignment::Nearest if target_end > viewport_end => target_end - viewport_end,
            ScrollAlignment::Nearest => 0.0,
        }
}

fn clamp_point(point: Point, maximum: Point) -> Point {
    Point::new(point.x.clamp(0.0, maximum.x), point.y.clamp(0.0, maximum.y))
}
