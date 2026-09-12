use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{Border, ClipChain, ClipRegion, Color, DisplayList, Quad, QuadStyle};
use argui_ui::{
    NodeId, ScrollAxes, ScrollConfig, ScrollRegion, ScrollbarGeometry, ScrollbarRegion,
    ScrollbarVisibility,
};

use crate::engine::NodeMap;

pub(crate) fn content_size(
    tree: &crate::layout_tree::LayoutTree,
    node: &NodeMap,
) -> Result<Size, crate::LayoutError> {
    let layout = tree.layout(node.id)?;
    let mut size = Size::new(
        layout.size.width.max(layout.scrollable_overflow_rect.right),
        layout
            .size
            .height
            .max(layout.scrollable_overflow_rect.bottom),
    );
    for child in node
        .children
        .iter()
        .filter(|child| !crate::overlay::detached(&child.element))
    {
        let child_layout = tree.layout(child.id)?;
        size.width = size
            .width
            .max(child_layout.location.x + child_layout.size.width + layout.padding.right);
        size.height = size
            .height
            .max(child_layout.location.y + child_layout.size.height + layout.padding.bottom);
    }
    Ok(size)
}

pub(crate) fn clipped(node: &NodeMap, parent: Option<Rect>, bounds: Rect) -> Option<Rect> {
    if node.style.overflow.x.clips() || node.style.overflow.y.clips() {
        parent.and_then(|clip| clip.intersection(bounds))
    } else {
        parent
    }
}

pub(crate) fn config(node: &NodeMap, element: &argui_ui::Element) -> Option<ScrollConfig> {
    let axes = match (
        node.style.overflow.x.scrolls(),
        node.style.overflow.y.scrolls(),
    ) {
        (false, false) => return None,
        (true, false) => argui_ui::ScrollAxes::Horizontal,
        (false, true) => argui_ui::ScrollAxes::Vertical,
        (true, true) => argui_ui::ScrollAxes::Both,
    };
    let mut config = element.scroll.as_deref().cloned().unwrap_or_default();
    config.axes = axes;
    Some(config)
}

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
    let scrollbar = config.scrollbar.clone().and_then(|style| {
        (style.visibility != ScrollbarVisibility::Hidden).then(|| ScrollbarRegion {
            vertical: matches!(config.axes, ScrollAxes::Vertical | ScrollAxes::Both)
                .then(|| vertical_bar(bounds, max_offset.y, offset.y, &style))
                .flatten(),
            horizontal: matches!(config.axes, ScrollAxes::Horizontal | ScrollAxes::Both)
                .then(|| horizontal_bar(bounds, max_offset.x, offset.x, &style))
                .flatten(),
            style,
        })
    });
    ScrollRegion {
        node,
        bounds,
        clip,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(clip, Affine2D::IDENTITY)]),
        max_offset,
        config,
        scrollbar,
        interaction_order: 0,
    }
}

pub(crate) fn paint(region: &ScrollRegion, display_list: &mut DisplayList) {
    let Some(scrollbar) = region.scrollbar.as_ref() else {
        return;
    };
    for geometry in [scrollbar.vertical, scrollbar.horizontal]
        .into_iter()
        .flatten()
    {
        push_quad(
            display_list,
            geometry.track,
            &region.clips,
            region.transform,
            scrollbar.style.track.base.clone(),
        );
        push_quad(
            display_list,
            geometry.thumb,
            &region.clips,
            region.transform,
            scrollbar.style.thumb.base.clone(),
        );
    }
}

fn vertical_bar(
    bounds: Rect,
    max_offset: f32,
    offset: f32,
    style: &argui_ui::ScrollbarStyle,
) -> Option<ScrollbarGeometry> {
    if max_offset <= 0.0 {
        return None;
    }
    let insets = argui_ui::Sides {
        top: style.insets.top.max(0.0),
        right: style.insets.right.max(0.0),
        bottom: style.insets.bottom.max(0.0),
        left: style.insets.left.max(0.0),
    };
    let available_width = (bounds.size.width - insets.left - insets.right).max(0.0);
    let width = style.width.max(0.0).min(available_width);
    let track_height = (bounds.size.height - insets.top - insets.bottom).max(0.0);
    if width == 0.0 || track_height == 0.0 {
        return None;
    }
    let track = Rect::new(
        Point::new(
            bounds.origin.x + bounds.size.width - insets.right - width,
            bounds.origin.y + insets.top,
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
    Some(ScrollbarGeometry {
        track,
        thumb: Rect::new(
            Point::new(track.origin.x, thumb_y),
            Size::new(width, thumb_height),
        ),
    })
}

fn horizontal_bar(
    bounds: Rect,
    max_offset: f32,
    offset: f32,
    style: &argui_ui::ScrollbarStyle,
) -> Option<ScrollbarGeometry> {
    if max_offset <= 0.0 {
        return None;
    }
    let insets = argui_ui::Sides {
        top: style.insets.top.max(0.0),
        right: style.insets.right.max(0.0),
        bottom: style.insets.bottom.max(0.0),
        left: style.insets.left.max(0.0),
    };
    let available_height = (bounds.size.height - insets.top - insets.bottom).max(0.0);
    let width = style.width.max(0.0).min(available_height);
    let track_width = (bounds.size.width - insets.left - insets.right).max(0.0);
    if width == 0.0 || track_width == 0.0 {
        return None;
    }
    let track = Rect::new(
        Point::new(
            bounds.origin.x + insets.left,
            bounds.origin.y + bounds.size.height - insets.bottom - width,
        ),
        Size::new(track_width, width),
    );
    let content_width = bounds.size.width + max_offset;
    let thumb_width = (track_width * bounds.size.width / content_width)
        .max(style.min_thumb.min(track_width))
        .min(track_width);
    let travel = track_width - thumb_width;
    let thumb_x = track.origin.x + offset.clamp(0.0, max_offset) / max_offset * travel;
    Some(ScrollbarGeometry {
        track,
        thumb: Rect::new(
            Point::new(thumb_x, track.origin.y),
            Size::new(thumb_width, width),
        ),
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
