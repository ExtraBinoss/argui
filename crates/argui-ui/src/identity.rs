use crate::{Element, ElementKind, NodeId};
use argui_core::Name;
use std::collections::{HashMap, HashSet, VecDeque};

/// Opaque compiler/runtime identity used to retain a source site across rebuilds.
///
/// This identity is independent from application keys, focus targets, and
/// accessibility identifiers. The `owner` is a component-instance identity and
/// `site` is a stable compiled source-site identity.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RetainedIdentity {
    owner: u64,
    site: u64,
    item: Option<RetainedItemKey>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RetainedItemKey {
    Signed(i64),
    Unsigned(u64),
    Name(Name),
}

impl RetainedIdentity {
    /// Creates the identity of one compiled source site in a component instance.
    ///
    /// * `owner` — stable identity of the owning component instance.
    /// * `site` — stable identity of the source site inside its component definition.
    #[must_use]
    pub const fn new(owner: u64, site: u64) -> Self {
        Self {
            owner,
            site,
            item: None,
        }
    }

    /// Adds a signed integer repeater key to this source-site identity.
    ///
    /// * `key` — model key identifying one repeated component instance.
    #[must_use]
    pub fn with_signed_key(mut self, key: i64) -> Self {
        self.item = Some(RetainedItemKey::Signed(key));
        self
    }

    /// Adds an unsigned integer repeater key to this source-site identity.
    ///
    /// * `key` — model key identifying one repeated component instance.
    #[must_use]
    pub fn with_unsigned_key(mut self, key: u64) -> Self {
        self.item = Some(RetainedItemKey::Unsigned(key));
        self
    }

    /// Adds a string repeater key to this source-site identity.
    ///
    /// * `key` — model key identifying one repeated component instance.
    #[must_use]
    pub fn with_name_key(mut self, key: impl Into<Name>) -> Self {
        self.item = Some(RetainedItemKey::Name(key.into()));
        self
    }
}

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
        let mut retained = HashMap::<&RetainedIdentity, VecDeque<(&Element, usize)>>::new();
        let mut keyed = HashMap::<&str, VecDeque<(&Element, usize)>>::new();
        let mut children = Vec::new();
        if let Some((parent, index)) = reusable {
            let mut cursor = index + 1;
            for child in &parent.children {
                children.push((child, cursor));
                if let Some(identity) = child.retained_identity.as_ref() {
                    retained
                        .entry(identity)
                        .or_default()
                        .push_back((child, cursor));
                } else if let Some(key) = child.key.as_deref() {
                    keyed.entry(key).or_default().push_back((child, cursor));
                }
                cursor = self.old_ends[cursor] as usize;
            }
        }
        let mut used = HashSet::new();
        for (index, child) in new.children.iter().enumerate() {
            let candidate = match (&child.retained_identity, child.key.as_deref()) {
                (Some(identity), _) => retained.get_mut(identity).and_then(VecDeque::pop_front),
                (None, Some(key)) => keyed.get_mut(key).and_then(VecDeque::pop_front),
                (None, None) => children.get(index).copied().filter(|(element, _)| {
                    element.key.is_none() && element.retained_identity.is_none()
                }),
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
    if (old.retained_identity.is_some() || new.retained_identity.is_some())
        && old.retained_identity != new.retained_identity
    {
        return false;
    }
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
