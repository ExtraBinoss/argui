use argui_paint::{LayerStyle, QuadStyle};

use crate::{Element, LayoutStyle, NodeId, ScrollConfig};

use super::UiTree;

impl UiTree {
    #[must_use]
    pub fn resolved_text_style(
        &self,
        node: NodeId,
        style: &argui_text::TextStyle,
    ) -> argui_text::TextStyle {
        let mut style = style.clone();
        style.color = self.resolved_text_color(node, style.color);
        style
    }

    #[must_use]
    pub fn resolved_text_color(
        &self,
        node: NodeId,
        mut color: argui_core::Color,
    ) -> argui_core::Color {
        super::transition::apply_text_color(&self.transitions, node, &mut color);
        color
    }

    #[must_use]
    pub fn resolved_vector_color(
        &self,
        node: NodeId,
        mut color: argui_core::Color,
    ) -> argui_core::Color {
        super::transition::apply_vector_color(&self.transitions, node, &mut color);
        color
    }

    #[must_use]
    pub fn resolved_quad(&self, node: NodeId, element: &Element) -> QuadStyle {
        let mut resolved = element.paint.quad.clone();
        super::transition::apply_quad(&self.transitions, node, &mut resolved);
        crate::binding::resolved_quad(&element.bindings, resolved)
    }

    #[must_use]
    pub fn resolved_transform(&self, node: NodeId, element: &Element) -> argui_core::Transform2D {
        let mut transform = element.transform;
        super::transition::apply_transform(&self.transitions, node, &mut transform);
        crate::binding::resolved_transform(&element.bindings, transform)
    }

    #[must_use]
    pub fn resolved_layout_style(&self, node: NodeId, element: &Element) -> LayoutStyle {
        let mut style = element.style.clone();
        super::transition::apply_layout(&self.transitions, node, &mut style);
        let mut style = crate::binding::resolved_layout(&element.bindings, &style);
        if let Some(border) = self.resolved_quad(node, element).border {
            style.border = crate::Sides {
                left: crate::LengthPercentage::length(border.widths.left),
                right: crate::LengthPercentage::length(border.widths.right),
                top: crate::LengthPercentage::length(border.widths.top),
                bottom: crate::LengthPercentage::length(border.widths.bottom),
            };
        }
        style
    }

    #[must_use]
    pub fn resolved_scroll_config(&self, node: NodeId, config: &ScrollConfig) -> ScrollConfig {
        let mut resolved = config.clone();
        if let Some(scrollbar) = &mut resolved.scrollbar {
            super::transition::apply_scrollbar_part(
                &self.transitions,
                node,
                crate::scroll::ScrollbarPart::Track,
                &mut scrollbar.track.base,
            );
            super::transition::apply_scrollbar_part(
                &self.transitions,
                node,
                crate::scroll::ScrollbarPart::Thumb,
                &mut scrollbar.thumb.base,
            );
        }
        resolved
    }

    #[must_use]
    pub fn resolved_layer(
        &self,
        node: NodeId,
        element: &Element,
        layer: &LayerStyle,
    ) -> LayerStyle {
        let mut layer = layer.clone();
        super::transition::apply_layer(&self.transitions, node, &mut layer);
        crate::binding::resolved_layer(&element.bindings, &layer)
    }
}
