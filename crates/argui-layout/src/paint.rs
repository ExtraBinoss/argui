use argui_core::{Affine2D, Rect};
use argui_paint::{
    Border, ClipChain, ClipRegion, Color, DisplayList, ImagePrimitive, LayerStyle, ProfileDomain,
    Quad, QuadStyle, RenderObjectId, VectorPrimitive,
};
use argui_ui::{EffectScope, Element, ElementKind, HitRegion, NodeId, PointerEvents, UiTree};
use std::collections::HashMap;

use crate::{LayoutNode, LayoutOutput, engine::NodeMap, input, scroll};

mod sync;

#[derive(Clone, Debug, PartialEq)]
struct PaintContext {
    transform: Affine2D,
    clips: ClipChain,
    hit_allowed: bool,
}

#[derive(Clone, Debug)]
struct ScrollPaintUpdate {
    node: NodeId,
    transform: Affine2D,
    clips: ClipChain,
    interaction_order: usize,
}

#[derive(Clone, Debug)]
struct CachedFragment {
    element: Element,
    node: LayoutNode,
    parent: PaintContext,
    commands: Vec<argui_paint::DisplayCommand>,
    hit_regions: Vec<HitRegion>,
    scroll_updates: Vec<ScrollPaintUpdate>,
    cacheable: bool,
    visual_revision: u64,
    selection_active: bool,
    selection_revision: u64,
}

#[derive(Default, Debug)]
pub(crate) struct PaintCache {
    fragments: HashMap<NodeId, CachedFragment>,
    pub(crate) visited: usize,
    pub(crate) reused: usize,
    pub(crate) reused_commands: usize,
}

pub(crate) fn repaint(
    root: Option<&NodeMap>,
    ui: &UiTree,
    output: &mut LayoutOutput,
    cache: &mut PaintCache,
) {
    let elements = crate::engine::flattened(ui.root());
    output.display_list.clear();
    output.hit_regions.clear();
    cache.visited = 0;
    cache.reused = 0;
    cache.reused_commands = 0;
    sync::scroll_config(&elements, ui, output);
    sync::text_colors(&elements, ui, output);
    let clips = ClipChain::from_regions([ClipRegion::new(output.viewport, Affine2D::IDENTITY)]);
    if let Some(root) = root {
        let mut scroll_updates = Vec::new();
        paint_node(
            root,
            &elements,
            ui,
            output,
            &PaintContext {
                transform: Affine2D::IDENTITY,
                clips,
                hit_allowed: true,
            },
            cache,
            &mut scroll_updates,
        );
    }
    output.paint_stats = crate::PaintStats {
        visited_subtrees: cache.visited,
        reused_subtrees: cache.reused,
        reused_commands: cache.reused_commands,
    };
}

fn paint_node(
    map: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    output: &mut LayoutOutput,
    parent: &PaintContext,
    cache: &mut PaintCache,
    scroll_updates: &mut Vec<ScrollPaintUpdate>,
) -> bool {
    cache.visited += 1;
    if map.style.display == argui_ui::Display::None {
        return true;
    }
    if parent.clips.is_empty() {
        return true;
    }
    let node = output.nodes[map.index];
    let element = elements[node.index];
    let selection_active = ui.document_selection_intersects(map.index, map.subtree_len);
    if let Some(fragment) = cache.fragments.get(&node.node)
        && fragment.cacheable
        && fragment.element.ptr_eq(element)
        && fragment.node == node
        && fragment.parent == *parent
        && fragment.visual_revision == ui.visual_revision(node.node)
        && (fragment.selection_revision == ui.document_selection_revision()
            || (!fragment.selection_active && !selection_active))
    {
        output.display_list.extend(fragment.commands.clone());
        output.hit_regions.extend(fragment.hit_regions.clone());
        for update in &fragment.scroll_updates {
            apply_scroll_update(output, update);
            scroll_updates.push(update.clone());
        }
        cache.reused += 1;
        cache.reused_commands += fragment.commands.len();
        return true;
    }
    let command_start = output.display_list.len();
    let hit_start = output.hit_regions.len();
    let scroll_start = scroll_updates.len();
    let portal;
    let parent = if element.overlay.is_some() {
        let clip = node.clip.unwrap_or(output.viewport);
        portal = PaintContext {
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions([ClipRegion::new(clip, Affine2D::IDENTITY)]),
            hit_allowed: parent.hit_allowed,
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
        hit_allowed: parent.hit_allowed,
    };
    paint_enter(
        ui,
        element,
        node,
        output,
        &context,
        map.style.overflow.x.clips() || map.style.overflow.y.clips(),
    );

    let child_clips = if map.style.overflow.x.clips() || map.style.overflow.y.clips() {
        let radii = ui.resolved_quad(node.node, element).radii;
        context
            .clips
            .appended(ClipRegion::rounded(node.bounds, transform, radii))
    } else {
        context.clips.clone()
    };
    let child_context = PaintContext {
        transform,
        clips: child_clips,
        hit_allowed: context.hit_allowed
            && !matches!(
                element.hit_test.pointer_events,
                PointerEvents::None | PointerEvents::BoxOnly
            ),
    };
    if (map.style.overflow.x.scrolls() || map.style.overflow.y.scrolls())
        && let Some(region) = output
            .scroll_regions
            .iter_mut()
            .find(|region| region.node == node.node)
    {
        region.transform = transform;
        region.clips = child_context.clips.clone();
        scroll_updates.push(ScrollPaintUpdate {
            node: node.node,
            transform,
            clips: child_context.clips.clone(),
            interaction_order: 0,
        });
    }
    let mut children = map.children.iter().collect::<Vec<_>>();
    children.sort_by_key(|child| elements[child.index].z_index);
    let mut cacheable = element.bindings.is_empty()
        && !element.has_state_animation()
        && !matches!(element.kind, ElementKind::TextEditor { .. })
        && !map.style.overflow.x.scrolls()
        && !map.style.overflow.y.scrolls();
    for child in children {
        cacheable &= paint_node(
            child,
            elements,
            ui,
            output,
            &child_context,
            cache,
            scroll_updates,
        );
    }
    let interaction_order = output.hit_regions.len();
    if let Some(region) = output
        .scroll_regions
        .iter_mut()
        .find(|region| region.node == node.node)
    {
        region.interaction_order = interaction_order;
        if let Some(update) = scroll_updates
            .iter_mut()
            .rev()
            .find(|update| update.node == node.node)
        {
            update.interaction_order = interaction_order;
        }
        scroll::paint(region, &mut output.display_list);
    }
    paint_exit(element, &mut output.display_list);
    if cacheable {
        remove_descendant_fragments(map, cache);
        cache.fragments.insert(
            node.node,
            CachedFragment {
                element: element.clone(),
                node,
                parent: parent.clone(),
                commands: output.display_list.commands()[command_start..].to_vec(),
                hit_regions: output.hit_regions[hit_start..].to_vec(),
                scroll_updates: scroll_updates[scroll_start..].to_vec(),
                cacheable,
                visual_revision: ui.visual_revision(node.node),
                selection_active,
                selection_revision: ui.document_selection_revision(),
            },
        );
    } else {
        cache.fragments.remove(&node.node);
    }
    cacheable
}

fn remove_descendant_fragments(map: &NodeMap, cache: &mut PaintCache) {
    for child in &map.children {
        cache.fragments.remove(&child.node);
    }
}

fn apply_scroll_update(output: &mut LayoutOutput, update: &ScrollPaintUpdate) {
    if let Some(region) = output
        .scroll_regions
        .iter_mut()
        .find(|region| region.node == update.node)
    {
        region.transform = update.transform;
        region.clips = update.clips.clone();
        region.interaction_order = update.interaction_order;
    }
}

fn paint_enter(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
    clips_content: bool,
) {
    let visual_bounds = context.transform.transform_rect(node.bounds);
    if let Some(layer) = element.layer.clone() {
        begin_layer(
            &mut output.display_list,
            ui.resolved_layer(node.node, element, &layer),
            visual_bounds,
            node.node,
        );
    }
    begin_scope(
        ui,
        &mut output.display_list,
        element,
        EffectScope::WholeElement,
        visual_bounds,
        node.node,
    );
    push_hit_region(element, node, output, context);
    push_quad(
        ui,
        ui.resolved_quad(node.node, element),
        element,
        node,
        output,
        context,
    );
    push_image(ui, element, node, output, context);
    push_vector(ui, element, node, output, context);
    begin_scope(
        ui,
        &mut output.display_list,
        element,
        EffectScope::Content,
        visual_bounds,
        node.node,
    );

    let content_clips = if clips_content {
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
    if let Some(index) = output
        .text_regions
        .iter()
        .position(|region| region.node == node.node)
    {
        let region = &mut output.text_regions[index];
        region.transform = context.transform;
        region.clips = content_clips.clone();
        region.interaction_order = output.display_list.len();
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
            node.node,
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
            ui,
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
    ui: &UiTree,
    style: QuadStyle,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let split = scope_count(element, EffectScope::Background) != 0
        || scope_count(element, EffectScope::Border) != 0;
    if !split {
        if style.is_visible() {
            output
                .display_list
                .push_quad(quad(style, node.bounds, context));
        }
        return;
    }
    if style.background.is_some() {
        push_scoped_quad(
            ui,
            QuadStyle {
                border: None,
                ..style
            },
            element,
            node,
            EffectScope::Background,
            output,
            context,
        );
    }
    if style.border.is_some() {
        push_scoped_quad(
            ui,
            QuadStyle {
                background: None,
                ..style
            },
            element,
            node,
            EffectScope::Border,
            output,
            context,
        );
    }
}

fn push_scoped_quad(
    ui: &UiTree,
    style: QuadStyle,
    element: &Element,
    node: LayoutNode,
    scope: EffectScope,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let visual_bounds = context.transform.transform_rect(node.bounds);
    let layers = begin_scope(
        ui,
        &mut output.display_list,
        element,
        scope,
        visual_bounds,
        node.node,
    );
    output
        .display_list
        .push_quad(quad(style, node.bounds, context));
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

fn push_image(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let ElementKind::Image {
        image,
        fit,
        sampling,
    } = element.kind
    else {
        return;
    };
    output.display_list.push_image(ImagePrimitive {
        bounds: node.bounds,
        image,
        fit,
        sampling,
        opacity: ui.resolved_quad(node.node, element).opacity,
        radii: ui.resolved_quad(node.node, element).radii,
        transform: context.transform,
        clips: context.clips.clone(),
    });
}

fn push_vector(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let ElementKind::Vector { vector, fit, color } = element.kind else {
        return;
    };
    output.display_list.push_vector(VectorPrimitive {
        vector,
        bounds: node.bounds,
        fit,
        color: ui.resolved_vector_color(node.node, color),
        opacity: ui.resolved_quad(node.node, element).opacity,
        transform: context.transform,
        clips: context.clips.clone(),
    });
}

fn begin_scope(
    ui: &UiTree,
    display_list: &mut DisplayList,
    element: &Element,
    scope: EffectScope,
    bounds: Rect,
    node: NodeId,
) -> usize {
    let effects = element
        .effects
        .iter()
        .filter(|effect| effect.scope == scope);
    let mut count = 0;
    for effect in effects {
        begin_layer(
            display_list,
            ui.resolved_layer(node, element, &effect.layer),
            bounds,
            node,
        );
        count += 1;
    }
    count
}

fn begin_layer(display_list: &mut DisplayList, mut layer: LayerStyle, bounds: Rect, node: NodeId) {
    layer.bounds = bounds;
    if layer.profile.is_none() {
        layer.profile = Some(RenderObjectId::new(ProfileDomain::Ui, node.get()));
    }
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
    let own_allowed = context.hit_allowed
        && !matches!(
            element.hit_test.pointer_events,
            PointerEvents::None | PointerEvents::ContentsOnly
        );
    let listens_to_pointer = element
        .event_listeners
        .iter()
        .any(|listener| listener.event.requires_hit_test());
    if own_allowed && (element.interaction.is_some() || listens_to_pointer) {
        let interaction = element.interaction.clone().unwrap_or_default();
        output.hit_regions.push(HitRegion {
            node: node.node,
            bounds: node.bounds,
            transform: context.transform,
            clips: context.clips.clone(),
            shape: element.hit_test.shape,
            slop: element.hit_test.slop,
            enabled: interaction.enabled,
            focusable: interaction.enabled && interaction.focusable,
            cursor: interaction.cursor,
            gestures: interaction.gestures,
            window_drag: interaction.window_drag,
        });
    }
}
