use super::MultiApplication;
use crate::AppEvent;
use argui_platform::{CloseBehavior, WindowKey};

impl MultiApplication {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn close_window(&mut self, key: &WindowKey) {
        if let Some(entry) = self.windows.remove(key) {
            if let Some(window_id) = entry.runtime.window_id() {
                self.by_native.remove(&window_id);
            }
            entry.runtime.collect_model_runtimes(&mut self.retired);
            drop(entry);
            let update = self.model.borrow_mut().update(&AppEvent::Window {
                window: key.clone(),
                event: argui_platform::PlatformEvent::Closed,
            });
            self.pending.borrow_mut().push(update);
        }
    }

    pub(crate) fn shutdown(&mut self) {
        #[cfg(feature = "tasks")]
        self.shutdown_tasks();
        let roots: Vec<_> = self
            .windows
            .values()
            .filter_map(|entry| entry.runtime.model.clone())
            .collect();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::shutdown_presentations(&roots)
        }));
        let windows: Vec<_> = self.windows.keys().cloned().collect();
        let mut panic = result.err();
        for key in windows {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.close_window(&key)));
            if panic.is_none() {
                panic = result.err();
            }
        }
        for runtime in self.retired.drain(..) {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.shutdown_host()));
            if panic.is_none() {
                panic = result.err();
            }
        }
        self.pending.borrow_mut().clear();
        if let Some(payload) = panic
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }
}

impl MultiApplication {
    pub(super) fn handle_close(
        &mut self,
        key: &WindowKey,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let behavior = self.windows.get(key).map_or(CloseBehavior::Quit, |entry| {
            entry.spec.window.close_behavior
        });
        match behavior {
            CloseBehavior::Quit => event_loop.exit(),
            CloseBehavior::CloseWindow => self.close_window(key),
            CloseBehavior::Hide => self.set_visible(key, false),
            CloseBehavior::NotifyApp => {}
        }
    }
}
