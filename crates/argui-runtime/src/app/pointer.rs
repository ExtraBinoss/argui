use argui_core::{Point, PointerEvent, PointerPhase};
use argui_platform::{ButtonState, ScrollDelta};
use argui_ui::{InteractionUpdate, UiTree};
use winit::{event_loop::ActiveEventLoop, window::Window};

use super::{Application, local_point};

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn flush_gesture_frame(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let update = self
            .ui_tree
            .as_mut()
            .map_or_else(InteractionUpdate::default, UiTree::flush_gesture_frame);
        if !update.events.is_empty() {
            self.apply_ui_update(update, window, event_loop);
        }
    }

    pub(super) fn flush_scrollbar_drag(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let Some(point) = self.pending_scrollbar_drag.take() else {
            return;
        };
        let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) else {
            return;
        };
        if let Some(update) = ui.scrollbar_dragged(point, &layout.scroll_regions) {
            self.apply_ui_update(update, window, event_loop);
        }
    }

    pub(super) fn pointer_left(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        self.scroll_inertia.cancel();
        self.flush_pointer_scroll(window, event_loop);
        self.flush_scrollbar_drag(window, event_loop);
        let point = self.pointer.unwrap_or_default();
        self.pointer = None;
        self.refresh_cursor(window);
        if let Some(ui) = &mut self.ui_tree {
            let mut update = ui.scrollbar_pointer_moved(None, &[]);
            update.merge(ui.pointer_event(
                PointerEvent {
                    buttons: self.pointer_buttons,
                    modifiers: self.modifiers,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Left, point)
                },
                &[],
            ));
            self.apply_ui_update(update, window, event_loop);
        }
    }

    pub(super) fn primary_button(
        &mut self,
        state: ButtonState,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        self.scroll_inertia.cancel();
        self.flush_pointer_scroll(window, event_loop);
        if state == ButtonState::Released {
            self.flush_scrollbar_drag(window, event_loop);
        }
        if state == ButtonState::Pressed && self.handle_window_drag(window) {
            return;
        }
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let placement = self.pointer.and_then(|point| {
            let target = layout
                .hit_regions
                .iter()
                .rev()
                .find(|region| region.contains(point))?
                .node;
            layout.text_inputs.iter().rev().find_map(|region| {
                if region.node != target {
                    return None;
                }
                let point = local_point(layout, region.node, point).unwrap_or(point);
                region
                    .hit_position(point)
                    .map(|position| (region.node, position))
            })
        });
        let static_placement = self
            .pointer
            .and_then(|point| Self::static_text_at(layout, point));
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        if state == ButtonState::Pressed
            && let Some(point) = self.pointer
            && let Some(region) =
                argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions)
            && let Some(update) = ui.scrollbar_pressed(point, std::slice::from_ref(region))
        {
            self.programmatic_scroll = None;
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if state == ButtonState::Released
            && let Some(mut update) = ui.scrollbar_released()
        {
            let scrollbar = self.pointer.and_then(|point| {
                argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions)
            });
            update.merge(ui.scrollbar_pointer_moved(
                self.pointer,
                scrollbar.map_or(&[], std::slice::from_ref),
            ));
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if state == ButtonState::Released {
            ui.release_text_cursor();
        }
        let selecting = ui.document_selection_dragging();
        let phase = match state {
            ButtonState::Pressed => PointerPhase::Pressed,
            ButtonState::Released if selecting && ui.has_document_selection() => {
                PointerPhase::Cancelled
            }
            ButtonState::Released => PointerPhase::Released,
        };
        let point = self.pointer.unwrap_or_default();
        let update = ui.pointer_event(
            PointerEvent {
                button: Some(argui_core::PointerButton::Primary),
                buttons: self.pointer_buttons,
                modifiers: self.modifiers,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(phase, point)
            },
            &layout.hit_regions,
        );
        let pointer_default = update
            .events
            .iter()
            .find(|event| {
                matches!(
                    event.kind,
                    argui_ui::UiEventKind::Pointer(argui_core::PointerEvent {
                        phase: PointerPhase::Pressed,
                        ..
                    })
                )
            })
            .cloned();
        self.apply_ui_update(update, window, event_loop);
        let default_prevented = pointer_default.is_some_and(|event| event.default_prevented());
        if state == ButtonState::Pressed {
            if !default_prevented
                && let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout)
            {
                let focus_update =
                    ui.focus_pointer_default(argui_core::PointerId::MOUSE, &layout.hit_regions);
                self.apply_ui_update(focus_update, window, event_loop);
            }
            let selection_update = if default_prevented {
                InteractionUpdate::default()
            } else if let Some(position) = static_placement {
                let granularity = self.selection_click.next(point, self.pointer_settings);
                self.ui_tree
                    .as_mut()
                    .map_or_else(InteractionUpdate::default, |ui| {
                        ui.begin_document_selection(position, self.modifiers.shift, granularity)
                    })
            } else if placement.is_none() {
                self.ui_tree
                    .as_mut()
                    .map_or_else(InteractionUpdate::default, UiTree::clear_document_selection)
            } else {
                InteractionUpdate::default()
            };
            self.apply_ui_update(selection_update, window, event_loop);
        } else {
            let update = self.ui_tree.as_mut().map_or_else(
                InteractionUpdate::default,
                UiTree::release_document_selection,
            );
            self.apply_ui_update(update, window, event_loop);
        }
        if state == ButtonState::Pressed
            && let Some((node, position)) = placement
            && let Some(ui) = &mut self.ui_tree
        {
            let update = ui.place_text_position(node, position, self.modifiers.shift);
            self.apply_ui_update(update, window, event_loop);
        }
        self.update_ime(window);
    }

    pub(super) fn secondary_button(
        &mut self,
        state: ButtonState,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        if state != ButtonState::Pressed {
            return;
        }
        let point = self.pointer.unwrap_or_default();
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let static_position = Self::static_text_at(layout, point);
        let static_target = static_position.map(|position| position.node);
        let hit_target = layout
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.contains(point))
            .map(|region| region.node);
        let Some(target) = static_target.or(hit_target) else {
            return;
        };
        if let Some(position) = static_position {
            let selection_update =
                self.ui_tree
                    .as_mut()
                    .map_or_else(InteractionUpdate::default, |ui| {
                        if ui.document_selection_contains(position) {
                            InteractionUpdate::default()
                        } else {
                            let mut update = ui.begin_document_selection(
                                position,
                                false,
                                argui_ui::SelectionGranularity::Word,
                            );
                            update.merge(ui.release_document_selection());
                            update
                        }
                    });
            self.apply_ui_update(selection_update, window, event_loop);
        }
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let capabilities = ui.selection_capabilities(target);
        let events = ui.event_deliveries(
            target,
            argui_ui::UiEventKind::ContextMenu {
                position: point,
                capabilities,
            },
        );
        self.apply_ui_update(
            InteractionUpdate {
                events,
                ..InteractionUpdate::default()
            },
            window,
            event_loop,
        );
    }

    pub(super) fn pointer_moved(
        &mut self,
        point: Point,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        self.pointer = Some(point);
        self.refresh_cursor(window);
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        if ui.scrollbar_dragging() {
            self.pending_scrollbar_drag = Some(point);
            window.request_redraw();
            return;
        }
        if ui.text_cursor_dragging()
            && let Some(node) = ui.focused_node()
            && let Some(region) = layout.text_inputs.iter().find(|region| region.node == node)
        {
            let local = local_point(layout, node, point).unwrap_or(point);
            let update = ui.drag_text_position(node, region.closest_position(local));
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if ui.document_selection_dragging()
            && let Some(position) = Self::closest_static_text(layout, point)
        {
            let update = ui.drag_document_selection(position);
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        let scrollbar = argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions);
        let mut update =
            ui.scrollbar_pointer_moved(Some(point), scrollbar.map_or(&[], std::slice::from_ref));
        let hit_regions = if scrollbar.is_some() {
            &[]
        } else {
            layout.hit_regions.as_slice()
        };
        update.merge(ui.pointer_event(
            PointerEvent {
                buttons: self.pointer_buttons,
                modifiers: self.modifiers,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(PointerPhase::Moved, point)
            },
            hit_regions,
        ));
        self.apply_ui_update(update, window, event_loop);
    }

    pub(super) fn touch_pointer(
        &mut self,
        mut event: PointerEvent,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) -> PointerEvent {
        if event.phase == PointerPhase::Pressed && self.primary_touch.is_none() {
            self.primary_touch = Some(event.id);
        }
        event.primary = self.primary_touch == Some(event.id);
        match event.phase {
            PointerPhase::Pressed if event.primary => {
                self.begin_touch_selection(event.id, event.position, window);
            }
            PointerPhase::Moved => self.move_touch_selection_candidate(event.id, event.position),
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                self.cancel_touch_selection(event.id);
            }
            PointerPhase::Entered | PointerPhase::Pressed => {}
        }
        let finger_delta = match event.phase {
            PointerPhase::Pressed => {
                self.touch_points.insert(event.id, event.position);
                None
            }
            PointerPhase::Moved => self
                .touch_points
                .insert(event.id, event.position)
                .map(|previous| {
                    Point::new(event.position.x - previous.x, event.position.y - previous.y)
                })
                .filter(|_| event.primary),
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                self.touch_points.remove(&event.id);
                None
            }
            PointerPhase::Entered => None,
        };
        let mut touch_scroll = None;
        let update = if let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) {
            let selecting = ui.document_selection_dragging() && event.primary;
            let mut update = if selecting && event.phase == PointerPhase::Moved {
                Self::closest_static_text(layout, event.position)
                    .map_or_else(InteractionUpdate::default, |position| {
                        ui.drag_document_selection(position)
                    })
            } else {
                ui.pointer_event(event, &layout.hit_regions)
            };
            let default_prevented = update.events.iter().any(|event| event.default_prevented());
            if let Some(delta) = finger_delta.filter(|_| !selecting && !default_prevented) {
                self.programmatic_scroll = None;
                touch_scroll = Some((Some(delta), event.phase));
                update.merge(ui.scroll(
                    event.position,
                    ScrollDelta::Pixels(delta),
                    &layout.scroll_regions,
                ));
            } else if event.primary {
                let phase = if selecting || default_prevented {
                    PointerPhase::Cancelled
                } else {
                    event.phase
                };
                touch_scroll = Some((None, phase));
            }
            Some(update)
        } else {
            None
        };
        if let Some((delta, phase)) = touch_scroll {
            self.observe_touch_scroll(event.position, delta, phase, window);
        }
        if let Some(update) = update {
            self.apply_ui_update(update, window, event_loop);
        }
        if matches!(event.phase, PointerPhase::Pressed | PointerPhase::Released) {
            self.update_ime(window);
        }
        if matches!(
            event.phase,
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left
        ) && event.primary
        {
            let update = self.ui_tree.as_mut().map_or_else(
                InteractionUpdate::default,
                UiTree::release_document_selection,
            );
            self.apply_ui_update(update, window, event_loop);
            self.primary_touch = self.touch_points.keys().min_by_key(|id| id.get()).copied();
        }
        event
    }
}
