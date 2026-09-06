use super::{HostId, WindowHost};
use argui_platform::{WindowCapabilities, gtk_host::GtkWindow};
use gtk::prelude::*;
use std::cell::Cell;
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    window::{CursorIcon, Window, WindowLevel},
};

pub(crate) struct GtkHost {
    pub(crate) platform: GtkWindow,
    pub(crate) ime_enabled: Cell<bool>,
}

impl WindowHost for GtkHost {
    fn id(&self) -> HostId {
        HostId::Gtk(self.platform.native().id())
    }
    fn set_visible(&self, visible: bool) {
        self.platform.native().set_visible(visible);
    }
    fn is_visible(&self) -> Option<bool> {
        Some(self.platform.native().is_visible())
    }
    fn focus_window(&self) {
        self.platform.native().set_focus();
    }
    fn set_title(&self, title: &str) {
        self.platform.native().set_title(title);
    }
    fn set_minimized(&self, minimized: bool) {
        self.platform.native().set_minimized(minimized);
    }
    fn set_window_level(&self, level: WindowLevel) {
        self.platform
            .native()
            .set_always_on_top(level == WindowLevel::AlwaysOnTop);
        self.platform
            .native()
            .set_always_on_bottom(level == WindowLevel::AlwaysOnBottom);
    }
    fn set_cursor_hittest(&self, enabled: bool) -> Result<(), String> {
        self.platform
            .native()
            .set_ignore_cursor_events(!enabled)
            .map_err(|error| error.to_string())
    }
    fn request_redraw(&self) {
        self.platform.native().request_redraw();
    }
    fn inner_size(&self) -> PhysicalSize<u32> {
        let (width, height, _) = self.platform.client_size();
        PhysicalSize::new(width, height)
    }
    fn pre_present_notify(&self) {
        // Tao/GTK has no presentation notification API. WGPU owns the child surface commits.
    }
    fn set_cursor(&self, cursor: CursorIcon) {
        if let Some(window) = self.platform.container().window() {
            let cursor = gtk::gdk::Cursor::from_name(&window.display(), cursor.name());
            window.set_cursor(cursor.as_ref());
        }
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.ime_enabled.set(allowed);
    }
    fn set_ime_cursor_area(&self, position: LogicalPosition<f64>, _: LogicalSize<f64>) {
        let allocation = self.platform.container().allocation();
        self.platform
            .native()
            .set_ime_position(tao::dpi::LogicalPosition::new(
                position.x + f64::from(allocation.x()),
                position.y + f64::from(allocation.y()),
            ));
    }
    fn is_maximized(&self) -> bool {
        self.platform.native().is_maximized()
    }
    fn set_maximized(&self, maximized: bool) {
        self.platform.native().set_maximized(maximized);
    }
    fn drag_window(&self) -> Result<(), String> {
        self.platform
            .native()
            .drag_window()
            .map_err(|error| error.to_string())
    }
    fn capabilities(&self) -> WindowCapabilities {
        argui_platform::WindowBackend::Wayland.capabilities()
    }
    fn winit(&self) -> Option<&Window> {
        None
    }
    fn gtk(&self) -> Option<&GtkHost> {
        Some(self)
    }
    fn recreate_surface(
        &self,
        renderer: &mut argui_render::SurfaceRenderer,
    ) -> Result<(), argui_render::RendererError> {
        renderer.recreate_surface(self.platform.canvas())
    }
}
