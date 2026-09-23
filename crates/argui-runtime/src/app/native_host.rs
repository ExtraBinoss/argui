//! Native presentation commits applied on the UI thread.

use argui_core::Insets;
#[cfg(target_os = "android")]
use argui_paint::Fill;
use argui_ui::{Element, TreeUpdate, UiTree, percent};

use super::Application;
use crate::{
    HostCommitResult, NativeHostAssets, NativeHostCommit, WireOperation,
    validate_native_host_assets,
};

/// Wraps a native presentation root in full-window safe-area padding.
///
/// `root` is the synthetic host root. When its only child owns the page
/// background, that fill is copied onto the root so the padding paints behind
/// transparent Android system bars. `insets` are logical-pixel safe distances.
/// The returned element spans the viewport and retains the supplied subtree.
pub(super) fn native_host_root_with_safe_area(mut root: Element, insets: Insets) -> Element {
    if root.paint.quad.background.is_none() && root.children.len() == 1 {
        root.paint.quad.background = root.children[0].paint.quad.background.clone();
    }
    #[cfg(target_os = "android")]
    if let Some(Fill::Solid(color)) = &root.paint.quad.background {
        argui_platform::mobile::set_android_system_bar_color_scheme(
            color.preferred_contrast_scheme(),
        );
    }
    root.safe_area(insets)
        .width(percent(1.0))
        .height(percent(1.0))
}

impl Application {
    /// Installs decoded media before the native window and GPU renderer start.
    ///
    /// * `assets` — image and SVG resources addressed by the JavaScript asset manifest.
    pub(crate) fn install_native_host_assets(&mut self, assets: NativeHostAssets) {
        self.layout_engine
            .set_assets(&assets.images, &assets.vectors);
        self.image_assets = assets.images;
        self.vector_assets = assets.vectors;
    }

    /// Validates and applies `operations`, returning the resulting tree invalidation.
    ///
    /// The wire values are decoded here because layout dimensions cannot cross the
    /// event-loop thread boundary as Rust values. An invalid batch leaves the tree intact.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn commit_native_host(
        &mut self,
        operations: Vec<WireOperation>,
    ) -> Result<NativeHostCommit, String> {
        let operations = operations
            .into_iter()
            .map(WireOperation::into_native)
            .collect::<Result<Vec<_>, _>>()?;
        validate_native_host_assets(&operations, &self.image_assets, &self.vector_assets)?;
        let host = self
            .native_host
            .as_mut()
            .ok_or_else(|| "window has no native presentation host".to_string())?;
        let HostCommitResult { changed_nodes, .. } = host
            .commit(&operations)
            .map_err(|error| error.to_string())?;
        let root = host
            .root_element()
            .map(|root| native_host_root_with_safe_area(root, self.environment.safe_area_insets));
        let update = match (self.ui_tree.as_mut(), root) {
            (Some(tree), Some(root)) => tree.update(root),
            (None, Some(root)) => {
                let mut tree = UiTree::new(root);
                tree.set_pointer_settings(self.pointer_settings);
                tree.set_reduced_motion(self.environment.reduced_motion);
                self.ui_tree = Some(tree);
                TreeUpdate::Layout
            }
            (Some(_), None) => {
                self.ui_tree = None;
                self.ui_layout = None;
                TreeUpdate::Layout
            }
            (None, None) => TreeUpdate::None,
        };
        match update {
            TreeUpdate::Layout => self.pending_ui_frame.request_layout(),
            TreeUpdate::Scroll => self.pending_ui_frame.request_scroll_update(),
            TreeUpdate::Paint => self.pending_ui_frame.request_paint(),
            TreeUpdate::Composite => self.pending_ui_frame.request_composite(),
            TreeUpdate::Semantics => self.sync_accessibility(),
            TreeUpdate::None => {}
        }
        let animation_changed = self.sync_animations();
        if (self.pending_ui_frame.needs_frame() || animation_changed)
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
        Ok(NativeHostCommit {
            update,
            changed_nodes,
        })
    }
}
