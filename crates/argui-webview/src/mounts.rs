use crate::{WebViewMount, WebViewState};
use argui_layout::LayoutOutput;
use argui_ui::{ElementKind, UiTree};

/// Resolves native rectangles from the same retained layout as the GPU renderer.
/// Partially clipped or covered native surfaces are hidden, not painted above overlays.
#[must_use]
pub fn resolve_mounts(ui: &UiTree, layout: &LayoutOutput, host: u64) -> Vec<WebViewMount> {
    resolve_clipped_mounts(ui, layout, host)
        .into_iter()
        .map(|(mut mount, clip)| {
            mount.occluded |=
                clip.is_some_and(|clip| mount.bounds.intersection(clip) != Some(mount.bounds));
            mount
        })
        .collect()
}

/// DOM surfaces support rectangular clipping without resizing their document viewport.
#[must_use]
pub fn resolve_clipped_mounts(
    ui: &UiTree,
    layout: &LayoutOutput,
    host: u64,
) -> Vec<(WebViewMount, Option<argui_core::Rect>)> {
    let mut mounts = Vec::new();
    for (index, node) in layout.nodes.iter().enumerate() {
        let Some(element) = ui.element_at(node.index) else {
            continue;
        };
        let Some(state) = element
            .native_content
            .as_ref()
            .and_then(|content| content.downcast_ref::<WebViewState>())
        else {
            continue;
        };
        let clipped = node
            .clip
            .map_or(Some(node.bounds), |clip| node.bounds.intersection(clip));
        if clipped.is_none() {
            continue;
        }
        let mut occluded = false;
        let mut ancestor = Some(node.node);
        while let Some(id) = ancestor {
            if let Some(position) = ui.node_ids().iter().position(|candidate| *candidate == id)
                && let Some(parent) = ui.element_at(position)
                && (parent.transform != argui_core::Transform2D::IDENTITY
                    || parent.paint.quad.opacity < 1.0
                    || parent.layer.is_some())
            {
                occluded = true;
            }
            ancestor = ui.parent_of(id);
        }
        occluded |= layout.nodes[index + 1..].iter().any(|later| {
            let Some(element) = ui.element_at(later.index) else {
                return false;
            };
            if !element.paint.is_visible()
                && matches!(element.kind, ElementKind::Container)
                && element.native_content.is_none()
            {
                return false;
            }
            let bounds = later
                .clip
                .map_or(Some(later.bounds), |clip| later.bounds.intersection(clip));
            bounds.is_some_and(|bounds| bounds.intersection(node.bounds).is_some())
        });
        mounts.push((
            WebViewMount {
                state: state.clone(),
                host,
                bounds: node.bounds,
                occluded,
            },
            node.clip,
        ));
    }
    mounts
}
