//! Generic authored effect attachment shared by compiled and live renderers.

use argui_core::Rect;
use argui_paint::{EffectInstance, Filter, LayerStyle};
use argui_ui::{EffectScope, Element};

/// Region of an element affected by a custom visual shader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualEffectTarget {
    /// The element and its descendants.
    WholeElement,
    /// The element's background brush.
    Background,
    /// The element's painted border.
    Border,
    /// The element's child content.
    Content,
    /// Text painted directly by the element.
    Text,
    /// Pixels painted behind the element.
    Backdrop,
}

/// Applies an effect to the complete visual output of an element.
///
/// `element` supplies the visual and `effect` supplies the renderer definition
/// and parameter values. Returns the decorated element.
#[must_use]
pub fn apply_visual_effect(element: Element, effect: EffectInstance) -> Element {
    apply_visual_effect_scoped(element, effect, VisualEffectTarget::WholeElement)
}

/// Applies an effect to one visual region of an element.
///
/// `element` may be any visual kind, including containers, text, raster images,
/// and vectors. `effect` carries the renderer definition ID and its current
/// ordered parameter values. `target` selects a paint region or the already
/// painted backdrop. Returns the element with one additional compositing effect;
/// existing effects remain in order.
#[must_use]
pub fn apply_visual_effect_scoped(
    element: Element,
    effect: EffectInstance,
    target: VisualEffectTarget,
) -> Element {
    let scope = match target {
        VisualEffectTarget::WholeElement => EffectScope::WholeElement,
        VisualEffectTarget::Background => EffectScope::Background,
        VisualEffectTarget::Border => EffectScope::Border,
        VisualEffectTarget::Content => EffectScope::Content,
        VisualEffectTarget::Text => EffectScope::Text,
        VisualEffectTarget::Backdrop => return element.backdrop_filter(Filter::Effect(effect)),
    };
    element.effect(
        scope,
        LayerStyle::new(Rect::default()).filter(Filter::Effect(effect)),
    )
}
