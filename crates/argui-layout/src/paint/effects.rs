use crate::{LayoutNode, LayoutOutput};
use argui_core::{Affine2D, Rect};
use argui_paint::{DisplayList, LayerStyle, ProfileDomain, RenderObjectId};
use argui_ui::{EffectScope, Element, NodeId, ScrollAxes, ScrollMetrics, ScrollbarGutter, UiTree};

pub(super) fn begin_scope(
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

pub(super) fn begin_layer(
    display_list: &mut DisplayList,
    mut layer: LayerStyle,
    bounds: Rect,
    node: NodeId,
) {
    layer.bounds = bounds;
    if layer.profile.is_none() {
        layer.profile = Some(RenderObjectId::new(ProfileDomain::Ui, node.get()));
    }
    display_list.begin_layer(layer);
}

pub(super) fn end_layers(display_list: &mut DisplayList, count: usize) {
    for _ in 0..count {
        display_list.end_layer();
    }
}

pub(super) fn scope_count(element: &Element, scope: EffectScope) -> usize {
    element
        .effects
        .iter()
        .filter(|effect| effect.scope == scope)
        .count()
}

pub(super) fn begin_scroll(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    transform: Affine2D,
) -> usize {
    if element
        .scroll
        .as_ref()
        .is_none_or(|config| config.effects.is_empty())
    {
        return 0;
    }
    let Some(region) = output
        .scroll_regions
        .iter()
        .find(|region| region.node == node.node)
    else {
        return 0;
    };
    let mut viewport = region.bounds;
    let style = ui.resolved_layout_style(node.node, element);
    if style.scrollbar_gutter == ScrollbarGutter::Stable {
        let gutter = style.scrollbar_width.max(0.0);
        if matches!(region.config.axes, ScrollAxes::Vertical | ScrollAxes::Both) {
            viewport.size.width = (viewport.size.width - gutter).max(0.0);
        }
        if matches!(
            region.config.axes,
            ScrollAxes::Horizontal | ScrollAxes::Both
        ) {
            viewport.size.height = (viewport.size.height - gutter).max(0.0);
        }
    }
    let mut max_offset = region.max_offset;
    match region.config.axes {
        ScrollAxes::Vertical => max_offset.x = 0.0,
        ScrollAxes::Horizontal => max_offset.y = 0.0,
        ScrollAxes::Both => {}
    }
    let metrics = ScrollMetrics {
        viewport,
        transform,
        offset: ui.scroll_offset(node.node),
        max_offset,
    };
    let mut count = 0;
    // Reverse nesting preserves the declaration order at composition time.
    for effect in region.config.effects.iter().rev() {
        if let Some(layer) = effect.resolve(metrics) {
            begin_layer(
                &mut output.display_list,
                layer,
                transform.transform_rect(viewport),
                node.node,
            );
            count += 1;
        }
    }
    count
}
