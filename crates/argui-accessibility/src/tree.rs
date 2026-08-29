use std::collections::{HashMap, HashSet};

use argui_core::Rect;

use crate::Semantics;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticNodeId(u64);

impl SemanticNodeId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticNode {
    pub id: SemanticNodeId,
    pub bounds: Rect,
    pub semantics: Semantics,
    pub children: Vec<SemanticNodeId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticTree {
    pub root: SemanticNodeId,
    pub focus: SemanticNodeId,
    pub nodes: Vec<SemanticNode>,
}

impl SemanticTree {
    #[must_use]
    pub fn node(&self, id: SemanticNodeId) -> Option<&SemanticNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    #[must_use]
    pub fn diff(&self, next: &Self) -> SemanticPatch {
        let old = self
            .nodes
            .iter()
            .map(|node| (node.id, node))
            .collect::<HashMap<_, _>>();
        let next_ids = next
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<HashSet<_>>();
        SemanticPatch {
            root: (self.root != next.root).then_some(next.root),
            focus: (self.focus != next.focus).then_some(next.focus),
            upserts: next
                .nodes
                .iter()
                .filter(|node| old.get(&node.id).copied() != Some(*node))
                .cloned()
                .collect(),
            removed: self
                .nodes
                .iter()
                .filter(|node| !next_ids.contains(&node.id))
                .map(|node| node.id)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticPatch {
    pub root: Option<SemanticNodeId>,
    pub focus: Option<SemanticNodeId>,
    pub upserts: Vec<SemanticNode>,
    pub removed: Vec<SemanticNodeId>,
}

impl SemanticPatch {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
            && self.focus.is_none()
            && self.upserts.is_empty()
            && self.removed.is_empty()
    }
}
