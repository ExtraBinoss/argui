use argui_core::{PinchUpdate, Point, PointerEvent, PointerPhase};
use argui_platform::ButtonState;
use argui_ui::{InteractionUpdate, UiTree};

use super::Application;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn flush_gesture_frame(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let update = self
            .ui_tree
            .as_mut()
            .map_or_else(InteractionUpdate::default, UiTree::flush_gesture_frame);
        if !update.events.is_empty() {
            self.apply_ui_update(update, window, event_loop);
        }
    }

    pub(super) fn flush_scrollbar_drag(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
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

    pub(super) fn pointer_left(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        self.scroll_inertia.cancel();
        self.flush_pointer_scroll(window, event_loop);
        self.flush_scrollbar_drag(window, event_loop);
        let point = self.pointer.unwrap_or_default();
        self.pointer = None;
        self.mouse_selection_origin = None;
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
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
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
                let point = layout.local_point(region.node, point).unwrap_or(point);
                region
                    .hit_position(point)
                    .map(|position| (region.node, position))
            })
        });
        let static_placement = self
            .pointer
            .and_then(|point| Self::static_text_at(layout, point));
        let hit_target = self.pointer.and_then(|point| {
            layout
                .hit_regions
                .iter()
                .rev()
                .find(|region| region.contains(point))
                .map(|region| region.node)
        });
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let hit_blocks_selection = hit_target
            .is_some_and(|target| ui.resolved_user_select(target) == argui_ui::UserSelect::None);
        let static_placement = static_placement.filter(|_| !hit_blocks_selection);
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
        if state == ButtonState::Pressed {
            let capture_update = match (&mut self.ui_tree, &self.ui_layout) {
                (Some(ui), Some(layout)) => ui.pointer_press_default(
                    argui_core::PointerId::MOUSE,
                    pointer_default.as_ref(),
                    &layout.hit_regions,
                ),
                _ => InteractionUpdate::default(),
            };
            self.apply_ui_update(capture_update, window, event_loop);
        }
        let default_prevented = pointer_default.is_some_and(|event| event.default_prevented());
        let granularity = if state == ButtonState::Pressed
            && !default_prevented
            && (placement.is_some() || static_placement.is_some())
        {
            self.selection_click.next(point, self.pointer_settings)
        } else {
            argui_ui::SelectionGranularity::Character
        };
        if state == ButtonState::Pressed {
            self.mouse_selection_origin = (!default_prevented
                && placement.is_none()
                && static_placement.is_none()
                && !hit_blocks_selection)
                .then_some(point);
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
            self.mouse_selection_origin = None;
            let update = self.ui_tree.as_mut().map_or_else(
                InteractionUpdate::default,
                UiTree::release_document_selection,
            );
            self.apply_ui_update(update, window, event_loop);
        }
        if state == ButtonState::Pressed
            && !default_prevented
            && let Some((node, position)) = placement
            && let Some(ui) = &mut self.ui_tree
        {
            let update = ui.begin_text_selection(node, position, self.modifiers.shift, granularity);
            self.apply_ui_update(update, window, event_loop);
        }
        self.update_ime(window);
    }

    pub(super) fn secondary_button(
        &mut self,
        state: ButtonState,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
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
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
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
            let local = layout.local_point(node, point).unwrap_or(point);
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
        if let Some(origin) = self.mouse_selection_origin
            && let Some(anchor) = Self::closest_static_text(layout, origin)
            && let Some(focus) = Self::closest_static_text(layout, point)
        {
            self.mouse_selection_origin = None;
            let mut update = ui.begin_document_selection(
                anchor,
                false,
                argui_ui::SelectionGranularity::Character,
            );
            update.merge(ui.drag_document_selection(focus));
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

    /// Dispatches one touch contact after applying global zoom gesture ownership.
    ///
    /// `event` is expressed in UI coordinates, `zoom` describes whether the
    /// runtime-owned pinch consumes it, and `window`/`event_loop` receive any
    /// resulting UI work. Returns the normalized event for platform observers.
    pub(super) fn touch_pointer(
        &mut self,
        mut event: PointerEvent,
        zoom: PinchUpdate,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) -> PointerEvent {
        if event.phase == PointerPhase::Pressed && self.primary_touch.is_none() {
            self.primary_touch = Some(event.id);
            self.last_touch = Some(event.id);
        }
        event.primary = self.primary_touch == Some(event.id);
        if zoom.started {
            self.scroll_inertia.cancel();
            self.touch_scroll.cancel();
            self.pending_pointer_scroll = None;
            self.programmatic_scroll = None;
            self.scroll_gesture = argui_ui::ScrollGesture::default();
            self.touch_selection = None;
            self.touch_selection_handle = None;
            if let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) {
                let mut update = InteractionUpdate::default();
                for (&id, &position) in &self.touch_points {
                    update.merge(ui.pointer_event(
                        PointerEvent {
                            id,
                            kind: argui_core::PointerKind::Touch,
                            phase: PointerPhase::Cancelled,
                            position,
                            button: Some(argui_core::PointerButton::Primary),
                            buttons: 0,
                            pressure: None,
                            primary: self.primary_touch == Some(id),
                            modifiers: self.modifiers,
                            timestamp: self.input_epoch.elapsed(),
                        },
                        &layout.hit_regions,
                    ));
                }
                self.apply_ui_update(update, window, event_loop);
            }
        }
        if zoom.consumed {
            match event.phase {
                PointerPhase::Pressed | PointerPhase::Moved | PointerPhase::Entered => {
                    self.touch_points.insert(event.id, event.position);
                }
                PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                    self.touch_points.remove(&event.id);
                }
            }
            if matches!(
                event.phase,
                PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left
            ) && event.primary
            {
                self.primary_touch = self.touch_points.keys().min_by_key(|id| id.get()).copied();
            }
            return event;
        }
        let started_selection_handle = event.phase == PointerPhase::Pressed
            && event.primary
            && self.begin_touch_selection_handle(event.id, event.position, window, event_loop);
        let dragging_selection_handle = started_selection_handle
            || self
                .touch_selection_handle
                .is_some_and(|capture| capture.tracks(event.id));
        if !dragging_selection_handle {
            match event.phase {
                PointerPhase::Pressed if event.primary => {
                    self.begin_touch_selection(event.id, event.position, window);
                }
                PointerPhase::Moved => {
                    self.move_touch_selection_candidate(event.id, event.position)
                }
                PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                    self.cancel_touch_selection(event.id);
                }
                PointerPhase::Entered | PointerPhase::Pressed => {}
            }
        }
        match event.phase {
            PointerPhase::Pressed => {
                self.touch_points.insert(event.id, event.position);
            }
            PointerPhase::Moved => {
                self.touch_points.insert(event.id, event.position);
            }
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                self.touch_points.remove(&event.id);
            }
            PointerPhase::Entered => {}
        }
        let mut pointer_default = None;
        let mut selecting = false;
        let update = if let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) {
            let is_selecting = ui.document_selection_dragging() && event.primary;
            let update = if dragging_selection_handle && event.phase == PointerPhase::Moved {
                Self::closest_static_text(layout, event.position)
                    .map_or_else(InteractionUpdate::default, |position| {
                        ui.drag_document_selection(position)
                    })
            } else if dragging_selection_handle {
                InteractionUpdate::default()
            } else if is_selecting && event.phase == PointerPhase::Moved {
                Self::closest_static_text(layout, event.position)
                    .map_or_else(InteractionUpdate::default, |position| {
                        ui.drag_document_selection(position)
                    })
            } else {
                ui.pointer_event(event, &layout.hit_regions)
            };
            pointer_default = update
                .events
                .iter()
                .find(|delivery| {
                    matches!(delivery.kind, argui_ui::UiEventKind::Pointer(pointer)
                    if pointer.id == event.id && pointer.phase == event.phase)
                })
                .cloned();
            selecting = is_selecting;
            Some(update)
        } else {
            None
        };
        if let Some(update) = update {
            self.apply_ui_update(update, window, event_loop);
        }
        if event.phase == PointerPhase::Pressed {
            let capture_update = match (&mut self.ui_tree, &self.ui_layout) {
                (Some(ui), Some(layout)) => ui.pointer_press_default(
                    event.id,
                    pointer_default.as_ref(),
                    &layout.hit_regions,
                ),
                _ => InteractionUpdate::default(),
            };
            self.apply_ui_update(capture_update, window, event_loop);
        }
        let captured = self
            .ui_tree
            .as_ref()
            .is_some_and(|ui| ui.pointer_captured(event.id));
        let default_prevented = pointer_default
            .as_ref()
            .is_some_and(argui_ui::UiEvent::default_prevented);
        let scroll_blocked = selecting
            || dragging_selection_handle
            || captured
            || default_prevented
            || !event.primary;
        match event.phase {
            PointerPhase::Pressed if !scroll_blocked => {
                self.touch_scroll.begin(event.id, event.position);
            }
            PointerPhase::Moved if scroll_blocked => {
                if let Some(update) = self.touch_scroll.end(event.id, true) {
                    self.apply_touch_scroll(update, event.position, window, event_loop);
                }
            }
            PointerPhase::Moved => {
                let update = self.ui_layout.as_ref().and_then(|layout| {
                    self.touch_scroll.moved(
                        event.id,
                        event.position,
                        self.pointer_settings.touch_slop(),
                        &layout.scroll_regions,
                    )
                });
                if let Some(update) = update {
                    self.apply_touch_scroll(update, event.position, window, event_loop);
                }
            }
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                if let Some(update) = self.touch_scroll.end(
                    event.id,
                    event.phase != PointerPhase::Released || scroll_blocked,
                ) {
                    self.apply_touch_scroll(update, event.position, window, event_loop);
                }
            }
            PointerPhase::Entered | PointerPhase::Pressed => {}
        }
        if event.phase == PointerPhase::Pressed
            && pointer_default.is_some_and(|event| !event.default_prevented())
            && let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout)
        {
            let update = ui.focus_pointer_default(event.id, &layout.hit_regions);
            self.apply_ui_update(update, window, event_loop);
        }
        if !dragging_selection_handle
            && matches!(event.phase, PointerPhase::Pressed | PointerPhase::Released)
        {
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
            if self
                .touch_selection_handle
                .is_some_and(|capture| capture.tracks(event.id))
            {
                self.touch_selection_handle = None;
            }
            self.primary_touch = self.touch_points.keys().min_by_key(|id| id.get()).copied();
        }
        event
    }
}
