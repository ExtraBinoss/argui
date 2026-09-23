use std::collections::HashMap;

use crate::{
    Element, NodeId, RetainedIdentity, TextSelectionHighlight, TextSelectionStyle, UserSelect,
    WritingDirection,
};

const NONE: u32 = u32::MAX;

/// Dense preorder columns shared by layout and input queries. Stable NodeIds
/// remain independent from these positions, which can change at every revision.
#[derive(Clone, Debug)]
pub(crate) struct TreeIndex {
    elements: Vec<Element>,
    positions: HashMap<NodeId, usize>,
    identities: HashMap<RetainedIdentity, Option<NodeId>>,
    parents: Vec<u32>,
    subtree_ends: Vec<u32>,
    selection: Vec<UserSelect>,
    selection_owners: Vec<u32>,
    selection_highlight_owners: Vec<u32>,
    directions: Vec<Option<WritingDirection>>,
    default_selection_style: TextSelectionStyle,
    transition_nodes: usize,
    layout_roots: Vec<usize>,
}

impl TreeIndex {
    pub(super) fn column_bytes(&self) -> usize {
        self.elements.capacity() * size_of::<Element>()
            + (self.parents.capacity()
                + self.subtree_ends.capacity()
                + self.selection_owners.capacity()
                + self.selection_highlight_owners.capacity())
                * size_of::<u32>()
            + self.selection.capacity() * size_of::<UserSelect>()
            + self.directions.capacity() * size_of::<Option<WritingDirection>>()
            + self.layout_roots.capacity() * size_of::<usize>()
    }
    pub(super) fn new(root: &Element, ids: &[NodeId]) -> Self {
        assert!(
            ids.len() < NONE as usize,
            "UI tree exceeds compact index capacity"
        );
        let mut index = Self {
            elements: Vec::with_capacity(ids.len()),
            positions: HashMap::with_capacity(ids.len()),
            identities: HashMap::new(),
            parents: Vec::with_capacity(ids.len()),
            subtree_ends: Vec::with_capacity(ids.len()),
            selection: Vec::with_capacity(ids.len()),
            selection_owners: Vec::with_capacity(ids.len()),
            selection_highlight_owners: Vec::with_capacity(ids.len()),
            directions: Vec::with_capacity(ids.len()),
            default_selection_style: TextSelectionStyle::default(),
            transition_nodes: 0,
            layout_roots: Vec::new(),
        };
        index.visit(root, ids, NONE);
        index.sync_identities();
        index
    }

    /// Reuses dense column allocations when child order or tree shape changes.
    ///
    /// * `root` — replacement element hierarchy in preorder.
    /// * `ids` — stable node identities in the same preorder.
    ///
    /// # Panics
    ///
    /// Panics when the tree cannot fit in the compact index representation.
    pub(super) fn rebuild(&mut self, root: &Element, ids: &[NodeId]) {
        assert!(
            ids.len() < NONE as usize,
            "UI tree exceeds compact index capacity"
        );
        self.elements.clear();
        self.positions.clear();
        self.identities.clear();
        self.parents.clear();
        self.subtree_ends.clear();
        self.selection.clear();
        self.selection_owners.clear();
        self.selection_highlight_owners.clear();
        self.directions.clear();
        self.transition_nodes = 0;
        self.layout_roots.clear();
        self.visit(root, ids, NONE);
        self.sync_identities();
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
        let highlight_owner = if element.selection_highlight.is_some() {
            position as u32
        } else {
            inherited.map_or(NONE, |p| self.selection_highlight_owners[p])
        };
        let direction = element
            .direction_scope
            .or_else(|| inherited.and_then(|p| self.directions[p]));
        self.positions.insert(ids[position], position);
        self.elements.push(element.clone());
        self.parents.push(parent);
        self.subtree_ends.push(0);
        self.transition_nodes += has_transitions(element) as usize;
        if is_layout_root(element) {
            self.layout_roots.push(position);
        }
        self.selection.push(policy);
        self.selection_owners.push(owner);
        self.selection_highlight_owners.push(highlight_owner);
        self.directions.push(direction);
        for child in &element.children {
            self.visit(child, ids, position as u32);
        }
        self.subtree_ends[position] = self.elements.len() as u32;
    }

    pub(super) fn subtree_ends(&self) -> &[u32] {
        &self.subtree_ends
    }

    pub(super) fn has_transitions(&self) -> bool {
        self.transition_nodes != 0
    }

    pub(super) fn layout_roots(&self) -> &[usize] {
        &self.layout_roots
    }

    /// Refresh descriptions while identities and topology are unchanged. Shared
    /// subtrees need no visit unless their inherited selection/direction changed.
    pub(super) fn sync(&mut self, root: &Element) -> bool {
        self.sync_node(root, 0).1
    }

    /// Rebuilds unique source-identity targets after element descriptions change.
    fn sync_identities(&mut self) {
        self.identities.clear();
        for (node, index) in &self.positions {
            let Some(identity) = self.elements[*index].source_identity() else {
                continue;
            };
            self.identities
                .entry(identity.clone())
                .and_modify(|target| *target = None)
                .or_insert(Some(*node));
        }
    }

    /// Resolves a source identity only when exactly one retained node owns it.
    ///
    /// * `identity` — producer identity of a current element.
    pub(super) fn identity(&self, identity: &RetainedIdentity) -> Option<NodeId> {
        self.identities.get(identity).copied().flatten()
    }

    fn sync_node(&mut self, element: &Element, position: usize) -> (usize, bool) {
        let parent = self.parent(position);
        let policy = match element.user_select {
            UserSelect::Auto => parent.map_or(UserSelect::Text, |p| self.selection[p]),
            explicit => explicit,
        };
        let owner = if element.selection_style.is_some() {
            position as u32
        } else {
            parent.map_or(NONE, |p| self.selection_owners[p])
        };
        let highlight_owner = if element.selection_highlight.is_some() {
            position as u32
        } else {
            parent.map_or(NONE, |p| self.selection_highlight_owners[p])
        };
        let direction = element
            .direction_scope
            .or_else(|| parent.and_then(|p| self.directions[p]));
        let shared = self.elements[position].ptr_eq(element);
        if shared
            && self.selection[position] == policy
            && self.selection_owners[position] == owner
            && self.selection_highlight_owners[position] == highlight_owner
            && self.directions[position] == direction
        {
            return (self.subtree_ends[position] as usize, false);
        }
        let mut bindings_changed = false;
        if !shared {
            if is_layout_root(&self.elements[position]) != is_layout_root(element) {
                match self.layout_roots.binary_search(&position) {
                    Ok(slot) => {
                        self.layout_roots.remove(slot);
                    }
                    Err(slot) => self.layout_roots.insert(slot, position),
                }
            }
            bindings_changed = self.elements[position].bindings != element.bindings;
            self.transition_nodes += has_transitions(element) as usize;
            self.transition_nodes -= has_transitions(&self.elements[position]) as usize;
            self.elements[position] = element.clone();
        }
        self.selection[position] = policy;
        self.selection_owners[position] = owner;
        self.selection_highlight_owners[position] = highlight_owner;
        self.directions[position] = direction;
        let mut cursor = position + 1;
        for child in &element.children {
            let (end, changed) = self.sync_node(child, cursor);
            cursor = end;
            bindings_changed |= changed;
        }
        (cursor, bindings_changed)
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

    /// Resolves the nearest inherited highlight, falling back to the resolved solid color.
    pub(crate) fn selection_highlight(&self, node: NodeId) -> TextSelectionHighlight {
        let Some(index) = self.position(node) else {
            return TextSelectionHighlight::solid(self.default_selection_style.background);
        };
        let owner = self.selection_highlight_owners[index];
        self.elements
            .get(owner as usize)
            .and_then(|element| element.selection_highlight.as_ref())
            .cloned()
            .unwrap_or_else(|| TextSelectionHighlight::solid(self.selection_style(node).background))
    }
}

fn has_transitions(element: &Element) -> bool {
    element.has_state_animation()
        || element
            .scroll
            .as_ref()
            .is_some_and(|config| config.scrollbar.is_some())
}

fn is_layout_root(element: &Element) -> bool {
    element.layout_boundary || element.portal.is_some()
}
