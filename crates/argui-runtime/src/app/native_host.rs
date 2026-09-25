//! Native presentation commits applied on the UI thread.

use argui_core::Insets;
#[cfg(target_os = "android")]
use argui_paint::Fill;
use argui_paint::VectorAsset;
use argui_ui::{Element, TreeUpdate, UiEvent, UiTree, percent};

use super::Application;
use crate::{
    HostCommitResult, NativeHostAssets, NativeHostCommit, NativeHostControl, WireOperation,
    validate_native_host_assets, validate_native_host_canvases,
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
    /// Delivers `event` to the current presentation callback, if one is installed.
    /// Native hosts use a channel and browser hosts call their subscriber directly.
    pub(super) fn deliver_native_host_event(&self, event: &UiEvent) {
        let Some(callback) = self
            .native_host
            .as_ref()
            .and_then(|host| host.callback_for(event))
        else {
            return;
        };
        let pointer = self
            .ui_layout
            .as_ref()
            .and_then(|layout| crate::NativePointerPosition::from_event(event, layout));
        let delivery = crate::NativeHostDelivery {
            callback,
            kind: event.kind.clone(),
            pointer,
        };
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(sender) = &self.native_host_events {
            let _ = sender.send(delivery);
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(subscriber) = &self.web_host_events {
            subscriber(delivery);
        }
    }

    /// Installs decoded media before the native window and GPU renderer start.
    ///
    /// * `assets` — image and SVG resources addressed by the JavaScript asset manifest.
    pub(crate) fn install_native_host_assets(&mut self, assets: NativeHostAssets) {
        self.layout_engine
            .set_assets(&assets.images, &assets.vectors);
        self.image_assets = assets.images;
        self.vector_assets = assets.vectors;
    }

    /// Validates and applies `operations`, then applies `controls`, returning the
    /// resulting tree invalidation.
    ///
    /// The wire values are decoded here because layout dimensions cannot cross the
    /// event-loop thread boundary as Rust values. An invalid batch leaves the tree intact.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn commit_native_host(
        &mut self,
        operations: Vec<WireOperation>,
        controls: Vec<NativeHostControl>,
    ) -> Result<NativeHostCommit, String> {
        if operations.is_empty() {
            for control in controls {
                self.apply_native_host_control(control)?;
            }
            return Ok(NativeHostCommit {
                update: TreeUpdate::None,
                changed_nodes: 0,
            });
        }
        let operations = operations
            .into_iter()
            .map(WireOperation::into_native)
            .collect::<Result<Vec<_>, _>>()?;
        let mut next_vectors = self.vector_assets.clone();
        for control in &controls {
            if let NativeHostControl::RegisterVector(vector) = control {
                if let Some(existing) = next_vectors.iter().find(|asset| asset.id == vector.id) {
                    if existing != vector {
                        return Err(format!("conflicting native vector id {:?}", vector.id));
                    }
                } else {
                    next_vectors.push(vector.clone());
                }
            }
        }
        validate_native_host_assets(&operations, &self.image_assets, &next_vectors)?;
        validate_native_host_canvases(&operations, &self.renderer_config.gpu_canvases)?;
        let replacement = controls.iter().find_map(|control| match control {
            NativeHostControl::ReplaceEffects(effects) => Some(effects.clone()),
            _ => None,
        });
        if replacement.is_some() {
            self.native_host
                .as_ref()
                .ok_or_else(|| "window has no native presentation host".to_string())?
                .validate(&operations)
                .map_err(|error| error.to_string())?;
        }
        let previous_effects = replacement.as_ref().map(|_| self.base_effects.clone());
        for vector in next_vectors.iter().skip(self.vector_assets.len()) {
            self.register_native_vector(vector.clone())?;
        }
        if let Some(effects) = replacement {
            self.replace_native_effects(effects)
                .map_err(|error| error.to_string())?;
        }
        let committed = self
            .native_host
            .as_mut()
            .ok_or_else(|| "window has no native presentation host".to_string())?
            .commit(&operations);
        let HostCommitResult { changed_nodes, .. } = match committed {
            Ok(commit) => commit,
            Err(error) => {
                if let Some(previous) = previous_effects {
                    self.replace_native_effects(previous).map_err(|rollback| {
                        format!("{error}; effect rollback failed: {rollback}")
                    })?;
                }
                return Err(error.to_string());
            }
        };
        let root = self
            .native_host
            .as_ref()
            .and_then(|host| host.root_element())
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
            #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
            if let Some(popup) = self.popups.entries.last() {
                popup.native.window().request_redraw();
            } else {
                window.request_redraw();
            }
            #[cfg(not(all(feature = "native-popups", not(target_arch = "wasm32"))))]
            window.request_redraw();
        }
        for control in controls {
            if !matches!(
                control,
                NativeHostControl::ReplaceEffects(_) | NativeHostControl::RegisterVector(_)
            ) {
                self.apply_native_host_control(control)?;
            }
        }
        Ok(NativeHostCommit {
            update,
            changed_nodes,
        })
    }

    /// Registers a validated SVG in the active renderer and layout engine.
    /// `vector` supplies its stable ID and source. Returns an error if an ID
    /// names different bytes or GPU registration fails; equal repeats do no work.
    ///
    /// # Errors
    /// Returns a conflict or renderer registration error.
    fn register_native_vector(&mut self, vector: VectorAsset) -> Result<(), String> {
        if let Some(existing) = self
            .vector_assets
            .iter()
            .find(|asset| asset.id == vector.id)
        {
            return if existing == &vector {
                Ok(())
            } else {
                Err(format!("conflicting native vector id {:?}", vector.id))
            };
        }
        if let super::RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
            renderer
                .register_vector(&vector)
                .map_err(|error| error.to_string())?;
        }
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        for popup in &mut self.popups.entries {
            popup
                .register_vector(&vector)
                .map_err(|error| error.to_string())?;
        }
        self.vector_assets.push(vector);
        self.layout_engine
            .set_assets(&self.image_assets, &self.vector_assets);
        Ok(())
    }

    /// Applies `control` to the active renderer and requests any needed redraw.
    /// Applies one renderer `control` and returns an error if GPU preparation fails.
    fn apply_native_host_control(&mut self, control: NativeHostControl) -> Result<(), String> {
        match control {
            NativeHostControl::SetDamageTracking(tracking) => self.set_damage_tracking(tracking),
            NativeHostControl::SetRendererProfiling(enabled) => {
                self.set_renderer_profiling(enabled)
            }
            NativeHostControl::ReplaceEffects(effects) => self
                .replace_native_effects(effects)
                .map_err(|error| error.to_string())?,
            NativeHostControl::RegisterVector(vector) => self.register_native_vector(vector)?,
        }
        Ok(())
    }
}
