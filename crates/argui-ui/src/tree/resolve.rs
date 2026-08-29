use argui_paint::{LayerStyle, QuadStyle};

use crate::{Element, LayoutStyle, NodeId};

use super::UiTree;

impl UiTree {
    #[must_use]
    pub fn resolved_quad(&self, node: NodeId, element: &Element) -> QuadStyle {
        let base = element.paint.quad.clone();
        let resolved = element
            .interaction
            .as_ref()
            .map_or(base.clone(), |interaction| {
                interaction.resolve(base, self.visual_state(node))
            });
        crate::binding::resolved_quad(&element.bindings, resolved)
    }

    #[must_use]
    pub fn resolved_transform(&self, _node: NodeId, element: &Element) -> argui_core::Transform2D {
        crate::binding::resolved_transform(&element.bindings, element.transform)
    }

    #[must_use]
    pub fn resolved_layout_style(&self, element: &Element) -> LayoutStyle {
        crate::binding::resolved_layout(&element.bindings, &element.style)
    }

    #[must_use]
    pub fn resolved_layer(&self, element: &Element, layer: &LayerStyle) -> LayerStyle {
        crate::binding::resolved_layer(&element.bindings, layer)
    }
}
