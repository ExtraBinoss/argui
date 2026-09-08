use argui_platform::{ImeInput, Key, KeyInput, KeyState};
use winit::dpi::{LogicalPosition, LogicalSize};

use crate::app::Application;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn refresh_text_inputs(&mut self) -> bool {
        let (Some(ui), Some(layout)) = (&mut self.ui_tree, &mut self.ui_layout) else {
            return false;
        };
        self.layout_engine
            .update_text_inputs(ui, &mut self.text_engine, layout);
        if let Some(prepared) = &mut self.prepared_text {
            for (index, block) in layout.text.blocks().iter().enumerate() {
                prepared.reposition_block(index, block.bounds.origin, block.clip);
            }
        }
        true
    }

    pub(super) fn keyboard_input(
        &mut self,
        input: &KeyInput,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let key_update = ui.keyboard_event(input, &[]);
        let key_event = key_update.events.first().cloned();
        self.apply_ui_update(key_update, window, event_loop);
        if key_event.is_some_and(|event| event.default_prevented()) {
            return;
        }
        let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) else {
            return;
        };
        if ui
            .focused_node()
            .is_some_and(|node| ui.text_input_composing(node))
        {
            return;
        }
        let focused_editor = ui
            .focused_node()
            .is_some_and(|node| ui.text_input_value(node).is_some());
        if input.state == KeyState::Pressed && input.key == Key::Escape && !focused_editor {
            let update = ui.clear_document_selection();
            if update.paint_changed {
                self.apply_ui_update(update, window, event_loop);
                return;
            }
        }
        let update = if input.state == KeyState::Pressed
            && matches!(
                input.key,
                Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown
            )
            && let Some(node) = ui.focused_node()
            && let Some(position) = ui.text_input_position(node)
            && let Some(region) = layout.text_inputs.iter().find(|region| region.node == node)
        {
            let mut update = ui.keyboard_default(input, &layout.hit_regions);
            let position = match input.key {
                Key::ArrowLeft | Key::ArrowRight => region.visual_neighbor(
                    position,
                    input.key == Key::ArrowLeft,
                    input.modifiers.command() || input.modifiers.alt,
                ),
                Key::ArrowUp | Key::ArrowDown => {
                    region.vertical_neighbor(position, input.key == Key::ArrowUp)
                }
                _ => position,
            };
            update.merge(ui.move_text_position(node, position, input.modifiers.shift));
            update
        } else {
            let mut update = ui.keyboard_default(input, &layout.hit_regions);
            if !(input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat) {
                update.merge(ui.edit_text_input(input));
            }
            update
        };
        self.apply_ui_update(update, window, event_loop);
        self.update_ime(window);
    }

    pub(super) fn ime_input(
        &mut self,
        input: ImeInput,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let update = ui.ime_input(input);
        self.apply_ui_update(update, window, event_loop);
    }

    pub(super) fn update_ime(&self, window: &dyn crate::host::WindowHost) {
        let enabled = self
            .ui_tree
            .as_ref()
            .and_then(argui_ui::UiTree::focused_node)
            .is_some_and(|node| {
                self.ui_tree
                    .as_ref()
                    .is_some_and(|ui| ui.text_input_value(node).is_some())
            });
        window.set_ime_allowed(enabled);
        if enabled
            && let Some(node) = self
                .ui_tree
                .as_ref()
                .and_then(argui_ui::UiTree::focused_node)
            && let Some(caret) = self
                .ui_layout
                .as_ref()
                .and_then(|layout| layout.text_inputs.iter().find(|region| region.node == node))
                .and_then(|region| region.caret)
        {
            window.set_ime_cursor_area(
                LogicalPosition::new(
                    f64::from(caret.origin.x),
                    f64::from(caret.origin.y + caret.size.height),
                ),
                LogicalSize::new(1.0, f64::from(caret.size.height)),
            );
        }
    }
}
