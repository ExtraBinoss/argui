use crate::{Element, ElementKind, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
struct Existing<'a> {
    element: &'a Element,
    id: NodeId,
    children: Vec<Self>,
}

pub(crate) fn initial_ids(root: &Element, next: &mut u64) -> Vec<NodeId> {
    let mut ids = Vec::new();
    assign_new(root, next, &mut ids);
    ids
}

pub(crate) fn reconcile_ids(
    old: &Element,
    old_ids: &[NodeId],
    new: &Element,
    next: &mut u64,
) -> Vec<NodeId> {
    let mut cursor = 0;
    let existing = existing_tree(old, old_ids, &mut cursor);
    let mut ids = Vec::new();
    reconcile_node(Some(&existing), new, next, &mut ids);
    ids
}

fn assign_new(element: &Element, next: &mut u64, ids: &mut Vec<NodeId>) {
    ids.push(allocate(next));
    for child in &element.children {
        assign_new(child, next, ids);
    }
}

fn existing_tree<'a>(element: &'a Element, ids: &[NodeId], cursor: &mut usize) -> Existing<'a> {
    let id = ids[*cursor];
    *cursor += 1;
    let children = element
        .children
        .iter()
        .map(|child| existing_tree(child, ids, cursor))
        .collect();
    Existing {
        element,
        id,
        children,
    }
}

fn reconcile_node(
    old: Option<&Existing<'_>>,
    new: &Element,
    next: &mut u64,
    ids: &mut Vec<NodeId>,
) {
    let reusable = old.filter(|candidate| compatible(candidate.element, new));
    ids.push(reusable.map_or_else(|| allocate(next), |existing| existing.id));

    let mut keyed = HashMap::<&str, VecDeque<&Existing<'_>>>::new();
    if let Some(parent) = reusable {
        for existing in &parent.children {
            if let Some(key) = existing.element.key.as_deref() {
                keyed.entry(key).or_default().push_back(existing);
            }
        }
    }
    let mut used = HashSet::new();
    for (index, child) in new.children.iter().enumerate() {
        let candidate = match child.key.as_deref() {
            Some(key) => keyed.get_mut(key).and_then(VecDeque::pop_front),
            None => reusable
                .and_then(|parent| parent.children.get(index))
                .filter(|existing| existing.element.key.is_none()),
        }
        .filter(|existing| !used.contains(&existing.id));
        if let Some(existing) = candidate {
            used.insert(existing.id);
        }
        reconcile_node(candidate, child, next, ids);
    }
}

fn compatible(old: &Element, new: &Element) -> bool {
    old.key == new.key
        && matches!(
            (&old.kind, &new.kind),
            (ElementKind::Container, ElementKind::Container)
                | (ElementKind::Text { .. }, ElementKind::Text { .. })
                | (ElementKind::TextInput { .. }, ElementKind::TextInput { .. })
                | (ElementKind::Image { .. }, ElementKind::Image { .. })
                | (ElementKind::Vector { .. }, ElementKind::Vector { .. })
        )
}

fn allocate(next: &mut u64) -> NodeId {
    let id = NodeId::new(*next);
    *next = next.wrapping_add(1).max(1);
    id
}
