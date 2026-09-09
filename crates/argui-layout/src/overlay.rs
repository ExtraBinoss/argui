use crate::layout_tree::LayoutTree;
use std::collections::HashMap;

use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{AnchorWidth, Element, PortalTarget, UiTree};

use crate::{LayoutError, LayoutOutput, PortalLayout, engine::NodeMap, scroll};

#[derive(Clone, Copy, Debug)]
pub(crate) enum PortalConstraint {
    Bounds {
        node: taffy::NodeId,
        size: Size,
        anchor_width: AnchorWidth,
    },
    Fill {
        node: taffy::NodeId,
        size: Size,
    },
}

pub(crate) fn constraints(
    tree: &LayoutTree,
    root: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    viewport: Rect,
) -> Result<Vec<PortalConstraint>, LayoutError> {
    let mut bounds = vec![Rect::default(); elements.len()];
    collect_bounds(
        tree,
        root,
        ui,
        Point::default(),
        Point::default(),
        &mut bounds,
    )?;
    let anchors = elements
        .iter()
        .enumerate()
        .filter_map(|(index, element)| element.key.as_deref().map(|key| (key, bounds[index])))
        .collect::<HashMap<_, _>>();
    let mut portals = Vec::new();
    collect(root, elements, &mut portals);
    let mut output = Vec::with_capacity(portals.len());
    for map in portals {
        let element = elements[map.index];
        let Some(portal) = &element.portal else {
            continue;
        };
        let desired = bounds[map.index].size;
        let constraint = match &portal.target {
            PortalTarget::Layout => continue,
            PortalTarget::Anchor(anchor) => {
                let Some(anchor_bounds) = anchors.get(anchor.key.as_str()).copied() else {
                    continue;
                };
                let placed = anchor.placement.place(
                    viewport,
                    anchor_bounds,
                    desired,
                    ui.resolved_layout_style(map.node, element)
                        .writing_direction,
                );
                PortalConstraint::Bounds {
                    node: map.id,
                    size: placed.bounds.size,
                    anchor_width: anchor.placement.anchor_width,
                }
            }
            PortalTarget::Rect { bounds, placement } => {
                let placed = placement.place(
                    viewport,
                    *bounds,
                    desired,
                    ui.resolved_layout_style(map.node, element)
                        .writing_direction,
                );
                PortalConstraint::Bounds {
                    node: map.id,
                    size: placed.bounds.size,
                    anchor_width: placement.anchor_width,
                }
            }
            PortalTarget::Viewport(placement) => {
                let placed = placement.place(viewport, desired);
                match placement {
                    argui_ui::ViewportPlacement::Fill { .. } => PortalConstraint::Fill {
                        node: map.id,
                        size: placed.size,
                    },
                    argui_ui::ViewportPlacement::Positioned { .. } => PortalConstraint::Bounds {
                        node: map.id,
                        size: placed.size,
                        anchor_width: AnchorWidth::Content,
                    },
                }
            }
        };
        output.push(constraint);
    }
    Ok(output)
}

fn collect_bounds(
    tree: &LayoutTree,
    node: &NodeMap,
    ui: &UiTree,
    parent: Point,
    translation: Point,
    output: &mut [Rect],
) -> Result<(), LayoutError> {
    let layout = tree.layout(node.id)?;
    let layout_origin = Point::new(parent.x + layout.location.x, parent.y + layout.location.y);
    output[node.index] = Rect::new(
        Point::new(
            layout_origin.x - translation.x,
            layout_origin.y - translation.y,
        ),
        Size::new(layout.size.width, layout.size.height),
    );
    let scroll = ui.scroll_offset(node.node);
    let child_translation = Point::new(translation.x + scroll.x, translation.y + scroll.y);
    for child in &node.children {
        collect_bounds(tree, child, ui, layout_origin, child_translation, output)?;
    }
    Ok(())
}

pub(crate) fn resolve(
    tree: &LayoutTree,
    root: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let mut overlays = Vec::new();
    collect(root, elements, &mut overlays);
    output.portals.clear();
    let anchors = elements
        .iter()
        .enumerate()
        .filter_map(|(index, element)| element.key.as_deref().map(|key| (key, index)))
        .collect::<HashMap<_, _>>();
    for map in overlays {
        let element = elements[map.index];
        let Some(portal) = element.portal.as_ref() else {
            continue;
        };
        let viewport = output.viewport;
        let desired = output.nodes[map.index].bounds.size;
        let (placed, anchor_key, requested, resolved, available, constrained) = match &portal.target
        {
            PortalTarget::Layout => {
                let current = output.nodes[map.index].bounds;
                translate(map, output, Point::default(), viewport);
                output.nodes[map.index].bounds.size = current.size;
                output.portals.push(PortalLayout {
                    node: map.node,
                    layer: portal.layer,
                    anchor: None,
                    requested: None,
                    resolved: None,
                    bounds: current,
                    available_size: viewport.size,
                    constrained_width: false,
                    constrained_height: false,
                });
                continue;
            }
            PortalTarget::Anchor(anchor) => {
                let Some(anchor_index) = anchors.get(anchor.key.as_str()).copied() else {
                    continue;
                };
                let result = anchor.placement.place(
                    viewport,
                    output.nodes[anchor_index].bounds,
                    desired,
                    ui.resolved_layout_style(map.node, element)
                        .writing_direction,
                );
                (
                    result.bounds,
                    Some(anchor.key.clone()),
                    Some(anchor.placement.preferred),
                    Some(result.placement),
                    result.available_size,
                    (result.constrained_width, result.constrained_height),
                )
            }
            PortalTarget::Rect { bounds, placement } => {
                let result = placement.place(
                    viewport,
                    *bounds,
                    desired,
                    ui.resolved_layout_style(map.node, element)
                        .writing_direction,
                );
                (
                    result.bounds,
                    None,
                    Some(placement.preferred),
                    Some(result.placement),
                    result.available_size,
                    (result.constrained_width, result.constrained_height),
                )
            }
            PortalTarget::Viewport(placement) => {
                let bounds = placement.place(viewport, desired);
                (
                    bounds,
                    None,
                    None,
                    None,
                    bounds.size,
                    (
                        bounds.size.width < desired.width,
                        bounds.size.height < desired.height,
                    ),
                )
            }
        };
        let current = output.nodes[map.index].bounds;
        let delta = Point::new(
            placed.origin.x - current.origin.x,
            placed.origin.y - current.origin.y,
        );
        translate(map, output, delta, viewport);
        output.nodes[map.index].bounds.size = placed.size;
        output.portals.push(PortalLayout {
            node: map.node,
            layer: portal.layer,
            anchor: anchor_key,
            requested,
            resolved,
            bounds: placed,
            available_size: available,
            constrained_width: constrained.0,
            constrained_height: constrained.1,
        });

        if let Some(config) = overlay_scroll_config(map, element)
            && let Some(clip) = viewport.intersection(output.nodes[map.index].bounds)
        {
            let content = super::engine::content_size(tree, map)?;
            let region = scroll::region(
                map.node,
                output.nodes[map.index].bounds,
                clip,
                content,
                ui.resolved_scroll_config(map.node, &config),
                ui.scroll_offset(map.node),
            );
            if let Some(index) = output
                .scroll_regions
                .iter()
                .position(|candidate| candidate.node == map.node)
            {
                output.scroll_regions[index] = region;
            } else {
                output.scroll_regions.push(region);
            }
        }
    }
    Ok(())
}

fn collect<'a>(map: &'a NodeMap, elements: &[&Element], output: &mut Vec<&'a NodeMap>) {
    if elements[map.index].portal.is_some() {
        output.push(map);
    }
    for child in &map.children {
        collect(child, elements, output);
    }
}

fn translate(map: &NodeMap, output: &mut LayoutOutput, delta: Point, incoming_clip: Rect) {
    let node = &mut output.nodes[map.index];
    node.bounds.origin = add(node.bounds.origin, delta);
    node.clip = Some(incoming_clip);
    let bounds = node.bounds;
    let text_index = node.text_index;
    let content_clip = if map.style.overflow.x.clips() || map.style.overflow.y.clips() {
        incoming_clip.intersection(bounds).unwrap_or_default()
    } else {
        incoming_clip
    };
    if let Some(index) = text_index {
        let block = &mut output.text.blocks_mut()[index];
        block.bounds.origin = add(block.bounds.origin, delta);
        block.clip = content_clip;
    }
    if let Some(input) = output
        .text_inputs
        .iter_mut()
        .find(|input| input.node == map.node)
    {
        input.translate(delta, content_clip);
    }
    if let Some(region) = output
        .text_regions
        .iter_mut()
        .find(|region| region.node == map.node)
    {
        region.translate(delta);
        region.clips = ClipChain::from_regions([ClipRegion::new(content_clip, Affine2D::IDENTITY)]);
    }
    if let Some(region) = output
        .scroll_regions
        .iter_mut()
        .find(|region| region.node == map.node)
    {
        region.bounds.origin = add(region.bounds.origin, delta);
        region.clip = incoming_clip.intersection(bounds).unwrap_or_default();
        region.clips = ClipChain::from_regions([ClipRegion::new(region.clip, Affine2D::IDENTITY)]);
        if let Some(scrollbar) = &mut region.scrollbar {
            for geometry in [&mut scrollbar.vertical, &mut scrollbar.horizontal]
                .into_iter()
                .flatten()
            {
                geometry.track.origin = add(geometry.track.origin, delta);
                geometry.thumb.origin = add(geometry.thumb.origin, delta);
            }
        }
    }
    let child_clip = if map.style.overflow.x.clips() || map.style.overflow.y.clips() {
        content_clip
    } else {
        incoming_clip
    };
    for child in &map.children {
        translate(child, output, delta, child_clip);
    }
}

fn overlay_scroll_config(map: &NodeMap, element: &Element) -> Option<argui_ui::ScrollConfig> {
    let axes = match (
        map.style.overflow.x.scrolls(),
        map.style.overflow.y.scrolls(),
    ) {
        (false, false) => return None,
        (true, false) => argui_ui::ScrollAxes::Horizontal,
        (false, true) => argui_ui::ScrollAxes::Vertical,
        (true, true) => argui_ui::ScrollAxes::Both,
    };
    let mut config = element.scroll.clone().unwrap_or_default();
    config.axes = axes;
    Some(config)
}

fn add(left: Point, right: Point) -> Point {
    Point::new(left.x + right.x, left.y + right.y)
}
