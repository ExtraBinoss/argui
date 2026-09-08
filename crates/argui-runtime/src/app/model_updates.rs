use super::Application;
use crate::{ViewUpdate, host::LoopControl};

impl Application {
    pub(crate) fn collect_model_runtimes(&self, visited: &mut Vec<crate::ModelRuntime>) {
        if let Some(model) = &self.model {
            model.collect_model_runtimes(visited);
        }
    }

    pub(crate) fn models_ready(&mut self, control: &dyn LoopControl) {
        let mut runtimes = Vec::new();
        self.collect_model_runtimes(&mut runtimes);
        for runtime in runtimes {
            runtime.dispatch_pending();
        }
        self.collect_model_effects(control);
    }

    pub(crate) fn collect_model_effects(&mut self, control: &dyn LoopControl) {
        let Some(model) = &self.model else {
            return;
        };
        let effects = model.take_model_effects();
        self.apply_model_effects(effects, control);
    }

    #[cfg(feature = "tasks")]
    pub(crate) fn tasks_ready(&mut self, control: &dyn LoopControl) {
        let Some(model) = &self.model else {
            return;
        };
        let effects = model.take_task_effects();
        self.apply_model_effects(effects, control);
    }

    fn apply_model_effects(
        &mut self,
        effects: crate::model::effects::ContextEffects,
        control: &dyn LoopControl,
    ) {
        self.pending_app_commands.extend(effects.commands);
        self.pending_ui_frame.merge(
            &argui_ui::InteractionUpdate {
                paint_changed: effects.update == ViewUpdate::Paint,
                ..Default::default()
            },
            effects.update == ViewUpdate::Rebuild,
        );
        self.pending_ui_frame.request_focus(effects.focus);
        self.pending_ui_frame.request_scroll(effects.scroll);
        self.pending_ui_frame
            .request_text_selection(effects.text_selection);
        if let Some(theme) = effects.theme {
            self.apply_theme_request(theme);
            self.pending_ui_frame.request_layout();
        }
        if let Some(window) = self.window.clone() {
            if let Some(clipboard) = effects.clipboard {
                self.clipboard_request(clipboard, &window, control);
            }
            for command in effects.ui_commands {
                if let Some(ui) = &mut self.ui_tree {
                    let update = ui.apply_command(command);
                    self.apply_ui_update(update, &window, control);
                }
            }
            let animation_changed = self.sync_animations();
            if self.presentation_visible
                && (self.pending_ui_frame.needs_frame() || animation_changed)
            {
                window.request_redraw();
            }
        }
    }
}
