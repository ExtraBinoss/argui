use crate::{BindingImpact, Element, ElementKind, TreeUpdate, TreeUpdateStats};

pub(crate) fn classify_update(
    old: &Element,
    new: &mut Element,
    stats: &mut TreeUpdateStats,
) -> TreeUpdate {
    stats.visited += 1;
    if old.ptr_eq(new) {
        stats.shared_subtrees += 1;
        return TreeUpdate::None;
    }
    if old == new {
        // Equal descriptions keep the retained allocation used by the paint cache.
        *new = old.clone();
        return TreeUpdate::None;
    }
    let binding_update = binding_update(old, new);
    let state_update = state_update(old, new);
    if old.key != new.key
        || old.layout_boundary != new.layout_boundary
        || old.text_privacy != new.text_privacy
        || old.text_history != new.text_history
        || kind_changes_layout(old, new)
        || old.style != new.style
        || old.direction_scope != new.direction_scope
        || old.container_scope != new.container_scope
        || (old.scroll != new.scroll
            && (old.has_container_queries() || new.has_container_queries()))
        || old.scroll.is_some() != new.scroll.is_some()
        || old.portal != new.portal
        || old.virtual_item != new.virtual_item
        || old.children.len() != new.children.len()
        || binding_update == TreeUpdate::Layout
        || state_update == TreeUpdate::Layout
    {
        return TreeUpdate::Layout;
    }
    let children = old
        .children
        .iter()
        .zip(&mut new.children)
        .map(|(old, new)| classify_update(old, new, stats))
        .max_by_key(|update| update_priority(*update))
        .unwrap_or(TreeUpdate::None);
    let visual_update = visual_update(old, new).unwrap_or(TreeUpdate::None);
    let local = strongest_update(
        state_update,
        strongest_update(visual_update, binding_update),
    );
    let local = if local == TreeUpdate::Composite && !old.needs_compositor_layer() {
        // The first composite-capable value must establish its retained layer.
        TreeUpdate::Paint
    } else {
        local
    };
    let local = if local != TreeUpdate::None {
        local
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
        TreeUpdate::Composite => 2,
        TreeUpdate::Paint => 3,
        TreeUpdate::Scroll => 4,
        TreeUpdate::Layout => 5,
    }
}

/// Classifies non-layout element changes by their strongest visual phase.
fn visual_update(old: &Element, new: &Element) -> Option<TreeUpdate> {
    let paint = old.inspectable != new.inspectable
        || old.kind != new.kind
        || old.paint != new.paint
        || old.desktop_backdrop != new.desktop_backdrop
        || old.interaction != new.interaction
        || old.hit_test != new.hit_test
        || old.style_transition != new.style_transition
        || old.state_scope != new.state_scope
        || old.container_scope != new.container_scope
        || old.active_states != new.active_states
        || layer_content(old) != layer_content(new)
        || old.effects != new.effects
        || old.scroll != new.scroll
        || old.event_listeners != new.event_listeners
        || old.action_scope != new.action_scope
        || old.action != new.action
        || old.user_select != new.user_select
        || old.selection_style != new.selection_style
        || old.selection_highlight != new.selection_highlight
        || old.z_index != new.z_index;
    if paint {
        Some(TreeUpdate::Paint)
    } else if (old.transform != new.transform
        || old.transform_origin != new.transform_origin
        || layer_composition(old) != layer_composition(new))
        && old.needs_compositor_layer()
    {
        Some(TreeUpdate::Composite)
    } else if old.transform != new.transform
        || old.transform_origin != new.transform_origin
        || layer_composition(old) != layer_composition(new)
    {
        Some(TreeUpdate::Paint)
    } else {
        None
    }
}

/// Returns layer properties that affect retained pixels rather than presentation.
fn layer_content(element: &Element) -> Option<argui_paint::LayerStyle> {
    element.layer.as_ref().map(|layer| {
        let mut layer = layer.as_ref().clone();
        layer.opacity = 1.0;
        layer
    })
}

/// Returns the group-opacity portion of an element layer.
fn layer_composition(element: &Element) -> Option<f32> {
    element.layer.as_ref().map(|layer| layer.opacity)
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
        .fold(TreeUpdate::None, |update, impact| {
            strongest_update(
                update,
                match impact {
                    BindingImpact::Composite => TreeUpdate::Composite,
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
                BindingImpact::Composite => TreeUpdate::Composite,
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

fn kind_changes_layout(old: &Element, new: &Element) -> bool {
    match (&old.kind, &new.kind) {
        (ElementKind::Custom(old), ElementKind::Custom(new)) => !old.same_layout(new),
        (ElementKind::Container, ElementKind::Container)
        | (ElementKind::GpuCanvas(_), ElementKind::GpuCanvas(_))
        | (ElementKind::Image { .. }, ElementKind::Image { .. })
        | (ElementKind::Vector { .. }, ElementKind::Vector { .. }) => false,
        (
            ElementKind::Text {
                content: old_content,
                style: old_style,
            },
            ElementKind::Text {
                content: new_content,
                style: new_style,
            },
        ) => {
            // Base text color is refreshed by repaint; it cannot change glyph placement.
            let mut old_metrics = old_style.clone();
            old_metrics.color = new_style.color;
            old_metrics != *new_style
                || (old_content != new_content && !fixed_nonselectable_text(old, new))
        }
        (ElementKind::TextEditor { .. }, ElementKind::TextEditor { .. }) => old != new,
        _ => true,
    }
}

fn fixed_nonselectable_text(old: &Element, new: &Element) -> bool {
    old.user_select == crate::UserSelect::None
        && new.user_select == crate::UserSelect::None
        && old.style.size.width != crate::Dimension::auto()
        && old.style.size.height != crate::Dimension::auto()
        && new.style.size.width != crate::Dimension::auto()
        && new.style.size.height != crate::Dimension::auto()
}
