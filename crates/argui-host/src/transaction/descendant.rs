use argui_ui::Element;

/// Returns whether two elements retain the same materialized direct children.
///
/// * `previous` — the currently visible element.
/// * `current` — the newly built element.
///
/// A rectangle may reuse its prior element only when every child still points
/// to the same subtree. This prevents descendant edits from being hidden.
pub(super) fn same_children(previous: &Element, current: &Element) -> bool {
    previous.children.len() == current.children.len()
        && previous
            .children
            .iter()
            .zip(&current.children)
            .all(|(before, after)| before.ptr_eq(after))
}
