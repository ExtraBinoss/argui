use crate::{Element, NodeId, UiTree};
use argui_accessibility::{SemanticNodeId, SemanticTree};
use std::collections::{HashMap, HashSet};

/// A key relative to the nearest explicit semantic scope (the window by default).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SemanticTarget(pub String);

impl From<&str> for SemanticTarget {
    fn from(key: &str) -> Self {
        Self(key.into())
    }
}
impl From<String> for SemanticTarget {
    fn from(key: String) -> Self {
        Self(key)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticBindings {
    pub labelled_by: Vec<SemanticTarget>,
    pub described_by: Vec<SemanticTarget>,
    pub controls: Vec<SemanticTarget>,
    pub active_descendant: Option<SemanticTarget>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticReferenceError {
    Missing,
    Ambiguous,
    SelfReference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub source: NodeId,
    pub target: SemanticTarget,
    pub error: SemanticReferenceError,
}

impl Element {
    #[must_use]
    pub fn semantics(mut self, semantics: argui_accessibility::Semantics) -> Self {
        self.semantics = Some(Box::new(semantics));
        self
    }

    #[must_use]
    pub fn semantic_hidden(mut self, hidden: bool) -> Self {
        self.semantic_hidden = hidden;
        self
    }

    /// Creates a namespace for accessible references, retained across visual portals.
    #[must_use]
    pub fn semantic_scope(mut self) -> Self {
        if !self.semantic_scope {
            self.semantic_scope = true;
        }
        self
    }

    #[must_use]
    pub fn labelled_by(
        mut self,
        targets: impl IntoIterator<Item = impl Into<SemanticTarget>>,
    ) -> Self {
        self.semantic_bindings.labelled_by = targets.into_iter().map(Into::into).collect();
        self
    }
    #[must_use]
    pub fn described_by(
        mut self,
        targets: impl IntoIterator<Item = impl Into<SemanticTarget>>,
    ) -> Self {
        self.semantic_bindings.described_by = targets.into_iter().map(Into::into).collect();
        self
    }
    #[must_use]
    pub fn controls(
        mut self,
        targets: impl IntoIterator<Item = impl Into<SemanticTarget>>,
    ) -> Self {
        self.semantic_bindings.controls = targets.into_iter().map(Into::into).collect();
        self
    }
    #[must_use]
    pub fn active_descendant(mut self, target: impl Into<SemanticTarget>) -> Self {
        self.semantic_bindings.active_descendant = Some(target.into());
        self
    }
}

impl UiTree {
    /// On-demand diagnostics; normal semantic snapshots omit unresolved references.
    #[must_use]
    pub fn semantic_diagnostics(&self) -> Vec<SemanticDiagnostic> {
        let mut semantic = self.semantic_tree(&[], 1.0);
        resolve(self, &mut semantic)
    }
}

pub(crate) fn resolve(tree: &UiTree, semantic: &mut SemanticTree) -> Vec<SemanticDiagnostic> {
    let visible: HashSet<_> = semantic.nodes.iter().map(|node| node.id).collect();
    let mut scopes = HashMap::new();
    let mut keys: HashMap<(NodeId, &str), Vec<NodeId>> = HashMap::new();
    let mut declarations = Vec::new();
    fn collect<'a>(
        element: &'a Element,
        ids: &mut impl Iterator<Item = NodeId>,
        scope: NodeId,
        scopes: &mut HashMap<NodeId, NodeId>,
        keys: &mut HashMap<(NodeId, &'a str), Vec<NodeId>>,
        declarations: &mut Vec<(NodeId, &'a SemanticBindings)>,
    ) {
        let node = ids.next().expect("retained node identity");
        // A scope's root key belongs to its own scope, along with its descendants.
        let scope = if element.semantic_scope { node } else { scope };
        scopes.insert(node, scope);
        if let Some(key) = &element.key {
            keys.entry((scope, key)).or_default().push(node);
        }
        declarations.push((node, &element.semantic_bindings));
        for child in &element.children {
            collect(child, ids, scope, scopes, keys, declarations);
        }
    }
    collect(
        tree.root(),
        &mut tree.node_ids().iter().copied(),
        tree.node_ids()[0],
        &mut scopes,
        &mut keys,
        &mut declarations,
    );
    let mut diagnostics = Vec::new();
    let positions: HashMap<_, _> = semantic
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, i))
        .collect();
    for (source, bindings) in declarations {
        let Some(&position) = positions.get(&SemanticNodeId::new(source.get())) else {
            continue;
        };
        let mut target = |key: &SemanticTarget| {
            let candidates = keys.get(&(scopes[&source], key.0.as_str()));
            let error = match candidates.map(Vec::as_slice) {
                Some([node]) if *node == source => SemanticReferenceError::SelfReference,
                Some([node]) if visible.contains(&SemanticNodeId::new(node.get())) => {
                    return Some(SemanticNodeId::new(node.get()));
                }
                Some(nodes) if nodes.len() > 1 => SemanticReferenceError::Ambiguous,
                _ => SemanticReferenceError::Missing,
            };
            diagnostics.push(SemanticDiagnostic {
                source,
                target: key.clone(),
                error,
            });
            None
        };
        let relations = &mut semantic.nodes[position].semantics.relations;
        for (references, output) in [
            (&bindings.labelled_by, &mut relations.labelled_by),
            (&bindings.described_by, &mut relations.described_by),
            (&bindings.controls, &mut relations.controls),
        ] {
            let mut seen = HashSet::new();
            *output = references
                .iter()
                .filter_map(&mut target)
                .filter(|node| seen.insert(*node))
                .collect();
        }
        relations.active_descendant = bindings.active_descendant.as_ref().and_then(&mut target);
    }
    diagnostics
}
