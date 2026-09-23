use crate::{Element, ElementKind, NodeId};
use argui_core::Name;
use std::collections::{HashMap, HashSet, VecDeque};

/// Opaque owner/site identity used to retain a node across rebuilds and moves.
///
/// This identity is independent from application keys, focus targets, and
/// accessibility identifiers. The `owner` identifies a producer and `site`
/// identifies one stable node within it.
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
    /// Creates one stable node identity within an owner.
    ///
    /// * `owner` — stable identity of the producer.
    /// * `site` — stable identity of the node within that producer.
    #[must_use]
    pub const fn new(owner: u64, site: u64) -> Self {
        Self {
            owner,
            site,
            item: None,
        }
    }

    /// Adds a signed key to this node identity.
    ///
    /// * `key` — signed item key distinguishing repeated nodes.
    #[must_use]
    pub fn with_signed_key(mut self, key: i64) -> Self {
        self.item = Some(RetainedItemKey::Signed(key));
        self
    }

    /// Adds an unsigned key to this node identity.
    ///
    /// * `key` — unsigned item key distinguishing repeated nodes.
    #[must_use]
    pub fn with_unsigned_key(mut self, key: u64) -> Self {
        self.item = Some(RetainedItemKey::Unsigned(key));
        self
    }

    /// Adds a string key to this node identity.
    ///
    /// * `key` — string item key distinguishing repeated nodes.
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
    if same_topology(old, new) {
        return old_ids.to_vec();
    }
    if let Some(ids) = reordered_shared_children(old, old_ids, old_ends, new) {
        return ids;
    }
    let mut retained = HashMap::new();
    collect_retained(old, 0, old_ends, &mut retained);
    let mut reconcile = Reconcile {
        old_ids,
        old_ends,
        next,
        ids: Vec::with_capacity(old_ids.len()),
        retained,
        claimed: HashSet::new(),
    };
    reconcile.visit(Some((old, 0)), new);
    reconcile.ids
}

/// Reorders stable ID slices when direct children are shared, uniquely identified subtrees.
///
/// * `old` — current parent and its retained children.
/// * `old_ids` — node IDs in the current preorder.
/// * `old_ends` — exclusive subtree ends in that preorder.
/// * `new` — replacement parent whose children may have moved.
///
/// Returns reordered IDs for a pure move, or `None` when ordinary reconciliation is needed.
fn reordered_shared_children(
    old: &Element,
    old_ids: &[NodeId],
    old_ends: &[u32],
    new: &Element,
) -> Option<Vec<NodeId>> {
    if !compatible(old, new) || old.children.len() != new.children.len() {
        return None;
    }
    let mut children = HashMap::with_capacity(old.children.len());
    let mut cursor = 1;
    for child in &old.children {
        let identity = child.retained_identity.as_ref()?;
        let end = *old_ends.get(cursor)? as usize;
        if children.insert(identity, (child, cursor, end)).is_some() {
            return None;
        }
        cursor = end;
    }
    let mut seen = HashSet::with_capacity(new.children.len());
    let mut ids = Vec::with_capacity(old_ids.len());
    ids.push(*old_ids.first()?);
    for child in &new.children {
        let identity = child.retained_identity.as_ref()?;
        if !seen.insert(identity) {
            return None;
        }
        let (previous, start, end) = children.get(identity)?;
        if !previous.ptr_eq(child) {
            return None;
        }
        ids.extend_from_slice(old_ids.get(*start..*end)?);
    }
    (ids.len() == old_ids.len()).then_some(ids)
}

/// Returns whether ordinal positions can retain their existing node IDs.
///
/// * `old` — the currently retained subtree.
/// * `new` — the replacement subtree.
///
/// Shared descriptions skip their entire descendants; keyed moves and
/// additions fall back to the full reconciliation path.
fn same_topology(old: &Element, new: &Element) -> bool {
    if old.ptr_eq(new) {
        return true;
    }
    compatible(old, new)
        && old.children.len() == new.children.len()
        && old
            .children
            .iter()
            .zip(&new.children)
            .all(|(old, new)| same_topology(old, new))
}

struct Reconcile<'a> {
    old_ids: &'a [NodeId],
    old_ends: &'a [u32],
    next: &'a mut u64,
    ids: Vec<NodeId>,
    retained: HashMap<&'a RetainedIdentity, VecDeque<(&'a Element, usize)>>,
    claimed: HashSet<usize>,
}

impl Reconcile<'_> {
    fn visit(&mut self, old: Option<(&Element, usize)>, new: &Element) {
        let reusable = old.filter(|(element, _)| compatible(element, new));
        if let Some((element, index)) = reusable
            && element.ptr_eq(new)
        {
            self.claimed.extend(index..self.old_ends[index] as usize);
            self.ids
                .extend_from_slice(&self.old_ids[index..self.old_ends[index] as usize]);
            return;
        }
        if let Some((_, index)) = reusable {
            self.claimed.insert(index);
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
        for (index, child) in new.children.iter().enumerate() {
            let candidate = match (&child.retained_identity, child.key.as_deref()) {
                (Some(identity), _) => retained.get_mut(identity).and_then(VecDeque::pop_front),
                (None, Some(key)) => keyed.get_mut(key).and_then(VecDeque::pop_front),
                (None, None) => children.get(index).copied().filter(|(element, _)| {
                    element.key.is_none() && element.retained_identity.is_none()
                }),
            }
            .filter(|(_, index)| !self.claimed.contains(index));
            let candidate = candidate.or_else(|| {
                child.retained_identity.as_ref().and_then(|identity| {
                    self.retained.get_mut(identity).and_then(|matches| {
                        matches
                            .iter()
                            .position(|(_, index)| !self.claimed.contains(index))
                            .and_then(|position| matches.remove(position))
                    })
                })
            });
            self.visit(candidate, child);
        }
    }
}

fn collect_retained<'a>(
    element: &'a Element,
    index: usize,
    ends: &[u32],
    retained: &mut HashMap<&'a RetainedIdentity, VecDeque<(&'a Element, usize)>>,
) {
    if let Some(identity) = element.retained_identity.as_ref() {
        retained
            .entry(identity)
            .or_default()
            .push_back((element, index));
    }
    let mut cursor = index + 1;
    for child in &element.children {
        collect_retained(child, cursor, ends, retained);
        cursor = ends[cursor] as usize;
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
    let same_kind = if let (ElementKind::Custom(old_kind), ElementKind::Custom(new_kind)) =
        (&old.kind, &new.kind)
    {
        old_kind.same_type(new_kind)
    } else {
        matches!(
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
    };
    same_kind && (old.retained_identity.is_some() || old.key == new.key)
}

fn allocate(next: &mut u64) -> NodeId {
    let id = NodeId::new(*next);
    *next = next.wrapping_add(1).max(1);
    id
}
