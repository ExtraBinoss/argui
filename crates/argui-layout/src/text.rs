use std::borrow::Cow;

use argui_core::Rect;
use argui_ui::{Element, ElementKind, NodeId, UiTree};

use crate::engine::NodeMap;

pub(super) fn content<'a>(
    ui: &'a UiTree,
    node: NodeId,
    element: &'a Element,
) -> Option<(Cow<'a, str>, Cow<'a, argui_text::TextStyle>)> {
    match &element.kind {
        ElementKind::Text { content, style } => Some((
            Cow::Borrowed(content),
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
                    Cow::Borrowed(placeholder),
                    Cow::Owned(ui.resolved_text_style(node, placeholder_text)),
                ))
            } else {
                Some((
                    Cow::Owned(value),
                    Cow::Owned(ui.resolved_text_style(node, text)),
                ))
            }
        }
        ElementKind::Container | ElementKind::Image { .. } | ElementKind::Vector { .. } => None,
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
