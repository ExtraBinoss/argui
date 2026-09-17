use crate::{Element, ElementKind, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

pub(crate) fn initial_ids(root: &Element, next: &mut u64) -> Vec<NodeId> {
    let mut ids = Vec::new();
    assign_new(root, next, &mut ids);
    ids
}

pub(crate) fn reconcile_ids(
    old: &Element,
    old_ids: &[NodeId],
    old_ends: &[u32],
    new: &Element,
    next: &mut u64,
) -> Vec<NodeId> {
    let mut reconcile = Reconcile {
        old_ids,
        old_ends,
        next,
        ids: Vec::with_capacity(old_ids.len()),
    };
    reconcile.visit(Some((old, 0)), new);
    reconcile.ids
}

struct Reconcile<'a> {
    old_ids: &'a [NodeId],
    old_ends: &'a [u32],
    next: &'a mut u64,
    ids: Vec<NodeId>,
}

impl Reconcile<'_> {
    fn visit(&mut self, old: Option<(&Element, usize)>, new: &Element) {
        let reusable = old.filter(|(element, _)| compatible(element, new));
        if let Some((element, index)) = reusable
            && element.ptr_eq(new)
        {
            self.ids
                .extend_from_slice(&self.old_ids[index..self.old_ends[index] as usize]);
            return;
        }
        self.ids
            .push(reusable.map_or_else(|| allocate(self.next), |(_, index)| self.old_ids[index]));
        let mut keyed = HashMap::<&str, VecDeque<(&Element, usize)>>::new();
        let mut children = Vec::new();
        if let Some((parent, index)) = reusable {
            let mut cursor = index + 1;
            for child in &parent.children {
                children.push((child, cursor));
                if let Some(key) = child.key.as_deref() {
                    keyed.entry(key).or_default().push_back((child, cursor));
                }
                cursor = self.old_ends[cursor] as usize;
            }
        }
        let mut used = HashSet::new();
        for (index, child) in new.children.iter().enumerate() {
            let candidate = match child.key.as_deref() {
                Some(key) => keyed.get_mut(key).and_then(VecDeque::pop_front),
                None => children
                    .get(index)
                    .copied()
                    .filter(|(element, _)| element.key.is_none()),
            }
            .filter(|(_, index)| !used.contains(index));
            if let Some((_, index)) = candidate {
                used.insert(index);
            }
            self.visit(candidate, child);
        }
    }
}

fn assign_new(element: &Element, next: &mut u64, ids: &mut Vec<NodeId>) {
    ids.push(allocate(next));
    for child in &element.children {
        assign_new(child, next, ids);
    }
}

fn compatible(old: &Element, new: &Element) -> bool {
    if let (ElementKind::Custom(old_kind), ElementKind::Custom(new_kind)) = (&old.kind, &new.kind) {
        return old.key == new.key && old_kind.same_type(new_kind);
    }
    old.key == new.key
        && matches!(
            (&old.kind, &new.kind),
            (ElementKind::Container, ElementKind::Container)
                | (ElementKind::GpuCanvas(_), ElementKind::GpuCanvas(_))
                | (ElementKind::Text { .. }, ElementKind::Text { .. })
                | (
                    ElementKind::TextEditor { .. },
                    ElementKind::TextEditor { .. }
                )
                | (ElementKind::Image { .. }, ElementKind::Image { .. })
                | (ElementKind::Vector { .. }, ElementKind::Vector { .. })
        )
}

fn allocate(next: &mut u64) -> NodeId {
    let id = NodeId::new(*next);
    *next = next.wrapping_add(1).max(1);
    id
}
