use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{Border, ClipChain, ClipRegion, Color, DisplayList, Quad, QuadStyle};
use argui_ui::{NodeId, ScrollConfig, ScrollRegion, ScrollbarRegion};

pub(crate) fn region(
    node: NodeId,
    bounds: Rect,
    clip: Rect,
    content: Size,
    config: ScrollConfig,
    offset: Point,
) -> ScrollRegion {
    let max_offset = Point::new(
        (content.width - bounds.size.width).max(0.0),
        (content.height - bounds.size.height).max(0.0),
    );
    let scrollbar = config
        .scrollbar
        .clone()
        .and_then(|style| vertical_bar(bounds, max_offset.y, offset.y, style));
    ScrollRegion {
        node,
        bounds,
        clip,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(clip, Affine2D::IDENTITY)]),
        max_offset,
        config,
        scrollbar,
    }
}

pub(crate) fn paint(region: &ScrollRegion, display_list: &mut DisplayList) {
    let Some(scrollbar) = region.scrollbar.as_ref() else {
        return;
    };
    push_quad(
        display_list,
        scrollbar.track,
        &region.clips,
        region.transform,
        scrollbar.style.track.clone(),
    );
    push_quad(
        display_list,
        scrollbar.thumb,
        &region.clips,
        region.transform,
        scrollbar.style.thumb.clone(),
    );
}

fn vertical_bar(
    bounds: Rect,
    max_offset: f32,
    offset: f32,
    style: argui_ui::ScrollbarStyle,
) -> Option<ScrollbarRegion> {
    if max_offset <= 0.0 {
        return None;
    }
    let inset = style.inset.max(0.0);
    let width = style.width.max(0.0).min(bounds.size.width);
    let track_height = (bounds.size.height - inset * 2.0).max(0.0);
    if width == 0.0 || track_height == 0.0 {
        return None;
    }
    let track = Rect::new(
        Point::new(
            bounds.origin.x + bounds.size.width - inset - width,
            bounds.origin.y + inset,
        ),
        Size::new(width, track_height),
    );
    let content_height = bounds.size.height + max_offset;
    let proportional = track_height * bounds.size.height / content_height;
    let thumb_height = proportional
        .max(style.min_thumb.min(track_height))
        .min(track_height);
    let travel = track_height - thumb_height;
    let thumb_y = track.origin.y + offset.clamp(0.0, max_offset) / max_offset * travel;
    Some(ScrollbarRegion {
        track,
        thumb: Rect::new(
            Point::new(track.origin.x, thumb_y),
            Size::new(width, thumb_height),
        ),
        style,
    })
}

fn push_quad(
    display_list: &mut DisplayList,
    bounds: Rect,
    clips: &ClipChain,
    transform: Affine2D,
    style: QuadStyle,
) {
    if !style.is_visible() {
        return;
    }
    display_list.push_quad(Quad {
        bounds,
        background: style.background,
        border: style.border.unwrap_or(Border::all(0.0, Color::TRANSPARENT)),
        radii: style.radii,
        opacity: style.opacity,
        transform,
        clips: clips.clone(),
    });
}
