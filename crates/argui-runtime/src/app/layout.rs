use super::Application;
use crate::{LayoutBounds, LayoutSnapshot, RuntimeError, RuntimeEvent, ViewUpdate};
use argui_core::Size;
use argui_layout::LayoutOutput;

impl Application {
    fn prepare_text(
        &mut self,
        event_loop: &dyn crate::host::LoopControl,
    ) -> Result<(), RuntimeError> {
        if let Some(mut layout) = self.compute_ui_layout()? {
            let refresh = layout.virtualization_changed;
            let Some(next) = self.refresh_layout(layout, refresh, true)? else {
                return Ok(());
            };
            layout = next;
            let snapshot = LayoutSnapshot {
                viewport: layout.viewport,
                nodes: layout
                    .nodes
                    .iter()
                    .map(|node| {
                        let element = self
                            .ui_tree
                            .as_ref()
                            .and_then(|ui| ui.element_for(node.node));
                        LayoutBounds {
                            node: node.node,
                            key: element.and_then(|element| element.key.clone()),
                            retained_identity: element
                                .and_then(|element| element.source_identity().cloned()),
                            bounds: node.bounds,
                        }
                    })
                    .collect(),
            };
            let layout_effects = self.model.as_ref().map(|model| {
                model.layout_changed(&snapshot);
                model.take_effects()
            });
            let mut rebuild = false;
            let mut scroll_request = None;
            if let Some(mut effects) = layout_effects {
                rebuild = effects.update == ViewUpdate::Rebuild;
                effects.update = ViewUpdate::None;
                scroll_request = effects.scroll.take();
                self.apply_model_effects(effects, event_loop);
            }
            let Some(next) = self.refresh_layout(layout, rebuild, false)? else {
                return Ok(());
            };
            layout = next;
            if rebuild {
                let refresh = layout.virtualization_changed;
                let Some(next) = self.refresh_layout(layout, refresh, true)? else {
                    return Ok(());
                };
                layout = next;
            }
            self.publish_virtual_events(std::mem::take(&mut layout.virtual_events));
            self.ui_layout = Some(layout);
            if let Some(request) = scroll_request {
                self.prepared_text = None;
                self.apply_scroll_request(request, event_loop);
            }
            self.publish_inspection();
            self.paint_inspection_highlight();
        }
        let scene = self
            .ui_layout
            .as_ref()
            .map(|layout| &layout.text)
            .or(self.text_scene.as_ref());
        self.prepared_text = scene.map(|scene| self.text_engine.prepare(scene, self.scale_factor));
        Ok(())
    }

    fn compute_ui_layout(&mut self) -> Result<Option<LayoutOutput>, RuntimeError> {
        let Some(ui) = &mut self.ui_tree else {
            return Ok(None);
        };
        self.layout_engine
            .compute(ui, &mut self.text_engine, self.viewport)
            .map(Some)
            .map_err(Into::into)
    }

    fn refresh_layout(
        &mut self,
        layout: LayoutOutput,
        refresh: bool,
        request_if_unsettled: bool,
    ) -> Result<Option<LayoutOutput>, RuntimeError> {
        if !refresh {
            return Ok(Some(layout));
        }
        if let Some(root) = self.inspected_view() {
            self.ui_tree
                .as_mut()
                .expect("a computed layout retains its UI tree")
                .update(root);
        }
        let Some(mut next) = self.compute_ui_layout()? else {
            return Ok(None);
        };
        let mut prior_events = layout.virtual_events;
        prior_events.append(&mut next.virtual_events);
        next.virtual_events = prior_events;
        if request_if_unsettled && next.virtualization_changed {
            self.pending_ui_frame.request_rebuild();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        Ok(Some(next))
    }

    /// Publishes layout-driven virtual-list events to native and application listeners.
    ///
    /// * `events` — measured extents and bounded-window requests from this layout.
    fn publish_virtual_events(&mut self, events: Vec<argui_ui::UiEvent>) {
        for event in events {
            if !event.should_dispatch() {
                continue;
            }
            self.deliver_native_host_event(&event);
            let published = self
                .ui_tree
                .as_ref()
                .map_or_else(|| event.clone(), |ui| ui.inspect_event(&event));
            (self.on_event)(RuntimeEvent::Ui(published));
        }
    }

    pub(super) fn update_viewport(&mut self, width: u32, height: u32) {
        self.viewport = Size::new(
            width as f32 / self.scale_factor,
            height as f32 / self.scale_factor,
        );
    }

    pub(super) fn prepare_or_exit(&mut self, event_loop: &dyn crate::host::LoopControl) -> bool {
        if let Err(error) = self.prepare_text(event_loop) {
            (self.on_event)(RuntimeEvent::LayoutFailed(error.to_string()));
            self.fatal_error = Some(error);
            event_loop.exit();
            return false;
        }
        true
    }
}
