use argui_core::{Affine2D, Rect};
use argui_paint::{
    Border, ClipChain, ClipRegion, Color, DisplayList, ImagePrimitive, LayerStyle, Quad, QuadStyle,
    VectorPrimitive,
};
use argui_ui::{EffectScope, Element, ElementKind, HitRegion, UiTree};

use crate::{LayoutNode, LayoutOutput, engine::NodeMap, input, scroll};

#[derive(Clone)]
struct PaintContext {
    transform: Affine2D,
    clips: ClipChain,
}

pub(crate) fn repaint(root: Option<&NodeMap>, ui: &UiTree, output: &mut LayoutOutput) {
    let elements = crate::engine::flattened(ui.root());
    output.display_list.clear();
    output.hit_regions.clear();
    sync_scroll_config(&elements, output);
    let clips = ClipChain::from_regions([ClipRegion::new(output.viewport, Affine2D::IDENTITY)]);
    if let Some(root) = root {
        paint_node(
            root,
            &elements,
            ui,
            output,
            &PaintContext {
                transform: Affine2D::IDENTITY,
                clips,
            },
        );
    }
}

fn paint_node(
    map: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    output: &mut LayoutOutput,
    parent: &PaintContext,
) {
    if parent.clips.is_empty() {
        return;
    }
    let node = output.nodes[map.index];
    let element = elements[node.index];
    let portal;
    let parent = if element.overlay.is_some() {
        let clip = node.clip.unwrap_or(output.viewport);
        portal = PaintContext {
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions([ClipRegion::new(clip, Affine2D::IDENTITY)]),
        };
        &portal
    } else {
        parent
    };
    let transform = parent.transform
        * ui.resolved_transform(node.node, element)
            .affine(node.bounds, element.transform_origin);
    let context = PaintContext {
        transform,
        clips: parent.clips.clone(),
    };
    paint_enter(ui, element, node, output, &context);

    let child_clips = if element.paint.clip == argui_paint::ClipBehavior::Bounds {
        context
            .clips
            .appended(ClipRegion::new(node.bounds, transform))
    } else {
        context.clips.clone()
    };
    let child_context = PaintContext {
        transform,
        clips: child_clips,
    };
    if element.scroll.is_some()
        && let Some(region) = output
            .scroll_regions
            .iter_mut()
            .find(|region| region.node == node.node)
    {
        region.transform = transform;
        region.clips = child_context.clips.clone();
    }
    let mut children = map.children.iter().collect::<Vec<_>>();
    children.sort_by_key(|child| elements[child.index].z_index);
    for child in children {
        paint_node(child, elements, ui, output, &child_context);
    }
    if let Some(region) = output
        .scroll_regions
        .iter()
        .find(|region| region.node == node.node)
        .cloned()
    {
        scroll::paint(&region, &mut output.display_list);
    }
    paint_exit(element, &mut output.display_list);
}

fn paint_enter(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let visual_bounds = context.transform.transform_rect(node.bounds);
    if let Some(layer) = element.layer.clone() {
        begin_layer(&mut output.display_list, layer, visual_bounds);
    }
    begin_scope(
        &mut output.display_list,
        element,
        EffectScope::WholeElement,
        visual_bounds,
    );
    push_hit_region(element, node, output, context);
    push_quad(
        ui.resolved_quad(node.node, element),
        element,
        node.bounds,
        output,
        context,
    );
    push_image(element, node.bounds, output, context);
    push_vector(element, node.bounds, output, context);
    begin_scope(
        &mut output.display_list,
        element,
        EffectScope::Content,
        visual_bounds,
    );

    let content_clips = if element.paint.clip == argui_paint::ClipBehavior::Bounds {
        context
            .clips
            .appended(ClipRegion::new(node.bounds, context.transform))
    } else {
        context.clips.clone()
    };
    let text_input = output
        .text_inputs
        .iter()
        .find(|region| region.node == node.node);
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
            &mut output.display_list,
            element,
            EffectScope::Text,
            visual_bounds,
        );
        output.display_list.push_text_transformed(
            text_index,
            context.transform,
            content_clips.clone(),
        );
        end_layers(&mut output.display_list, layers);
    }
    if let Some(region) = text_input {
        input::paint_caret(
            region,
            &mut output.display_list,
            context.transform,
            &content_clips,
        );
    }
}

fn paint_exit(element: &Element, display_list: &mut DisplayList) {
    end_layers(display_list, scope_count(element, EffectScope::Content));
    end_layers(
        display_list,
        scope_count(element, EffectScope::WholeElement),
    );
    if element.layer.is_some() {
        display_list.end_layer();
    }
}

fn push_quad(
    style: QuadStyle,
    element: &Element,
    bounds: Rect,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let split = scope_count(element, EffectScope::Background) != 0
        || scope_count(element, EffectScope::Border) != 0;
    if !split {
        if style.is_visible() {
            output.display_list.push_quad(quad(style, bounds, context));
        }
        return;
    }
    if style.background.is_some() {
        push_scoped_quad(
            QuadStyle {
                border: None,
                ..style
            },
            element,
            bounds,
            EffectScope::Background,
            output,
            context,
        );
    }
    if style.border.is_some() {
        push_scoped_quad(
            QuadStyle {
                background: None,
                ..style
            },
            element,
            bounds,
            EffectScope::Border,
            output,
            context,
        );
    }
}

fn push_scoped_quad(
    style: QuadStyle,
    element: &Element,
    bounds: Rect,
    scope: EffectScope,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let visual_bounds = context.transform.transform_rect(bounds);
    let layers = begin_scope(&mut output.display_list, element, scope, visual_bounds);
    output.display_list.push_quad(quad(style, bounds, context));
    end_layers(&mut output.display_list, layers);
}

fn quad(style: QuadStyle, bounds: Rect, context: &PaintContext) -> Quad {
    Quad {
        bounds,
        background: style.background,
        border: style.border.unwrap_or(Border::all(0.0, Color::TRANSPARENT)),
        radii: style.radii,
        opacity: style.opacity,
        transform: context.transform,
        clips: context.clips.clone(),
    }
}

fn push_image(element: &Element, bounds: Rect, output: &mut LayoutOutput, context: &PaintContext) {
    let ElementKind::Image {
        image,
        fit,
        sampling,
    } = element.kind
    else {
        return;
    };
    output.display_list.push_image(ImagePrimitive {
        bounds,
        image,
        fit,
        sampling,
        opacity: element.paint.quad.opacity,
        radii: element.paint.quad.radii,
        transform: context.transform,
        clips: context.clips.clone(),
    });
}

fn push_vector(element: &Element, bounds: Rect, output: &mut LayoutOutput, context: &PaintContext) {
    let ElementKind::Vector { vector, progress } = element.kind else {
        return;
    };
    output.display_list.push_vector(VectorPrimitive {
        vector,
        bounds,
        progress,
        opacity: element.paint.quad.opacity,
        transform: context.transform,
        clips: context.clips.clone(),
    });
}

fn begin_scope(
    display_list: &mut DisplayList,
    element: &Element,
    scope: EffectScope,
    bounds: Rect,
) -> usize {
    let effects = element
        .effects
        .iter()
        .filter(|effect| effect.scope == scope);
    let mut count = 0;
    for effect in effects {
        begin_layer(display_list, effect.layer.clone(), bounds);
        count += 1;
    }
    count
}

fn begin_layer(display_list: &mut DisplayList, mut layer: LayerStyle, bounds: Rect) {
    layer.bounds = bounds;
    display_list.begin_layer(layer);
}

fn end_layers(display_list: &mut DisplayList, count: usize) {
    for _ in 0..count {
        display_list.end_layer();
    }
}

fn scope_count(element: &Element, scope: EffectScope) -> usize {
    element
        .effects
        .iter()
        .filter(|effect| effect.scope == scope)
        .count()
}

fn push_hit_region(
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    if let Some(interaction) = element.interaction.as_ref()
        && interaction.enabled
    {
        output.hit_regions.push(HitRegion {
            node: node.node,
            bounds: node.bounds,
            transform: context.transform,
            clips: context.clips.clone(),
            focusable: interaction.focusable,
        });
    }
}

fn sync_scroll_config(elements: &[&Element], output: &mut LayoutOutput) {
    for region in &mut output.scroll_regions {
        if let Some(node) = output.nodes.iter().find(|node| node.node == region.node)
            && let Some(config) = elements[node.index].scroll.clone()
        {
            region.config = config;
        }
    }
}
