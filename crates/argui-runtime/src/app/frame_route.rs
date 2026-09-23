use argui_ui::TreeUpdate;

/// Keeps a compositor-only frame only when no fresh layout will replace its scene.
///
/// `composite_frame` is the result of any motion sampled earlier in this frame.
/// `tree_update` is the host/tree reconciliation result. `pending_layout` and
/// `assets_changed` request layout independently of reconciliation. Returns
/// whether the renderer may present retained compositor commands alone.
pub(crate) fn retain_compositor_frame(
    composite_frame: bool,
    tree_update: TreeUpdate,
    pending_layout: bool,
    assets_changed: bool,
) -> bool {
    composite_frame && tree_update != TreeUpdate::Layout && !pending_layout && !assets_changed
}
