use std::sync::Arc;
mod event_loop;
pub(crate) use event_loop::EventProxy;
#[cfg(all(feature = "webview", target_os = "linux"))]
pub(crate) mod gtk;

use argui_platform::WindowCapabilities;
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    window::{CursorIcon, Window},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum HostId {
    Winit(winit::window::WindowId),
    #[cfg(all(feature = "webview", target_os = "linux"))]
    Gtk(tao::window::WindowId),
}

/// Operations used by the retained runtime, independently of the native event loop.
pub(crate) trait WindowHost {
    fn id(&self) -> HostId;
    fn set_visible(&self, visible: bool);
    fn is_visible(&self) -> Option<bool>;
    fn focus_window(&self);
    fn set_title(&self, title: &str);
    fn set_minimized(&self, minimized: bool);
    fn set_window_level(&self, level: winit::window::WindowLevel);
    fn set_cursor_hittest(&self, enabled: bool) -> Result<(), String>;
    fn request_redraw(&self);
    fn inner_size(&self) -> PhysicalSize<u32>;
    fn pre_present_notify(&self);
    fn set_cursor(&self, cursor: CursorIcon);
    fn set_ime_allowed(&self, allowed: bool);
    fn set_ime_cursor_area(&self, position: LogicalPosition<f64>, size: LogicalSize<f64>);
    fn is_maximized(&self) -> bool;
    fn set_maximized(&self, maximized: bool);
    fn drag_window(&self) -> Result<(), String>;
    fn capabilities(&self) -> WindowCapabilities;
    fn winit(&self) -> Option<&Window>;
    #[cfg(all(feature = "webview", target_os = "linux"))]
    fn gtk(&self) -> Option<&gtk::GtkHost> {
        None
    }
    fn recreate_surface(
        &self,
        renderer: &mut argui_render::SurfaceRenderer,
    ) -> Result<(), argui_render::RendererError>;
}

impl WindowHost for Arc<Window> {
    fn id(&self) -> HostId {
        HostId::Winit(self.as_ref().id())
    }
    fn set_visible(&self, visible: bool) {
        self.as_ref().set_visible(visible);
    }
    fn is_visible(&self) -> Option<bool> {
        self.as_ref().is_visible()
    }
    fn focus_window(&self) {
        self.as_ref().focus_window();
    }
    fn set_title(&self, title: &str) {
        self.as_ref().set_title(title);
    }
    fn set_minimized(&self, minimized: bool) {
        self.as_ref().set_minimized(minimized);
    }
    fn set_window_level(&self, level: winit::window::WindowLevel) {
        self.as_ref().set_window_level(level);
    }
    fn set_cursor_hittest(&self, enabled: bool) -> Result<(), String> {
        self.as_ref()
            .set_cursor_hittest(enabled)
            .map_err(|error| error.to_string())
    }
    fn request_redraw(&self) {
        self.as_ref().request_redraw();
    }
    fn inner_size(&self) -> PhysicalSize<u32> {
        self.as_ref().inner_size()
    }
    fn pre_present_notify(&self) {
        self.as_ref().pre_present_notify();
    }
    fn set_cursor(&self, cursor: CursorIcon) {
        self.as_ref().set_cursor(cursor);
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.as_ref().set_ime_allowed(allowed);
    }
    fn set_ime_cursor_area(&self, position: LogicalPosition<f64>, size: LogicalSize<f64>) {
        self.as_ref().set_ime_cursor_area(position, size);
    }
    fn is_maximized(&self) -> bool {
        self.as_ref().is_maximized()
    }
    fn set_maximized(&self, maximized: bool) {
        self.as_ref().set_maximized(maximized);
    }
    fn drag_window(&self) -> Result<(), String> {
        self.as_ref()
            .drag_window()
            .map_err(|error| error.to_string())
    }
    fn capabilities(&self) -> WindowCapabilities {
        argui_platform::window_capabilities(self)
    }
    fn winit(&self) -> Option<&Window> {
        Some(self)
    }
    fn recreate_surface(
        &self,
        renderer: &mut argui_render::SurfaceRenderer,
    ) -> Result<(), argui_render::RendererError> {
        renderer.recreate_surface(self.clone())
    }
}

impl<T: WindowHost + ?Sized> WindowHost for std::rc::Rc<T> {
    #[cfg(all(feature = "webview", target_os = "linux"))]
    fn gtk(&self) -> Option<&gtk::GtkHost> {
        self.as_ref().gtk()
    }
    fn id(&self) -> HostId {
        self.as_ref().id()
    }
    fn set_visible(&self, visible: bool) {
        self.as_ref().set_visible(visible);
    }
    fn is_visible(&self) -> Option<bool> {
        self.as_ref().is_visible()
    }
    fn focus_window(&self) {
        self.as_ref().focus_window();
    }
    fn set_title(&self, title: &str) {
        self.as_ref().set_title(title);
    }
    fn set_minimized(&self, minimized: bool) {
        self.as_ref().set_minimized(minimized);
    }
    fn set_window_level(&self, level: winit::window::WindowLevel) {
        self.as_ref().set_window_level(level);
    }
    fn set_cursor_hittest(&self, enabled: bool) -> Result<(), String> {
        self.as_ref().set_cursor_hittest(enabled)
    }
    fn request_redraw(&self) {
        self.as_ref().request_redraw();
    }
    fn inner_size(&self) -> PhysicalSize<u32> {
        self.as_ref().inner_size()
    }
    fn pre_present_notify(&self) {
        self.as_ref().pre_present_notify();
    }
    fn set_cursor(&self, cursor: CursorIcon) {
        self.as_ref().set_cursor(cursor);
    }
    fn set_ime_allowed(&self, allowed: bool) {
        self.as_ref().set_ime_allowed(allowed);
    }
    fn set_ime_cursor_area(&self, position: LogicalPosition<f64>, size: LogicalSize<f64>) {
        self.as_ref().set_ime_cursor_area(position, size);
    }
    fn is_maximized(&self) -> bool {
        self.as_ref().is_maximized()
    }
    fn set_maximized(&self, maximized: bool) {
        self.as_ref().set_maximized(maximized);
    }
    fn drag_window(&self) -> Result<(), String> {
        self.as_ref().drag_window()
    }
    fn capabilities(&self) -> WindowCapabilities {
        self.as_ref().capabilities()
    }
    fn winit(&self) -> Option<&Window> {
        self.as_ref().winit()
    }
    fn recreate_surface(
        &self,
        renderer: &mut argui_render::SurfaceRenderer,
    ) -> Result<(), argui_render::RendererError> {
        self.as_ref().recreate_surface(renderer)
    }
}

pub(crate) trait LoopControl {
    fn exit(&self);
}

impl LoopControl for winit::event_loop::ActiveEventLoop {
    fn exit(&self) {
        self.exit();
    }
}

pub(crate) trait WindowFactory: LoopControl {
    fn open(&self, runtime: &mut crate::app::Application);
}

impl WindowFactory for winit::event_loop::ActiveEventLoop {
    fn open(&self, runtime: &mut crate::app::Application) {
        use winit::application::ApplicationHandler;
        runtime.resumed(self);
    }
}
