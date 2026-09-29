//! Element visuals, clips, content layers, and input presentation.

use super::effects::{begin_layer, begin_scope, end_layers, scope_count};
use super::primitives::{push_image, push_quad, push_vector};
use super::{PaintContext, effects, geometry, gpu_canvas};
use crate::{LayoutNode, LayoutOutput, input};
use argui_paint::{ClipRegion, CompositorId, DisplayList, LayerMask};
use argui_ui::{EffectScope, Element, UiTree};

pub(super) fn paint_enter(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
    clips_content: bool,
    composited: bool,
) -> usize {
    let visual_bounds = context.transform.transform_rect(node.bounds);
    if element
        .desktop_backdrop
        .is_some_and(|backdrop| backdrop.blur)
    {
        output.desktop_backdrops.push(crate::DesktopBackdropRegion {
            node: node.node,
            shape: context.clips.appended(owned_clip(
                ClipRegion::rounded(
                    node.bounds,
                    context.transform,
                    ui.resolved_quad(node.node, element).radii,
                ),
                context.compositor_owner,
            )),
        });
    }
    if let Some(layer) = &element.layer {
        let mut layer = ui.resolved_layer(node.node, element, layer);
        if composited {
            // Group opacity is applied by the retained compositor wrapper.
            layer.opacity = 1.0;
        }
        if layer.mask == LayerMask::None {
            let radii = ui.resolved_quad(node.node, element).radii;
            if radii.top_left > 0.0
                || radii.top_right > 0.0
                || radii.bottom_right > 0.0
                || radii.bottom_left > 0.0
            {
                layer.mask = LayerMask::Rounded(radii);
            }
        }
        begin_layer(
            &mut output.display_list,
            layer,
            visual_bounds,
            context.clip_bounds,
            node.node,
        );
    }
    begin_scope(
        ui,
        &mut output.display_list,
        element,
        EffectScope::WholeElement,
        visual_bounds,
        context.clip_bounds,
        node.node,
    );
    geometry::push_hit_region(element, node, output, context);
    push_quad(
        ui,
        ui.resolved_quad(node.node, element),
        element,
        node,
        output,
        context,
    );
    push_image(ui, element, node, output, context);
    gpu_canvas::push(ui, element, node, output, context);
    push_vector(ui, element, node, output, context);
    begin_scope(
        ui,
        &mut output.display_list,
        element,
        EffectScope::Content,
        visual_bounds,
        context.clip_bounds,
        node.node,
    );

    let scroll_layers = effects::begin_scroll(
        ui,
        element,
        node,
        output,
        context.transform,
        context.clip_bounds,
    );
    let content_clips = if clips_content {
        context.clips.appended(owned_clip(
            ClipRegion::new(node.bounds, context.transform),
            context.compositor_owner,
        ))
    } else {
        context.clips.clone()
    };
    let text_input = output
        .text_inputs
        .iter()
        .find(|region| region.node == node.node);
    if let Some(index) = output
        .text_regions
        .iter()
        .position(|region| region.node == node.node)
    {
        let region = &mut output.text_regions[index];
        region.transform = context.transform;
        region.clips = content_clips.clone();
        region.interaction_order = output.hit_regions.len();
        crate::selection::paint(ui, region, &mut output.display_list);
    }
    if let Some(region) = text_input {
        input::paint_selection(
            region,
            &mut output.display_list,
            context.transform,
            &content_clips,
        );
    }
    if let Some(text_index) = node.text_index {
        let layers = begin_scope(
            ui,
            &mut output.display_list,
            element,
            EffectScope::Text,
            visual_bounds,
            context.clip_bounds,
            node.node,
        );
        output.display_list.push_text_with_backdrop(
            text_index,
            context.transform,
            content_clips.clone(),
            context.backdrop,
        );
        end_layers(&mut output.display_list, layers);
    }
    if let Some(region) = text_input {
        input::paint_caret(
            ui,
            region,
            &mut output.display_list,
            context.transform,
            &content_clips,
        );
    }
    scroll_layers
}

/// Associates a clip with the nearest retained compositor layer, when present.
pub(super) fn owned_clip(mut clip: ClipRegion, owner: Option<CompositorId>) -> ClipRegion {
    clip.compositor = owner;
    clip
}

pub(super) fn paint_exit(element: &Element, display_list: &mut DisplayList) {
    end_layers(display_list, scope_count(element, EffectScope::Content));
    end_layers(
        display_list,
        scope_count(element, EffectScope::WholeElement),
    );
    if element.layer.is_some() {
        display_list.end_layer();
    }
}
