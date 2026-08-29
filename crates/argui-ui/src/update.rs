use crate::{Element, ElementKind, TreeUpdate, TreeUpdateStats};

pub(crate) fn classify_update(
    old: &Element,
    new: &Element,
    stats: &mut TreeUpdateStats,
) -> TreeUpdate {
    stats.visited += 1;
    if old.ptr_eq(new) {
        stats.shared_subtrees += 1;
        return TreeUpdate::None;
    }
    if old == new {
        return TreeUpdate::None;
    }
    if old.key != new.key
        || kind_changes_layout(&old.kind, &new.kind)
        || old.style != new.style
        || old.paint.clip != new.paint.clip
        || old.scroll.is_some() != new.scroll.is_some()
        || old.overlay != new.overlay
        || old.children.len() != new.children.len()
    {
        return TreeUpdate::Layout;
    }
    let children = old
        .children
        .iter()
        .zip(&new.children)
        .map(|(old, new)| classify_update(old, new, stats))
        .max_by_key(|update| update_priority(*update))
        .unwrap_or(TreeUpdate::None);
    if children == TreeUpdate::Layout {
        return TreeUpdate::Layout;
    }
    if visual_changed(old, new) {
        TreeUpdate::Paint
    } else if old.semantics != new.semantics
        || old.semantic_hidden != new.semantic_hidden
        || children == TreeUpdate::Semantics
    {
        TreeUpdate::Semantics
    } else {
        children
    }
}

const fn update_priority(update: TreeUpdate) -> u8 {
    match update {
        TreeUpdate::None => 0,
        TreeUpdate::Semantics => 1,
        TreeUpdate::Paint => 2,
        TreeUpdate::Layout => 3,
    }
}

fn visual_changed(old: &Element, new: &Element) -> bool {
    old.inspectable != new.inspectable
        || old.kind != new.kind
        || old.paint != new.paint
        || old.transform != new.transform
        || old.transform_origin != new.transform_origin
        || old.interaction != new.interaction
        || old.transition != new.transition
        || old.layer != new.layer
        || old.effects != new.effects
        || old.scroll != new.scroll
        || old.z_index != new.z_index
}

fn kind_changes_layout(old: &ElementKind, new: &ElementKind) -> bool {
    match (old, new) {
        (ElementKind::Container, ElementKind::Container)
        | (ElementKind::Image { .. }, ElementKind::Image { .. })
        | (ElementKind::Vector { .. }, ElementKind::Vector { .. }) => false,
        (ElementKind::Text { .. }, ElementKind::Text { .. })
        | (ElementKind::TextInput { .. }, ElementKind::TextInput { .. }) => old != new,
        _ => true,
    }
}
