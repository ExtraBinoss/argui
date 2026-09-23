use argui_core::Insets;
use argui_platform::PlatformEvent;

use crate::RuntimeEvent;

use super::Application;

impl Application {
    pub(crate) fn set_safe_area_insets(&mut self, override_insets: Option<Insets>) {
        self.window_config.safe_area_insets = override_insets;
        let detected = self
            .window
            .as_ref()
            .and_then(|window| window.safe_area_insets(self.scale_factor));
        self.apply_safe_area_insets(override_insets.or(detected).unwrap_or_default());
    }

    pub(super) fn refresh_safe_area_insets(
        &mut self,
        window: &dyn crate::host::WindowHost,
        scale_factor: f32,
    ) {
        let detected = window.safe_area_insets(scale_factor);
        self.apply_safe_area_insets(
            self.window_config
                .safe_area_insets
                .or(detected)
                .unwrap_or_default(),
        );
    }

    fn apply_safe_area_insets(&mut self, insets: Insets) {
        if self.environment.safe_area_insets == insets {
            return;
        }
        self.environment.safe_area_insets = insets;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(root) = self
            .native_host
            .as_ref()
            .and_then(|host| host.root_element())
        {
            let root = crate::app::native_host::native_host_root_with_safe_area(root, insets);
            if let Some(tree) = &mut self.ui_tree {
                tree.update(root);
            } else {
                self.ui_tree = Some(argui_ui::UiTree::new(root));
            }
            self.pending_ui_frame.request_layout();
        }
        self.pending_ui_frame.request_rebuild();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::SafeAreaChanged(
            insets,
        )));
    }
}
