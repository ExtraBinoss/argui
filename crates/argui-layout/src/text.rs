use std::borrow::Cow;

use argui_ui::{Element, ElementKind, NodeId, UiTree};

pub(super) fn content<'a>(
    ui: &'a UiTree,
    node: NodeId,
    element: &'a Element,
) -> Option<(Cow<'a, str>, &'a argui_text::TextStyle)> {
    match &element.kind {
        ElementKind::Text { content, style } => Some((Cow::Borrowed(content), style)),
        ElementKind::TextEditor {
            placeholder,
            text,
            placeholder_text,
            ..
        } => {
            let value = ui.text_input_display(node)?;
            if value.is_empty() {
                Some((Cow::Borrowed(placeholder), placeholder_text))
            } else {
                Some((Cow::Owned(value), text))
            }
        }
        ElementKind::Container | ElementKind::Image { .. } | ElementKind::Vector { .. } => None,
    }
}
