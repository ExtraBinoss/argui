use crate::{BindingImpact, Element, ElementKind, TreeUpdate, TreeUpdateStats};

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
    let binding_update = binding_update(old, new);
    let state_update = state_update(old, new);
    if old.key != new.key
        || kind_changes_layout(&old.kind, &new.kind)
        || old.style != new.style
        || old.container_scope != new.container_scope
        || (old.scroll != new.scroll
            && (old.has_container_queries() || new.has_container_queries()))
        || old.scroll.is_some() != new.scroll.is_some()
        || old.overlay != new.overlay
        || old.children.len() != new.children.len()
        || binding_update == TreeUpdate::Layout
        || state_update == TreeUpdate::Layout
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
    let local = if state_update != TreeUpdate::None {
        state_update
    } else if visual_changed(old, new) {
        TreeUpdate::Paint
    } else if binding_update != TreeUpdate::None {
        binding_update
    } else if old.semantics != new.semantics
        || old.semantic_hidden != new.semantic_hidden
        || old.focus_scope != new.focus_scope
    {
        TreeUpdate::Semantics
    } else {
        TreeUpdate::None
    };
    strongest_update(children, local)
}

const fn update_priority(update: TreeUpdate) -> u8 {
    match update {
        TreeUpdate::None => 0,
        TreeUpdate::Semantics => 1,
        TreeUpdate::Paint => 2,
        TreeUpdate::Scroll => 3,
        TreeUpdate::Layout => 4,
    }
}

fn visual_changed(old: &Element, new: &Element) -> bool {
    old.inspectable != new.inspectable
        || old.kind != new.kind
        || old.paint != new.paint
        || old.transform != new.transform
        || old.transform_origin != new.transform_origin
        || old.interaction != new.interaction
        || old.hit_test != new.hit_test
        || old.style_transition != new.style_transition
        || old.state_scope != new.state_scope
        || old.container_scope != new.container_scope
        || old.active_states != new.active_states
        || old.layer != new.layer
        || old.effects != new.effects
        || old.scroll != new.scroll
        || old.event_owner != new.event_owner
        || old.event_listeners != new.event_listeners
        || old.user_select != new.user_select
        || old.selection_style != new.selection_style
        || old.z_index != new.z_index
}

fn state_update(old: &Element, new: &Element) -> TreeUpdate {
    if old.conditional_styles == new.conditional_styles
        && old.style_transition == new.style_transition
        && old.state_scope == new.state_scope
        && old.container_scope == new.container_scope
        && old.active_states == new.active_states
        && old.interaction == new.interaction
    {
        return TreeUpdate::None;
    }
    if old.conditional_styles.has_container_queries()
        || new.conditional_styles.has_container_queries()
    {
        return TreeUpdate::Layout;
    }
    old.conditional_styles
        .impact()
        .into_iter()
        .chain(new.conditional_styles.impact())
        .fold(TreeUpdate::Paint, |update, impact| {
            strongest_update(
                update,
                match impact {
                    BindingImpact::Paint => TreeUpdate::Paint,
                    BindingImpact::Scroll => TreeUpdate::Scroll,
                    BindingImpact::Layout => TreeUpdate::Layout,
                },
            )
        })
}

fn binding_update(old: &Element, new: &Element) -> TreeUpdate {
    if old.bindings == new.bindings {
        return TreeUpdate::None;
    }
    old.bindings
        .iter()
        .chain(&new.bindings)
        .fold(TreeUpdate::None, |update, binding| {
            let next = match binding.impact() {
                BindingImpact::Paint => TreeUpdate::Paint,
                BindingImpact::Scroll => TreeUpdate::Scroll,
                BindingImpact::Layout => TreeUpdate::Layout,
            };
            strongest_update(update, next)
        })
}

pub(crate) fn strongest_update(left: TreeUpdate, right: TreeUpdate) -> TreeUpdate {
    if update_priority(left) >= update_priority(right) {
        left
    } else {
        right
    }
}

fn kind_changes_layout(old: &ElementKind, new: &ElementKind) -> bool {
    match (old, new) {
        (ElementKind::Container, ElementKind::Container)
        | (ElementKind::Image { .. }, ElementKind::Image { .. })
        | (ElementKind::Vector { .. }, ElementKind::Vector { .. }) => false,
        (ElementKind::Text { .. }, ElementKind::Text { .. })
        | (ElementKind::TextEditor { .. }, ElementKind::TextEditor { .. }) => old != new,
        _ => true,
    }
}
