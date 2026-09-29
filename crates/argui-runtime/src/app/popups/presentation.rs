//! Present the current popup scenes and recover failed native surfaces.

use super::Application;
use argui_platform::popup::PopupUnavailable;
use argui_render::{GpuCanvasDiagnosticKind, RenderStatus};
use std::sync::{Arc, atomic::Ordering};

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    /// Paints open popup scenes, registering their referenced assets before presentation.
    pub(in crate::app) fn render_popups(&mut self) {
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
            if popup
                .renderer
                .register_scene(
                    &surface.display_list,
                    &self.image_assets,
                    &self.vector_assets,
                )
                .is_err()
            {
                failed.push(popup.node);
                continue;
            }
            let window = popup.native.window();
            let size = window.inner_size();
            popup.renderer.surface.resize(size.width, size.height);
            let result = if self.composite_frame && popup.shown {
                popup.renderer.surface.render_composite_notified(
                    &surface.display_list,
                    self.scale_factor,
                    || window.pre_present_notify(),
                )
            } else {
                popup.renderer.surface.render_ui_notified(
                    &mut self.text_engine,
                    text,
                    &surface.display_list,
                    self.scale_factor,
                    || window.pre_present_notify(),
                )
            };
            for diagnostic in popup.renderer.surface.take_gpu_canvas_diagnostics() {
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
                        if popup.first_frame_ready.load(Ordering::Acquire) {
                            window.set_visible(true);
                            popup.shown = true;
                        } else if !popup.first_frame_queued {
                            popup.first_frame_queued = true;
                            let ready = Arc::clone(&popup.first_frame_ready);
                            let window = window.clone();
                            popup.renderer.surface.after_frame_ready(move || {
                                ready.store(true, Ordering::Release);
                                window.request_redraw();
                            });
                        }
                    }
                }
                Ok(RenderStatus::Skipped) => {}
                Ok(RenderStatus::Retry) => window.request_redraw(),
                Ok(RenderStatus::Reconfigure) => {
                    if size.width > 0 && size.height > 0 {
                        if !popup.renderer.surface.resize(size.width, size.height) {
                            popup.renderer.surface.reconfigure_surface();
                        }
                        window.request_redraw();
                    }
                }
                Ok(RenderStatus::RecreateSurface) => {
                    if popup
                        .renderer
                        .surface
                        .recreate_surface(window.clone())
                        .is_err()
                    {
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
}
