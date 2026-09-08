use std::borrow::Cow;

use argui_core::Rect;
use argui_text::TextContent;
use argui_ui::{Element, ElementKind, NodeId, UiTree};

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
                Some((
                    TextContent::plain(value),
                    Cow::Owned(ui.resolved_text_style(node, text)),
                ))
            }
        }
        ElementKind::Custom(_)
        | ElementKind::Container
        | ElementKind::Image { .. }
        | ElementKind::Vector { .. } => None,
    }
}

pub(super) fn clip(
    node: &NodeMap,
    element: &Element,
    parent: Option<Rect>,
    bounds: Rect,
) -> Option<Rect> {
    if matches!(element.kind, ElementKind::TextEditor { .. }) {
        parent.and_then(|clip| clip.intersection(bounds))
    } else {
        crate::scroll::clipped(node, parent, bounds)
    }
}
