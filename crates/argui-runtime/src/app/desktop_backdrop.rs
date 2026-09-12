use super::Application;
use argui_platform::desktop_backdrop::NativeBackdrop;
use std::sync::Arc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn focus_desktop_backdrop(&mut self, focused: bool) {
        if let Some(ui) = &mut self.ui_tree {
            let state = argui_ui::DesktopBackdropState {
                focused,
                ..ui.desktop_backdrop_state()
            };
            if ui.set_desktop_backdrop_state(state) {
                self.pending_ui_frame.request_paint();
            }
        }
    }
    pub(super) fn initialize_desktop_backdrop<T: HasWindowHandle + HasDisplayHandle + 'static>(
        &mut self,
        owner: Arc<T>,
    ) {
        let Some(material) = self.window_config.desktop_backdrop else {
            return;
        };
        match NativeBackdrop::new(owner).and_then(|mut backdrop| {
            let available = backdrop.update(
                &[],
                material,
                self.environment.color_scheme,
                self.viewport,
                self.scale_factor,
            )?;
            self.environment.desktop_backdrop_available =
                available && !self.environment.high_contrast;
            Ok(backdrop)
        }) {
            Ok(backdrop) => self.desktop_backdrop = Some(backdrop),
            Err(error) => (self.on_event)(crate::RuntimeEvent::DesktopBackdropUnavailable(
                error.to_string(),
            )),
        }
    }

    pub(super) fn sync_desktop_backdrop(&mut self) {
        let Some(material) = self.window_config.desktop_backdrop else {
            return;
        };
        let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout) else {
            return;
        };
        let shapes: Vec<_> = layout
            .desktop_backdrops
            .iter()
            .filter(|region| {
                !self.environment.high_contrast && ui.native_portal_owner(region.node).is_none()
            })
            .map(|region| region.shape.clone())
            .collect();
        let available = match self.desktop_backdrop.as_mut().map(|backdrop| {
            backdrop.update(
                &shapes,
                material,
                self.environment.color_scheme,
                self.viewport,
                self.scale_factor,
            )
        }) {
            Some(Ok(available)) => available && !self.environment.high_contrast,
            Some(Err(error)) => {
                (self.on_event)(crate::RuntimeEvent::DesktopBackdropUnavailable(
                    error.to_string(),
                ));
                self.desktop_backdrop = None;
                false
            }
            None => false,
        };
        let state = argui_ui::DesktopBackdropState {
            available,
            ..ui.desktop_backdrop_state()
        };
        if ui.set_desktop_backdrop_state(state) {
            self.repaint();
        }
        if self.environment.desktop_backdrop_available != available {
            self.environment.desktop_backdrop_available = available;
            self.pending_ui_frame.request_rebuild();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}
