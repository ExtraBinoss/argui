use std::sync::Arc;
mod event_loop;
pub(crate) use event_loop::EventProxy;
#[cfg(all(feature = "webview", target_os = "linux"))]
pub(crate) mod gtk;

use argui_core::Insets;
#[cfg(any(target_os = "android", target_os = "ios"))]
use argui_core::{Point, Rect, Size};
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
    fn is_minimized(&self) -> Option<bool>;
    fn set_window_level(&self, level: winit::window::WindowLevel);
    fn set_cursor_hittest(&self, enabled: bool) -> Result<(), String>;
    fn request_redraw(&self);
    /// Physical drawable extent. iOS uses Winit's outer bounds because its
    /// inner bounds describe the safe area rather than the full surface.
    fn drawable_size(&self) -> PhysicalSize<u32>;
    fn safe_area_insets(&self, _scale_factor: f32) -> Option<Insets> {
        None
    }
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
    fn is_minimized(&self) -> Option<bool> {
        self.as_ref().is_minimized()
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
    fn drawable_size(&self) -> PhysicalSize<u32> {
        #[cfg(target_arch = "wasm32")]
        {
            argui_platform::web_drawable_size(self.as_ref())
        }
        #[cfg(all(not(target_arch = "wasm32"), target_os = "ios"))]
        {
            self.as_ref().outer_size()
        }
        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
        {
            self.as_ref().inner_size()
        }
    }
    fn safe_area_insets(&self, scale_factor: f32) -> Option<Insets> {
        #[cfg(target_os = "android")]
        {
            android_safe_area_insets(self.as_ref(), scale_factor)
        }
        #[cfg(target_os = "ios")]
        {
            ios_safe_area_insets(self.as_ref(), scale_factor)
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let _ = scale_factor;
            None
        }
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
    fn is_minimized(&self) -> Option<bool> {
        self.as_ref().is_minimized()
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
    fn drawable_size(&self) -> PhysicalSize<u32> {
        self.as_ref().drawable_size()
    }
    fn safe_area_insets(&self, scale_factor: f32) -> Option<Insets> {
        self.as_ref().safe_area_insets(scale_factor)
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

#[cfg(target_os = "android")]
fn android_safe_area_insets(window: &Window, scale_factor: f32) -> Option<Insets> {
    use winit::platform::android::WindowExtAndroid;

    let size = window.inner_size();
    let content = window.content_rect();
    Insets::try_from_physical_rects(
        Rect::new(
            Point::default(),
            Size::new(size.width as f32, size.height as f32),
        ),
        Rect::new(
            Point::new(content.left as f32, content.top as f32),
            Size::new(
                content.right as f32 - content.left as f32,
                content.bottom as f32 - content.top as f32,
            ),
        ),
        scale_factor,
    )
}

#[cfg(target_os = "ios")]
fn ios_safe_area_insets(window: &Window, scale_factor: f32) -> Option<Insets> {
    let outer_position = window.outer_position().ok()?;
    let safe_position = window.inner_position().ok()?;
    let outer_size = window.outer_size();
    let safe_size = window.inner_size();
    Insets::try_from_physical_rects(
        Rect::new(
            Point::new(outer_position.x as f32, outer_position.y as f32),
            Size::new(outer_size.width as f32, outer_size.height as f32),
        ),
        Rect::new(
            Point::new(safe_position.x as f32, safe_position.y as f32),
            Size::new(safe_size.width as f32, safe_size.height as f32),
        ),
        scale_factor,
    )
}

pub(crate) trait LoopControl {
    fn exit(&self);
    #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
    fn popup(
        &self,
        _parent: &Window,
        _kind: argui_platform::popup::PopupKind,
        _environment: argui_platform::popup::PopupEnvironment,
        _bounds: argui_core::Rect,
    ) -> Result<argui_platform::popup::NativePopup, argui_platform::popup::PopupUnavailable> {
        Err(argui_platform::popup::PopupUnavailable::UnsupportedBackend)
    }
}

impl LoopControl for winit::event_loop::ActiveEventLoop {
    #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
    fn popup(
        &self,
        parent: &Window,
        kind: argui_platform::popup::PopupKind,
        environment: argui_platform::popup::PopupEnvironment,
        bounds: argui_core::Rect,
    ) -> Result<argui_platform::popup::NativePopup, argui_platform::popup::PopupUnavailable> {
        argui_platform::popup::NativePopup::create(self, parent, kind, environment, bounds)
    }
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
