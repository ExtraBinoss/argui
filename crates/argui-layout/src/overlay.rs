use argui_core::{Affine2D, Point, Rect};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{Element, UiTree};
use taffy::TaffyTree;

use crate::{LayoutError, LayoutOutput, engine::NodeMap, scroll};

pub(crate) fn resolve(
    tree: &TaffyTree<usize>,
    root: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let mut overlays = Vec::new();
    collect(root, elements, &mut overlays);
    for map in overlays {
        let element = elements[map.index];
        let Some(anchor) = element.overlay.as_ref() else {
            continue;
        };
        let Some(anchor_index) = elements
            .iter()
            .position(|element| element.key.as_deref() == Some(anchor.key.as_str()))
        else {
            continue;
        };
        let viewport = output.nodes[map.index].clip.unwrap_or(output.viewport);
        let desired = output.nodes[map.index].bounds.size;
        let placed = anchor
            .placement
            .place(viewport, output.nodes[anchor_index].bounds, desired);
        let current = output.nodes[map.index].bounds;
        let delta = Point::new(
            placed.bounds.origin.x - current.origin.x,
            placed.bounds.origin.y - current.origin.y,
        );
        translate(map, output, delta, viewport);
        output.nodes[map.index].bounds.size = placed.bounds.size;

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
    if elements[map.index].overlay.is_some() {
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
        .scroll_regions
        .iter_mut()
        .find(|region| region.node == map.node)
    {
        region.bounds.origin = add(region.bounds.origin, delta);
        region.clip = incoming_clip.intersection(bounds).unwrap_or_default();
        region.clips = ClipChain::from_regions([ClipRegion::new(region.clip, Affine2D::IDENTITY)]);
        if let Some(scrollbar) = &mut region.scrollbar {
            scrollbar.track.origin = add(scrollbar.track.origin, delta);
            scrollbar.thumb.origin = add(scrollbar.thumb.origin, delta);
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
