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
        self.prepared_text = Some(self.text_engine.prepare(&layout.text, self.scale_factor));
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
        let mut update = ui.keyboard_default(input, &layout.hit_regions);
        update.merge(ui.scroll_keyboard(input, &layout.scroll_regions));
        if !(input.state == KeyState::Pressed && input.key == Key::Tab && !input.repeat) {
            let editor_update = ui
                .focused_node()
                .and_then(|node| layout.text_inputs.iter().find(|region| region.node == node))
                .and_then(|region| region.navigate(ui, input))
                .unwrap_or_else(|| ui.edit_text_input(input));
            update.merge(editor_update);
        }
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
        let focused_text_input = self
            .ui_tree
            .as_ref()
            .and_then(argui_ui::UiTree::focused_node)
            .filter(|node| {
                self.ui_tree
                    .as_ref()
                    .is_some_and(|ui| ui.text_input_value(*node).is_some())
            });
        let enabled = focused_text_input.is_some();
        #[cfg(target_arch = "wasm32")]
        if let Some(dom) = &self.dom_accessibility {
            dom.sync_touch_text_input(
                focused_text_input.map(|node| argui_accessibility::SemanticNodeId::new(node.get())),
            );
        }
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        {
            let caret = self
                .ui_tree
                .as_ref()
                .and_then(argui_ui::UiTree::focused_node)
                .and_then(|node| {
                    self.ui_layout.as_ref().and_then(|layout| {
                        layout.text_inputs.iter().find(|region| region.node == node)
                    })
                })
                .and_then(|region| region.caret);
            if self.popup_ime(enabled, caret) {
                window.set_ime_allowed(false);
                return;
            }
        }
        window.set_ime_allowed(enabled);
        if enabled
            && let Some(node) = focused_text_input
            && let Some(caret) = self
                .ui_layout
                .as_ref()
                .and_then(|layout| layout.text_inputs.iter().find(|region| region.node == node))
                .and_then(|region| region.caret)
        {
            let zoom = f64::from(self.ui_zoom_factor);
            window.set_ime_cursor_area(
                LogicalPosition::new(
                    f64::from(caret.origin.x) * zoom,
                    f64::from(caret.origin.y + caret.size.height) * zoom,
                ),
                LogicalSize::new(1.0, f64::from(caret.size.height) * zoom),
            );
        }
    }
}
