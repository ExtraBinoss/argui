use std::collections::HashMap;

use argui_accessibility::{
    Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticTree, SemanticValue, Semantics,
};
use argui_core::{Point, Rect, Size};

use crate::{Display, Element, ElementKind, NodeId, UiTree};

impl UiTree {
    /// Builds the renderer-independent accessibility snapshot from retained
    /// identities and the latest resolved layout bounds.
    #[must_use]
    pub fn semantic_tree(&self, bounds: &[(NodeId, Rect)], scale_factor: f32) -> SemanticTree {
        let bounds = bounds.iter().copied().collect::<HashMap<_, _>>();
        let root = SemanticNodeId::new(self.node_ids()[0].get());
        let mut nodes = Vec::new();
        let mut index = 0;
        collect(
            self,
            self.root(),
            &bounds,
            scale_factor,
            &mut index,
            None,
            true,
            &mut nodes,
        );
        let focused = self
            .focused_node()
            .map(|node| SemanticNodeId::new(node.get()));
        let focus = focused
            .filter(|focused| nodes.iter().any(|node| node.id == *focused))
            .unwrap_or(root);
        SemanticTree { root, focus, nodes }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect(
    tree: &UiTree,
    element: &Element,
    bounds: &HashMap<NodeId, Rect>,
    scale: f32,
    index: &mut usize,
    semantic_parent: Option<SemanticNodeId>,
    is_root: bool,
    output: &mut Vec<SemanticNode>,
) {
    let node_id = tree.node_ids()[*index];
    *index += 1;
    if element.semantic_hidden
        || tree.resolved_layout_style(node_id, element).display == Display::None
        || !tree.semantic_focus_visible(node_id)
    {
        *index += descendant_count(element);
        return;
    }
    let semantics = resolved_semantics(tree, node_id, element, is_root);
    let current = semantics
        .as_ref()
        .map(|_| SemanticNodeId::new(node_id.get()));
    if let Some(semantics) = semantics {
        let id = SemanticNodeId::new(node_id.get());
        output.push(SemanticNode {
            id,
            bounds: scaled(bounds.get(&node_id).copied().unwrap_or_default(), scale),
            semantics,
            children: Vec::new(),
        });
        if let Some(parent) = semantic_parent
            && let Some(parent) = output.iter_mut().find(|node| node.id == parent)
        {
            parent.children.push(id);
        }
    }
    let parent = current.or(semantic_parent);
    let suppress_children = current.is_some_and(|_| composite_leaf(element));
    for child in &element.children {
        if suppress_children {
            *index += 1 + descendant_count(child);
        } else {
            collect(tree, child, bounds, scale, index, parent, false, output);
        }
    }
}

fn resolved_semantics(
    tree: &UiTree,
    node: NodeId,
    element: &Element,
    is_root: bool,
) -> Option<Semantics> {
    if let Some(mut semantics) = element.semantics.clone() {
        if matches!(element.kind, ElementKind::TextEditor { .. })
            && let Some(value) = tree.text_input_value(node)
        {
            semantics.value = Some(SemanticValue::Text(value.to_owned()));
        }
        semantics.state.disabled = element
            .interaction
            .as_ref()
            .is_some_and(|interaction| !interaction.enabled);
        semantics.state.modal = tree.active_modal_scope() == Some(node);
        return Some(semantics);
    }
    if tree.active_modal_scope() == Some(node) {
        let mut semantics = Semantics::new(Role::Dialog);
        semantics.state.modal = true;
        return Some(semantics);
    }
    match &element.kind {
        ElementKind::Text { content, .. } => {
            Some(Semantics::new(Role::Text).label(content.as_str()))
        }
        ElementKind::TextEditor {
            placeholder,
            value,
            multiline,
            ..
        } => Some(
            Semantics::new(if *multiline {
                Role::TextArea
            } else {
                Role::TextInput
            })
            .label(placeholder)
            .value(SemanticValue::Text(value.clone()))
            .action(SemanticAction::Focus)
            .action(SemanticAction::SetValue),
        ),
        _ if is_root => Some(Semantics::new(Role::Window)),
        _ => None,
    }
}

fn composite_leaf(element: &Element) -> bool {
    element.semantics.as_ref().is_some_and(|semantics| {
        matches!(
            semantics.role,
            Role::Button
                | Role::CheckBox
                | Role::RadioButton
                | Role::Switch
                | Role::TextInput
                | Role::SearchInput
                | Role::Slider
        )
    })
}

fn descendant_count(element: &Element) -> usize {
    element
        .children
        .iter()
        .map(|child| 1 + descendant_count(child))
        .sum()
}

fn scaled(rect: Rect, scale: f32) -> Rect {
    Rect::new(
        Point::new(rect.origin.x * scale, rect.origin.y * scale),
        Size::new(rect.size.width * scale, rect.size.height * scale),
    )
}
