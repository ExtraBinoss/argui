use super::MultiApplication;
use crate::host::WindowFactory;

impl MultiApplication {
    pub(super) fn collect_app_commands(&mut self, updates: &mut Vec<crate::AppUpdate>) {
        for entry in self.windows.values_mut() {
            let commands = std::mem::take(&mut entry.runtime.pending_app_commands);
            if !commands.is_empty() {
                updates.push(crate::AppUpdate {
                    commands,
                    ..crate::AppUpdate::none()
                });
            }
        }
    }

    pub(super) fn models_ready(&mut self, event_loop: &dyn WindowFactory) {
        let mut visited = self.retired.clone();
        for entry in self.windows.values() {
            entry.runtime.collect_model_runtimes(&mut visited);
        }
        for runtime in visited {
            runtime.dispatch_pending();
        }
        for entry in self.windows.values_mut() {
            entry.runtime.collect_model_effects(event_loop);
        }
        self.retired.retain(|runtime| {
            runtime.pending_events() != 0
                || runtime.pending_invalidations() != 0
                || runtime.pending_mount_events() != 0
        });
        self.process_pending(event_loop);
    }

    #[cfg(feature = "tasks")]
    pub(super) fn tasks_ready(&mut self, event_loop: &dyn WindowFactory) {
        if let Some(tasks) = &self.tasks {
            tasks.drain();
        }
        for entry in self.windows.values_mut() {
            entry.runtime.tasks_ready(event_loop);
        }
        self.process_pending(event_loop);
    }
}
