use std::collections::HashMap;

use argui_ui::{Element, ElementKind, NodeId as UiNodeId, UiTree};
use taffy::{NodeId, TaffyTree};

use crate::{LayoutError, engine::NodeMap, engine::taffy_style};

struct RetainedNode {
    index: usize,
    id: NodeId,
    kind: ElementKind,
    style: argui_ui::LayoutStyle,
    children: Vec<NodeId>,
}

pub(crate) fn sync(
    tree: &mut TaffyTree<usize>,
    root: NodeMap,
    ui: &UiTree,
) -> Result<NodeMap, LayoutError> {
    let mut retained = HashMap::new();
    retain(root, &mut retained);
    let mut index = 0;
    let root = reconcile_node(tree, &mut retained, ui, ui.root(), &mut index)?;
    for node in retained.into_values() {
        tree.remove(node.id)?;
    }
    Ok(root)
}

fn retain(node: NodeMap, retained: &mut HashMap<UiNodeId, RetainedNode>) {
    let children = node.children.iter().map(|child| child.id).collect();
    let entry = RetainedNode {
        index: node.index,
        id: node.id,
        kind: node.kind,
        style: node.style,
        children,
    };
    for child in node.children {
        retain(child, retained);
    }
    retained.insert(node.node, entry);
}

fn reconcile_node(
    tree: &mut TaffyTree<usize>,
    retained: &mut HashMap<UiNodeId, RetainedNode>,
    ui: &UiTree,
    element: &Element,
    cursor: &mut usize,
) -> Result<NodeMap, LayoutError> {
    let index = *cursor;
    let node = ui
        .node_id_at(index)
        .ok_or(LayoutError::MissingNodeIdentity(index))?;
    *cursor += 1;
    let previous = retained.remove(&node);
    let id = match &previous {
        Some(previous) => reuse_node(tree, previous, element, index)?,
        None => create_node(tree, element, index)?,
    };
    let children = element
        .children
        .iter()
        .map(|child| reconcile_node(tree, retained, ui, child, cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let child_ids = children.iter().map(|child| child.id).collect::<Vec<_>>();
    if previous
        .as_ref()
        .is_none_or(|previous| previous.children != child_ids)
    {
        tree.set_children(id, &child_ids)?;
    }
    Ok(NodeMap {
        index,
        node,
        id,
        kind: element.kind.clone(),
        style: element.style.clone(),
        children,
    })
}

fn reuse_node(
    tree: &mut TaffyTree<usize>,
    previous: &RetainedNode,
    element: &Element,
    index: usize,
) -> Result<NodeId, LayoutError> {
    if previous.style != element.style {
        tree.set_style(previous.id, taffy_style(&element.style))?;
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
    index: usize,
) -> Result<NodeId, LayoutError> {
    let style = taffy_style(&element.style);
    match element.kind {
        ElementKind::Container => tree.new_leaf(style).map_err(Into::into),
        _ => tree.new_leaf_with_context(style, index).map_err(Into::into),
    }
}

fn intrinsic_measure_changed(old: &ElementKind, new: &ElementKind) -> bool {
    old != new
        && matches!(
            (old, new),
            (ElementKind::Text { .. }, ElementKind::Text { .. })
                | (ElementKind::TextInput { .. }, ElementKind::TextInput { .. })
        )
}
