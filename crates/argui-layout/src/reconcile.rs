use crate::layout_tree::LayoutTree;
use std::collections::HashMap;

use argui_ui::{Element, ElementKind, UiTree};
use taffy::NodeId;

use crate::{LayoutError, assets::AssetMetrics, engine::NodeMap, style::taffy_style};

pub(crate) fn sync(
    tree: &mut LayoutTree,
    root: NodeMap,
    assets: &AssetMetrics,
    ui: &UiTree,
) -> Result<NodeMap, LayoutError> {
    let mut cursor = 0;
    reconcile_node(tree, Some(root), assets, ui, ui.root(), &mut cursor)
}

fn reconcile_node(
    tree: &mut LayoutTree,
    previous: Option<NodeMap>,
    assets: &AssetMetrics,
    ui: &UiTree,
    element: &Element,
    cursor: &mut usize,
) -> Result<NodeMap, LayoutError> {
    let index = *cursor;
    let node = ui
        .node_id_at(index)
        .ok_or(LayoutError::MissingNodeIdentity(index))?;

    if previous.as_ref().is_some_and(|previous| {
        previous.node == node && previous.index == index && previous.element.ptr_eq(element)
    }) {
        let previous = previous.expect("checked above");
        *cursor += previous.subtree_len;
        return Ok(previous);
    }

    *cursor += 1;
    let previous = match previous {
        Some(previous) if previous.node != node => {
            remove_subtree(tree, previous)?;
            None
        }
        previous => previous,
    };
    let custom_state = previous
        .as_ref()
        .and_then(|node| node.custom_state.clone())
        .or_else(|| {
            if let ElementKind::Custom(custom) = &element.kind {
                Some(std::rc::Rc::new(custom.create_state()))
            } else {
                None
            }
        });
    let resolved_style =
        assets.layout_style(ui.resolved_layout_style(node, element), &element.kind);
    let (id, previous_children) = match previous {
        Some(previous) => {
            let id = reuse_node(tree, &previous, element, &resolved_style, index)?;
            (id, previous.children)
        }
        None => (
            create_node(tree, element, &resolved_style, index)?,
            Vec::new(),
        ),
    };
    let previous_ids = crate::overlay::layout_children(&previous_children);
    let mut retained = previous_children
        .into_iter()
        .map(|child| (child.node, child))
        .collect::<HashMap<_, _>>();
    let mut children = Vec::with_capacity(element.children.len());
    for child in &element.children {
        let child_node = ui
            .node_id_at(*cursor)
            .ok_or(LayoutError::MissingNodeIdentity(*cursor))?;
        children.push(reconcile_node(
            tree,
            retained.remove(&child_node),
            assets,
            ui,
            child,
            cursor,
        )?);
    }
    for removed in retained.into_values() {
        remove_subtree(tree, removed)?;
    }
    let child_ids = crate::overlay::layout_children(&children);
    if previous_ids != child_ids {
        tree.set_children(id, &child_ids)?;
    }
    let subtree_len = 1 + children
        .iter()
        .map(|child| child.subtree_len)
        .sum::<usize>();
    Ok(NodeMap {
        custom_state: crate::custom::install(tree, id, &element.kind, custom_state)?,
        index,
        node,
        id,
        kind: element.kind.clone(),
        style: resolved_style,
        element: element.clone(),
        subtree_len,
        children,
    })
}

fn reuse_node(
    tree: &mut LayoutTree,
    previous: &NodeMap,
    element: &Element,
    style: &argui_ui::LayoutStyle,
    index: usize,
) -> Result<NodeId, LayoutError> {
    if previous.style != *style {
        tree.set_style(previous.id, taffy_style(style))?;
    }
    if previous.index != index && !matches!(element.kind, ElementKind::Container) {
        tree.set_node_context(previous.id, Some(index))?;
    }
    if intrinsic_measure_changed(&previous.kind, &element.kind) {
        tree.mark_dirty(previous.id)?;
    }
    Ok(previous.id)
}

fn create_node(
    tree: &mut LayoutTree,
    element: &Element,
    style: &argui_ui::LayoutStyle,
    index: usize,
) -> Result<NodeId, LayoutError> {
    let style = taffy_style(style);
    match element.kind {
        ElementKind::Container => tree.new_leaf(style).map_err(Into::into),
        _ => tree.new_leaf_with_context(style, index).map_err(Into::into),
    }
}

fn remove_subtree(tree: &mut LayoutTree, node: NodeMap) -> Result<(), LayoutError> {
    for child in node.children {
        remove_subtree(tree, child)?;
    }
    tree.remove(node.id)?;
    Ok(())
}

fn intrinsic_measure_changed(old: &ElementKind, new: &ElementKind) -> bool {
    if let (ElementKind::Custom(old), ElementKind::Custom(new)) = (old, new) {
        return !old.same_layout(new);
    }
    old != new
        && matches!(
            (old, new),
            (ElementKind::Text { .. }, ElementKind::Text { .. })
                | (
                    ElementKind::TextEditor { .. },
                    ElementKind::TextEditor { .. }
                )
        )
}
