use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicBool};

use argui_core::Rect;
use argui_platform::popup::{NativePopup, PopupEnvironment, PopupKind, PopupUnavailable};
use argui_render::{SurfaceAlphaMode, SurfaceRenderer};
use argui_ui::{InteractionUpdate, NodeId, OverlaySurface, Role, UiEventKind};
use winit::window::WindowId;

use super::Application;

mod input;
mod presentation;
mod renderer;
use renderer::PopupRenderer;

pub(super) struct Popup {
    node: NodeId,
    pub(super) native: NativePopup,
    renderer: PopupRenderer,
    bounds: Rect,
    requested_bounds: Rect,
    environment: PopupEnvironment,
    pub(super) shown: bool,
    first_frame_queued: bool,
    first_frame_ready: Arc<AtomicBool>,
}

impl Popup {
    /// Registers or replaces `image` in an already-open native popup.
    ///
    /// # Errors
    /// Returns the underlying GPU upload error.
    pub(super) fn register_image(
        &mut self,
        image: &argui_paint::ImageAsset,
    ) -> Result<(), argui_render::RendererError> {
        self.renderer.register_image(image)
    }
    /// Registers `vector` in an already-open native popup renderer.
    /// Returns an error if GPU registration fails.
    ///
    /// # Errors
    /// Returns the underlying renderer error.
    pub(super) fn register_vector(
        &mut self,
        vector: &argui_paint::VectorAsset,
    ) -> Result<(), argui_render::RendererError> {
        self.renderer.register_vector(vector)
    }
}

#[derive(Default)]
pub(super) struct Popups {
    pub(super) entries: Vec<Popup>,
    spare: Option<PopupRenderer>,
    rejected: HashMap<NodeId, PopupUnavailable>,
    environment: Option<Result<PopupEnvironment, PopupUnavailable>>,
    focus_owner: Option<NodeId>,
    pub(super) suspended: bool,
    pub(super) pending_blur: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    /// Requests another frame from the top native popup, if one is open.
    ///
    /// Returns whether a popup received the redraw request.
    pub(crate) fn request_top_popup_redraw(&self) -> bool {
        let Some(popup) = self.popups.entries.last() else {
            return false;
        };
        popup.native.window().request_redraw();
        true
    }

    pub(crate) fn is_popup_window(&self, id: WindowId) -> bool {
        self.popups
            .entries
            .iter()
            .any(|popup| popup.native.window().id() == id)
    }

    pub(super) fn invalidate_popup_environment(&mut self) {
        self.popups.environment = None;
    }

    pub(super) fn sync_popups(&mut self, event_loop: &dyn crate::host::LoopControl) {
        let Some(host) = self.window.clone() else {
            return;
        };
        if self.popups.pending_blur {
            self.popups.pending_blur = false;
            let focused = host.winit().is_some_and(|window| window.has_focus())
                || self
                    .popups
                    .entries
                    .iter()
                    .any(|popup| popup.native.window().has_focus());
            if !focused {
                self.popups.suspended = true;
                self.popups.focus_owner = None;
                let nodes: Vec<_> = self
                    .popups
                    .entries
                    .iter()
                    .map(|popup| popup.node)
                    .rev()
                    .collect();
                for node in nodes {
                    self.dismiss_popup(node, event_loop);
                }
                self.window_focus(false, &host, event_loop);
                self.flush_ui_frame(event_loop);
            }
        }
        // One acceptance pass, followed by a bounded layout/placement correction for nested anchors.
        for _ in 0..2 {
            if !self.sync_popup_surfaces(event_loop) {
                break;
            }
            if !self.prepare_or_exit(event_loop) {
                return;
            }
        }
        self.sync_popup_focus();
        self.update_ime(&host);
    }

    fn sync_popup_surfaces(&mut self, event_loop: &dyn crate::host::LoopControl) -> bool {
        let (Some(ui), Some(layout), Some(host)) =
            (&mut self.ui_tree, &self.ui_layout, &self.window)
        else {
            return false;
        };
        let requested: Vec<_> = layout
            .portals
            .iter()
            .filter(|portal| {
                ui.portal_surface_preference(portal.node) == OverlaySurface::PreferNative
            })
            .map(|portal| portal.node)
            .collect();
        let mut changed = false;
        for index in (0..self.popups.entries.len()).rev() {
            if !requested.contains(&self.popups.entries[index].node) {
                let popup = self.popups.entries.remove(index);
                popup.native.window().set_visible(false);
                changed |= ui.set_native_portal(popup.node, None);
                // Keep one renderer, without retaining any logical popup or focus owner.
                self.popups.spare = Some(popup.renderer);
            }
        }
        self.popups
            .rejected
            .retain(|node, _| requested.contains(node));
        if requested.is_empty() {
            self.popups.environment = None;
            return changed;
        }
        let Some(parent) = host.winit() else {
            for node in requested {
                if self
                    .popups
                    .rejected
                    .insert(node, PopupUnavailable::UnsupportedBackend)
                    .is_none()
                {
                    (self.on_event)(crate::RuntimeEvent::PopupFallback {
                        node,
                        reason: PopupUnavailable::UnsupportedBackend.to_string(),
                    });
                }
            }
            return changed;
        };
        let ui_zoom = self.ui_zoom_factor;
        let environment = self
            .popups
            .environment
            .get_or_insert_with(|| {
                NativePopup::environment(parent).and_then(|environment| {
                    environment
                        .with_ui_zoom(ui_zoom)
                        .ok_or(PopupUnavailable::UnknownGeometry)
                })
            })
            .clone();
        for node in requested {
            if self.popups.rejected.contains_key(&node) {
                continue;
            }
            let environment = match &environment {
                Ok(environment) => *environment,
                Err(error) => {
                    (self.on_event)(crate::RuntimeEvent::PopupFallback {
                        node,
                        reason: error.to_string(),
                    });
                    self.popups.rejected.insert(node, error.clone());
                    continue;
                }
            };
            let Some(bounds) = layout.native_portal_placement(ui, node, environment.work_area)
            else {
                continue;
            };
            let bounds = environment.snap(bounds);
            if let Some(popup) = self
                .popups
                .entries
                .iter_mut()
                .find(|popup| popup.node == node)
            {
                if popup.requested_bounds != bounds || popup.environment != environment {
                    popup.native.reposition(environment, bounds);
                    popup.bounds = bounds;
                    popup.requested_bounds = bounds;
                    popup.environment = environment;
                    changed |= ui.set_native_portal(node, Some(bounds));
                }
                continue;
            }
            let parent_owner = ui
                .parent_of(node)
                .and_then(|node| ui.native_portal_owner(node));
            let native_parent = self
                .popups
                .entries
                .iter()
                .find(|popup| Some(popup.node) == parent_owner)
                .map_or(parent, |popup| popup.native.window().as_ref());
            let kind = match ui
                .element_for(node)
                .and_then(|element| element.semantics.as_ref())
                .map(|semantics| semantics.role)
            {
                Some(Role::Tooltip) => PopupKind::Tooltip,
                Some(Role::Menu | Role::ListBox) => PopupKind::Menu,
                _ => PopupKind::Popover,
            };
            let Some(device) = self.renderer_device.borrow().clone() else {
                continue;
            };
            let result: Result<Popup, PopupUnavailable> = (|| {
                let profile = self.renderer_config.profiling.then(std::time::Instant::now);
                let native = event_loop.popup(native_parent, kind, environment, bounds)?;
                let native_ms = profile.map_or(0.0, |start| start.elapsed().as_secs_f64() * 1000.0);
                let size = native.window().inner_size();
                let config = self
                    .renderer_config
                    .clone()
                    .surface_alpha(SurfaceAlphaMode::Transparent);
                let reused = self.popups.spare.is_some();
                let mut renderer = if let Some(mut renderer) = self.popups.spare.take() {
                    renderer
                        .surface
                        .recreate_surface(native.window().clone())
                        .map_err(|error| PopupUnavailable::Platform(error.to_string()))?;
                    renderer.surface.resize(size.width, size.height);
                    renderer
                } else {
                    PopupRenderer::new(
                        pollster::block_on(SurfaceRenderer::new_with_device(
                            native.window().clone(),
                            size.width,
                            size.height,
                            config,
                            device,
                        ))
                        .map_err(|error| PopupUnavailable::Platform(error.to_string()))?,
                    )
                };
                renderer
                    .surface
                    .set_damage_tracking(self.renderer_config.damage_tracking);
                if let Some(start) = profile {
                    eprintln!(
                        "argui-popup-profile reused={reused} native_ms={native_ms:.3} renderer_ms={:.3}",
                        start.elapsed().as_secs_f64() * 1000.0 - native_ms
                    );
                }
                Ok(Popup {
                    node,
                    native,
                    renderer,
                    bounds,
                    requested_bounds: bounds,
                    environment,
                    shown: false,
                    first_frame_queued: false,
                    first_frame_ready: Arc::new(AtomicBool::new(false)),
                })
            })();
            match result {
                Ok(popup) => {
                    changed |= ui.set_native_portal(node, Some(bounds));
                    self.popups.entries.push(popup);
                }
                Err(error) => {
                    (self.on_event)(crate::RuntimeEvent::PopupFallback {
                        node,
                        reason: error.to_string(),
                    });
                    self.popups.rejected.insert(node, error);
                }
            }
        }
        changed
    }

    fn sync_popup_focus(&mut self) {
        if self.popups.suspended {
            return;
        }
        let owner = self.ui_tree.as_ref().and_then(|ui| {
            ui.focused_node()
                .and_then(|node| ui.native_portal_owner(node))
        });
        if owner == self.popups.focus_owner {
            return;
        }
        if let Some(popup) = self
            .popups
            .entries
            .iter()
            .find(|popup| Some(popup.node) == owner)
        {
            if !popup.shown {
                return;
            }
            popup.native.focus();
        } else if self.popups.focus_owner.is_some()
            && let Some(window) = &self.window
        {
            window.focus_window();
        }
        self.popups.focus_owner = owner;
    }

    fn dismiss_popup(&mut self, node: NodeId, event_loop: &dyn crate::host::LoopControl) {
        if let (Some(ui), Some(window)) = (&mut self.ui_tree, self.window.clone()) {
            let update = InteractionUpdate {
                events: ui.event_deliveries(node, UiEventKind::DismissRequested),
                ..InteractionUpdate::default()
            };
            self.apply_ui_update(update, &window, event_loop);
        }
    }
}

impl Popups {
    /// Replaces the complete effect registry for every open native popup.
    ///
    /// `registry` is the validated next generation. Returns an error if any
    /// popup renderer cannot prepare its WGSL pipelines.
    pub(super) fn replace_effect_registry(
        &mut self,
        registry: &argui_render::EffectRegistry,
    ) -> Result<(), argui_render::RendererError> {
        for popup in &mut self.entries {
            popup
                .renderer
                .surface
                .replace_effect_registry(registry.clone())?;
        }
        if let Some(renderer) = &mut self.spare {
            renderer.surface.replace_effect_registry(registry.clone())?;
        }
        Ok(())
    }

    /// Changes damage tracking for every currently open native popup.
    ///
    /// `tracking` is also inherited by popups created after this call through
    /// the owning application's renderer configuration.
    pub(super) fn set_damage_tracking(&mut self, tracking: argui_render::DamageTracking) {
        for popup in &mut self.entries {
            popup.renderer.surface.set_damage_tracking(tracking);
        }
        if let Some(renderer) = &mut self.spare {
            renderer.surface.set_damage_tracking(tracking);
        }
    }
}

impl Drop for Popups {
    fn drop(&mut self) {
        while self.entries.pop().is_some() {}
    }
}
