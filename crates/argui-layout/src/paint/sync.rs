use argui_ui::{Element, ElementKind, UiTree};

use crate::LayoutOutput;

pub(super) fn scroll_config(elements: &[&Element], ui: &UiTree, output: &mut LayoutOutput) {
    for region in &mut output.scroll_regions {
        if let Some(node) = output.nodes.iter().find(|node| node.node == region.node) {
            let authored = elements[node.index]
                .scroll
                .clone()
                .unwrap_or_else(|| region.config.clone());
            let config = ui.resolved_scroll_config(region.node, &authored);
            if let (Some(scrollbar), Some(style)) = (&mut region.scrollbar, &config.scrollbar) {
                scrollbar.style = style.clone();
            }
            region.config = config;
        }
    }
}

pub(super) fn text_colors(elements: &[&Element], ui: &UiTree, output: &mut LayoutOutput) {
    for node in &output.nodes {
        let Some(text_index) = node.text_index else {
            continue;
        };
        let color = match &elements[node.index].kind {
            ElementKind::Text { style, .. } => style.color,
            ElementKind::TextEditor {
                text,
                placeholder_text,
                ..
            } => {
                if ui
                    .text_input_display(node.node)
                    .is_some_and(|value| value.is_empty())
                {
                    if let Some(block) = output.text.blocks_mut().get_mut(text_index) {
                        block.style.color = placeholder_text.color;
                    }
                    continue;
                } else {
                    text.color
                }
            }
            ElementKind::Container | ElementKind::Image { .. } | ElementKind::Vector { .. } => {
                continue;
            }
        };
        if let Some(block) = output.text.blocks_mut().get_mut(text_index) {
            block.style.color = ui.resolved_text_color(node.node, color);
        }
    }
}
