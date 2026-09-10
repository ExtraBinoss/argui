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
        let mut semantic = SemanticTree { root, focus, nodes };
        crate::semantic_relations::resolve(self, &mut semantic);
        semantic
    }
}

#[allow(clippy::too_many_arguments)]
fn collect(
    tree: &UiTree,
    element: &Element,
    bounds: &HashMap<NodeId, Rect>,
    scale: f32,
    index: &mut usize,
    semantic_parent: Option<usize>,
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
    let semantics = resolved_semantics(tree, node_id, element, is_root).map(|mut semantics| {
        if let Some(interaction) = &element.interaction {
            semantics.focus_policy = if interaction.enabled {
                interaction.focus_policy
            } else {
                crate::FocusPolicy::None
            };
            if semantics.focus_policy.is_focusable()
                && !semantics.actions.contains(&SemanticAction::Focus)
            {
                semantics.actions.push(SemanticAction::Focus);
            }
        }
        semantics
    });
    let current = semantics.as_ref().map(|_| output.len());
    if let Some(semantics) = semantics {
        let id = SemanticNodeId::new(node_id.get());
        output.push(SemanticNode {
            id,
            bounds: scaled(bounds.get(&node_id).copied().unwrap_or_default(), scale),
            semantics,
            children: Vec::new(),
        });
        if let Some(parent) = semantic_parent {
            output[parent].children.push(id);
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
            semantics.value =
                (!element.text_privacy.protected()).then(|| SemanticValue::Text(value.to_owned()));
            semantics.state.protected = element.text_privacy.protected();
        }
        semantics.state.disabled |= element
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
        } => Some({
            let mut semantics = Semantics::new(if *multiline {
                Role::TextArea
            } else {
                Role::TextInput
            })
            .label(placeholder)
            .value(SemanticValue::Text(if element.text_privacy.protected() {
                String::new()
            } else {
                value.clone()
            }))
            .action(SemanticAction::Focus)
            .action(SemanticAction::SetValue);
            if element.text_privacy.protected() {
                semantics.value = None;
                semantics.state.protected = true;
            }
            semantics
        }),
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
