use argui_paint::{LayerStyle, QuadStyle};

use crate::{Element, LayoutStyle, NodeId, ScrollConfig};

use super::UiTree;

impl UiTree {
    /// Resolves a text style by applying current tree color state to `style`.
    ///
    /// * `node` — node whose style state is applied.
    /// * `style` — authored text style.
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

    /// Resolves a text color through active transitions.
    ///
    /// * `node` — node whose transitions are applied.
    /// * `color` — authored color.
    #[must_use]
    pub fn resolved_text_color(
        &self,
        node: NodeId,
        mut color: argui_core::Color,
    ) -> argui_core::Color {
        super::transition::apply_text_color(&self.transitions, node, &mut color);
        color
    }

    /// Resolves a vector color through active transitions.
    ///
    /// * `node` — node whose transitions are applied.
    /// * `color` — authored color.
    #[must_use]
    pub fn resolved_vector_color(
        &self,
        node: NodeId,
        mut color: argui_core::Color,
    ) -> argui_core::Color {
        super::transition::apply_vector_color(&self.transitions, node, &mut color);
        color
    }

    /// Resolves an element's quad paint style, transitions, and property bindings.
    ///
    /// * `node` — retained node identifier.
    /// * `element` — element whose authored paint style is resolved.
    #[must_use]
    pub fn resolved_quad(&self, node: NodeId, element: &Element) -> QuadStyle {
        let mut resolved = element.paint.quad.clone();
        super::transition::apply_quad(&self.transitions, node, &mut resolved);
        if let Some(backdrop) = element.desktop_backdrop {
            // Detached popup surfaces currently paint opaquely; their material must fall back
            // independently of the main window's compositor capability.
            let mut state = self.desktop_backdrop_state;
            state.available &= self.native_portal_owner(node).is_none();
            resolved.background = Some(argui_paint::Fill::Solid(backdrop.color(state)));
        }
        crate::binding::resolved_quad(&element.bindings, resolved)
    }

    /// Resolves an element transform using active transitions and bindings.
    ///
    /// * `node` — retained node identifier.
    /// * `element` — element whose authored transform is resolved.
    #[must_use]
    pub fn resolved_transform(&self, node: NodeId, element: &Element) -> argui_core::Transform2D {
        let mut transform = element.transform;
        super::transition::apply_transform(&self.transitions, node, &mut transform);
        crate::binding::resolved_transform(&element.bindings, transform)
    }

    /// Resolves an element layout style using transitions, bindings, direction, and borders.
    ///
    /// * `node` — retained node identifier.
    /// * `element` — element whose authored layout style is resolved.
    #[must_use]
    pub fn resolved_layout_style(&self, node: NodeId, element: &Element) -> LayoutStyle {
        let mut style = element.style.clone();
        super::transition::apply_layout(&self.transitions, node, &mut style);
        let mut style = crate::binding::resolved_layout(&element.bindings, style);
        if let Some(direction) = self.index.direction(node) {
            style.writing_direction = direction;
        }
        if let Some(insets) = style.logical_padding {
            let (left, right) = insets.horizontal(style.writing_direction);
            if let Some(left) = left {
                style.padding.left = crate::LengthPercentage::length(left);
            }
            if let Some(right) = right {
                style.padding.right = crate::LengthPercentage::length(right);
            }
        }
        if let Some(insets) = style.logical_margin {
            let (left, right) = insets.horizontal(style.writing_direction);
            if let Some(left) = left {
                style.margin.left = crate::LengthPercentageAuto::length(left);
            }
            if let Some(right) = right {
                style.margin.right = crate::LengthPercentageAuto::length(right);
            }
        }
        if let Some(insets) = style.logical_inset {
            let (left, right) = insets.horizontal(style.writing_direction);
            if let Some(left) = left {
                style.inset.left = crate::LengthPercentageAuto::length(left);
            }
            if let Some(right) = right {
                style.inset.right = crate::LengthPercentageAuto::length(right);
            }
        }
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

    /// Resolves a scroll configuration's animated scrollbar styles.
    ///
    /// * `node` — retained scroll-container node.
    /// * `config` — authored scroll configuration.
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
            let opacity = self.scroll.scrollbar_opacity(node, scrollbar.visibility);
            scrollbar.track.base.opacity *= opacity;
            scrollbar.thumb.base.opacity *= opacity;
        }
        resolved
    }

    /// Resolves a layer style using active transitions and element bindings.
    ///
    /// * `node` — retained node identifier.
    /// * `element` — element providing property bindings.
    /// * `layer` — authored layer style.
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
