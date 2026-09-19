use std::collections::HashMap;

use argui_core::Rect;
use argui_platform::popup::{NativePopup, PopupEnvironment, PopupKind, PopupUnavailable};
use argui_render::{GpuCanvasDiagnosticKind, RenderStatus, SurfaceRenderer};
use argui_ui::{InteractionUpdate, NodeId, OverlaySurface, Role, UiEventKind};
use winit::window::WindowId;

use super::Application;

mod input;

pub(super) struct Popup {
    node: NodeId,
    pub(super) native: NativePopup,
    renderer: SurfaceRenderer,
    bounds: Rect,
    requested_bounds: Rect,
    environment: PopupEnvironment,
    pub(super) shown: bool,
}

#[derive(Default)]
pub(super) struct Popups {
    pub(super) entries: Vec<Popup>,
    rejected: HashMap<NodeId, PopupUnavailable>,
    environment: Option<Result<PopupEnvironment, PopupUnavailable>>,
    focus_owner: Option<NodeId>,
    pub(super) suspended: bool,
    pub(super) pending_blur: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
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
                changed |= ui.set_native_portal(popup.node, None);
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
        let environment = self
            .popups
            .environment
            .get_or_insert_with(|| NativePopup::environment(parent))
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
                let native = event_loop.popup(native_parent, kind, environment, bounds)?;
                let size = native.window().inner_size();
                let mut config = self.renderer_config.clone();
                config.surface_alpha = argui_render::SurfaceAlphaMode::Opaque;
                if let Some(argui_paint::Fill::Solid(color)) = ui
                    .element_for(node)
                    .and_then(|element| element.paint.quad.background.as_ref())
                {
                    config.clear_color = color.with_alpha(1.0);
                }
                let mut renderer = pollster::block_on(SurfaceRenderer::new_with_device(
                    native.window().clone(),
                    size.width,
                    size.height,
                    config,
                    device,
                ))
                .map_err(|error| PopupUnavailable::Platform(error.to_string()))?;
                for image in &self.image_assets {
                    renderer
                        .register_image(image)
                        .map_err(|error| PopupUnavailable::Platform(error.to_string()))?;
                }
                for vector in &self.vector_assets {
                    renderer
                        .register_vector(vector)
                        .map_err(|error| PopupUnavailable::Platform(error.to_string()))?;
                }
                Ok(Popup {
                    node,
                    native,
                    renderer,
                    bounds,
                    requested_bounds: bounds,
                    environment,
                    shown: false,
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

    pub(super) fn render_popups(&mut self) {
        let (Some(layout), Some(text)) = (&self.ui_layout, &self.prepared_text) else {
            return;
        };
        let mut failed = Vec::new();
        for popup in &mut self.popups.entries {
            let Some(surface) = layout
                .native_surfaces
                .iter()
                .find(|surface| surface.node == popup.node)
            else {
                continue;
            };
            let window = popup.native.window();
            let size = window.inner_size();
            popup.renderer.resize(size.width, size.height);
            let result = if self.composite_frame && popup.shown {
                popup.renderer.render_composite_notified(
                    &surface.display_list,
                    self.scale_factor,
                    || window.pre_present_notify(),
                )
            } else {
                popup.renderer.render_ui_notified(
                    &mut self.text_engine,
                    text,
                    &surface.display_list,
                    self.scale_factor,
                    || window.pre_present_notify(),
                )
            };
            for diagnostic in popup.renderer.take_gpu_canvas_diagnostics() {
                let event = match diagnostic.kind {
                    GpuCanvasDiagnosticKind::Failed => {
                        crate::RuntimeEvent::GpuCanvasFailed(diagnostic)
                    }
                    GpuCanvasDiagnosticKind::Recovered => {
                        crate::RuntimeEvent::GpuCanvasRecovered(diagnostic)
                    }
                };
                (self.on_event)(event);
            }
            match result {
                Ok(RenderStatus::Presented) => {
                    if !popup.shown {
                        window.set_visible(true);
                        popup.shown = true;
                    }
                }
                Ok(RenderStatus::Skipped) => {}
                Ok(RenderStatus::Reconfigure) => {
                    popup.renderer.resize(size.width, size.height);
                    window.request_redraw();
                }
                Ok(RenderStatus::RecreateSurface) => {
                    if popup.renderer.recreate_surface(window.clone()).is_err() {
                        failed.push(popup.node);
                    } else {
                        window.request_redraw();
                    }
                }
                Err(_) => failed.push(popup.node),
            }
        }
        if !failed.is_empty() {
            // A lost parent surface invalidates all of its OS children. Retire children first.
            for index in (0..self.popups.entries.len()).rev() {
                let node = self.popups.entries[index].node;
                let mut ancestor = Some(node);
                let mut affected = false;
                while let Some(current) = ancestor {
                    affected |= failed.contains(&current);
                    ancestor = self.ui_tree.as_ref().and_then(|ui| ui.parent_of(current));
                }
                if affected {
                    self.popups.entries.remove(index);
                    let reason =
                        PopupUnavailable::Platform("native GPU surface unavailable".into());
                    (self.on_event)(crate::RuntimeEvent::PopupFallback {
                        node,
                        reason: reason.to_string(),
                    });
                    self.popups.rejected.insert(node, reason);
                    if let Some(ui) = &mut self.ui_tree {
                        ui.set_native_portal(node, None);
                    }
                }
            }
            self.pending_ui_frame.request_layout();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        self.sync_popup_focus();
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
    /// Changes damage tracking for every currently open native popup.
    ///
    /// `tracking` is also inherited by popups created after this call through
    /// the owning application's renderer configuration.
    pub(super) fn set_damage_tracking(&mut self, tracking: argui_render::DamageTracking) {
        for popup in &mut self.entries {
            popup.renderer.set_damage_tracking(tracking);
        }
    }
}

impl Drop for Popups {
    fn drop(&mut self) {
        while self.entries.pop().is_some() {}
    }
}
