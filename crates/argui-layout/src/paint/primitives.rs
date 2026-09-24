use argui_core::Rect;
use argui_paint::{Border, Color, ImagePrimitive, Quad, QuadStyle, VectorPrimitive};
use argui_ui::{EffectScope, Element, ElementKind, UiTree};

use crate::{LayoutNode, LayoutOutput};

use super::{PaintContext, effects};

/// Emits the element quad, splitting background and border effect scopes when needed.
pub(super) fn push_quad(
    ui: &UiTree,
    style: QuadStyle,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let split = effects::scope_count(element, EffectScope::Background) != 0
        || effects::scope_count(element, EffectScope::Border) != 0;
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

/// Emits one quad component inside its authored effect scope.
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
    let layers = effects::begin_scope(
        ui,
        &mut output.display_list,
        element,
        scope,
        visual_bounds,
        context.clip_bounds,
        node.node,
    );
    output
        .display_list
        .push_quad(quad(style, node.bounds, context));
    effects::end_layers(&mut output.display_list, layers);
}

/// Builds a renderer-neutral quad from resolved style and paint context.
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

/// Emits an image primitive when `element` is an image.
pub(super) fn push_image(
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

/// Emits a vector primitive when `element` is a vector.
pub(super) fn push_vector(
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
