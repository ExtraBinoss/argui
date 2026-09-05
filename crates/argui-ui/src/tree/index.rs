use std::collections::HashMap;

use crate::{Element, NodeId, TextSelectionStyle, UserSelect};

/// One preorder index per retained tree revision, shared by layout and input queries.
#[derive(Clone, Debug)]
pub(crate) struct TreeIndex {
    elements: Vec<Element>,
    positions: HashMap<NodeId, usize>,
    selection: Vec<(UserSelect, TextSelectionStyle)>,
}

impl TreeIndex {
    pub(super) fn new(root: &Element, ids: &[NodeId]) -> Self {
        let mut index = Self {
            elements: Vec::with_capacity(ids.len()),
            positions: HashMap::with_capacity(ids.len()),
            selection: Vec::with_capacity(ids.len()),
        };
        index.visit(root, ids, UserSelect::Text, TextSelectionStyle::default());
        index
    }

    fn visit(
        &mut self,
        element: &Element,
        ids: &[NodeId],
        parent: UserSelect,
        style: TextSelectionStyle,
    ) {
        let position = self.elements.len();
        let policy = match element.user_select {
            UserSelect::Auto => parent,
            explicit => explicit,
        };
        let style = element.selection_style.unwrap_or(style);
        self.positions.insert(ids[position], position);
        self.elements.push(element.clone());
        self.selection.push((policy, style));
        for child in &element.children {
            self.visit(child, ids, policy, style);
        }
    }

    pub(super) fn at(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }

    pub(super) fn element(&self, node: NodeId) -> Option<&Element> {
        self.at(*self.positions.get(&node)?)
    }

    pub(crate) fn selection(&self, node: NodeId) -> Option<(UserSelect, TextSelectionStyle)> {
        self.selection.get(*self.positions.get(&node)?).copied()
    }
}
