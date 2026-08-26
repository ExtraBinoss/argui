use argui_platform::{ImeInput, Key, KeyInput, KeyState};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::app::Application;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn refresh_text_inputs(&mut self) -> bool {
        let (Some(ui), Some(layout)) = (&self.ui_tree, &mut self.ui_layout) else {
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
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) else {
            return;
        };
        let update = if input.state == KeyState::Pressed && input.key == Key::Tab {
            ui.focus_next(&layout.hit_regions, input.modifiers.shift)
        } else if input.state == KeyState::Pressed
            && matches!(input.key, Key::ArrowLeft | Key::ArrowRight)
            && let Some(node) = ui.focused_node()
            && let Some(position) = ui.text_input_position(node)
            && let Some(region) = layout.text_inputs.iter().find(|region| region.node == node)
        {
            ui.move_text_position(
                node,
                region.visual_neighbor(
                    position,
                    input.key == Key::ArrowLeft,
                    input.modifiers.command() || input.modifiers.alt,
                ),
                input.modifiers.shift,
            )
        } else {
            ui.key_input(input)
        };
        self.apply_ui_update(update, window, event_loop);
        self.update_ime(window);
    }

    pub(super) fn ime_input(
        &mut self,
        input: ImeInput,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let update = ui.ime_input(input);
        self.apply_ui_update(update, window, event_loop);
    }

    pub(super) fn update_ime(&self, window: &Window) {
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
