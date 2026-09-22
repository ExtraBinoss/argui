use std::borrow::Cow;

use argui_core::{Rect, Size};
use argui_text::TextContent;
use argui_ui::{Element, ElementKind, NodeId, ScrollbarGutter, UiTree};

use crate::engine::NodeMap;

pub(super) fn content<'a>(
    ui: &'a UiTree,
    node: NodeId,
    element: &'a Element,
) -> Option<(TextContent, Cow<'a, argui_text::TextStyle>)> {
    match &element.kind {
        ElementKind::Text { content, style } => Some((
            content.clone(),
            Cow::Owned(ui.resolved_text_style(node, style)),
        )),
        ElementKind::TextEditor {
            placeholder,
            styled,
            text,
            placeholder_text,
            ..
        } => {
            let value = ui.text_input_display(node)?;
            if value.is_empty() {
                Some((
                    TextContent::plain(placeholder.clone()),
                    Cow::Borrowed(placeholder_text),
                ))
            } else {
                let content = styled
                    .as_deref()
                    .filter(|content| content.as_str() == value.as_ref())
                    .cloned()
                    .unwrap_or_else(|| TextContent::plain(value.into_owned()));
                Some((content, Cow::Owned(ui.resolved_text_style(node, text))))
            }
        }
        ElementKind::Custom(_)
        | ElementKind::GpuCanvas(_)
        | ElementKind::Container
        | ElementKind::Image { .. }
        | ElementKind::Vector { .. } => None,
    }
}

/// Clips editor paint before reserved scrollbar gutters without narrowing other text.
///
/// `node` supplies the resolved gutter and overflow, `element` identifies an editor,
/// `parent` is the inherited clip, and `bounds` is the element's laid-out rectangle.
/// Returns the intersection with the editor's visible area, if any.
pub(super) fn clip(
    node: &NodeMap,
    element: &Element,
    parent: Option<Rect>,
    bounds: Rect,
) -> Option<Rect> {
    if matches!(element.kind, ElementKind::TextEditor { .. }) {
        let gutter = if node.style.scrollbar_gutter == ScrollbarGutter::Stable {
            node.style.scrollbar_width.max(0.0)
        } else {
            0.0
        };
        let visible = Rect::new(
            bounds.origin,
            Size::new(
                (bounds.size.width
                    - if node.style.overflow.y.scrolls() {
                        gutter
                    } else {
                        0.0
                    })
                .max(0.0),
                (bounds.size.height
                    - if node.style.overflow.x.scrolls() {
                        gutter
                    } else {
                        0.0
                    })
                .max(0.0),
            ),
        );
        parent.and_then(|clip| clip.intersection(visible))
    } else {
        crate::scroll::clipped(node, parent, bounds)
    }
}
