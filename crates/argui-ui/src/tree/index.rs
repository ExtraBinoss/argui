use std::collections::HashMap;

use crate::{Element, NodeId, TextSelectionStyle, UserSelect, WritingDirection};

const NONE: u32 = u32::MAX;

/// Dense preorder columns shared by layout and input queries. Stable NodeIds
/// remain independent from these positions, which can change at every revision.
#[derive(Clone, Debug)]
pub(crate) struct TreeIndex {
    elements: Vec<Element>,
    positions: HashMap<NodeId, usize>,
    parents: Vec<u32>,
    selection: Vec<UserSelect>,
    selection_owners: Vec<u32>,
    directions: Vec<Option<WritingDirection>>,
    default_selection_style: TextSelectionStyle,
}

impl TreeIndex {
    pub(super) fn new(root: &Element, ids: &[NodeId]) -> Self {
        assert!(
            ids.len() < NONE as usize,
            "UI tree exceeds compact index capacity"
        );
        let mut index = Self {
            elements: Vec::with_capacity(ids.len()),
            positions: HashMap::with_capacity(ids.len()),
            parents: Vec::with_capacity(ids.len()),
            selection: Vec::with_capacity(ids.len()),
            selection_owners: Vec::with_capacity(ids.len()),
            directions: Vec::with_capacity(ids.len()),
            default_selection_style: TextSelectionStyle::default(),
        };
        index.visit(root, ids, NONE);
        index
    }

    fn visit(&mut self, element: &Element, ids: &[NodeId], parent: u32) {
        let position = self.elements.len();
        let inherited = (parent != NONE).then_some(parent as usize);
        let policy = match element.user_select {
            UserSelect::Auto => inherited.map_or(UserSelect::Text, |p| self.selection[p]),
            explicit => explicit,
        };
        let owner = if element.selection_style.is_some() {
            position as u32
        } else {
            inherited.map_or(NONE, |p| self.selection_owners[p])
        };
        let direction = element
            .direction_scope
            .or_else(|| inherited.and_then(|p| self.directions[p]));
        self.positions.insert(ids[position], position);
        self.elements.push(element.clone());
        self.parents.push(parent);
        self.selection.push(policy);
        self.selection_owners.push(owner);
        self.directions.push(direction);
        for child in &element.children {
            self.visit(child, ids, position as u32);
        }
    }

    pub(super) fn position(&self, node: NodeId) -> Option<usize> {
        self.positions.get(&node).copied()
    }

    pub(super) fn parent(&self, index: usize) -> Option<usize> {
        let parent = *self.parents.get(index)?;
        (parent != NONE).then_some(parent as usize)
    }

    pub(super) fn at(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }

    pub(super) fn element(&self, node: NodeId) -> Option<&Element> {
        self.at(self.position(node)?)
    }

    pub(super) fn direction(&self, node: NodeId) -> Option<WritingDirection> {
        self.directions[self.position(node)?]
    }

    pub(crate) fn user_select(&self, node: NodeId) -> UserSelect {
        self.position(node)
            .map_or(UserSelect::None, |index| self.selection[index])
    }

    pub(crate) fn selection_style(&self, node: NodeId) -> TextSelectionStyle {
        let Some(index) = self.position(node) else {
            return self.default_selection_style;
        };
        let owner = self.selection_owners[index];
        self.elements
            .get(owner as usize)
            .and_then(|element| element.selection_style)
            .unwrap_or(self.default_selection_style)
    }
}
