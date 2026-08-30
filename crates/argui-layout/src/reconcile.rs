use std::collections::HashMap;

use argui_ui::{Element, ElementKind, UiTree};
use taffy::{NodeId, TaffyTree};

use crate::{LayoutError, assets::AssetMetrics, engine::NodeMap, style::taffy_style};

pub(crate) fn sync(
    tree: &mut TaffyTree<usize>,
    root: NodeMap,
    assets: &AssetMetrics,
    ui: &UiTree,
) -> Result<NodeMap, LayoutError> {
    let mut cursor = 0;
    reconcile_node(tree, Some(root), assets, ui, ui.root(), &mut cursor)
}

fn reconcile_node(
    tree: &mut TaffyTree<usize>,
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
    let previous = previous.filter(|previous| previous.node == node);
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
    let previous_ids = previous_children
        .iter()
        .map(|child| child.id)
        .collect::<Vec<_>>();
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
    let child_ids = children.iter().map(|child| child.id).collect::<Vec<_>>();
    if previous_ids != child_ids {
        tree.set_children(id, &child_ids)?;
    }
    let subtree_len = 1 + children
        .iter()
        .map(|child| child.subtree_len)
        .sum::<usize>();
    Ok(NodeMap {
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
    tree: &mut TaffyTree<usize>,
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
    tree: &mut TaffyTree<usize>,
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

fn remove_subtree(tree: &mut TaffyTree<usize>, node: NodeMap) -> Result<(), LayoutError> {
    for child in node.children {
        remove_subtree(tree, child)?;
    }
    tree.remove(node.id)?;
    Ok(())
}

fn intrinsic_measure_changed(old: &ElementKind, new: &ElementKind) -> bool {
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
