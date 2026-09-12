//! Native owned surfaces. UI placement and scrolling belong to argui-ui/layout.
use std::sync::Arc;

use argui_core::{Point, Rect, Size};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::Backend;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::Backend;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::Backend;
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod unsupported;
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
use unsupported::Backend;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopupKind {
    Tooltip,
    Menu,
    Popover,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PopupUnavailable {
    UnsupportedBackend,
    UnknownGeometry,
    Platform(String),
}

impl std::fmt::Display for PopupUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedBackend => f.write_str("the window backend has no native popup API"),
            Self::UnknownGeometry => f.write_str("the parent or screen geometry is unavailable"),
            Self::Platform(error) => f.write_str(error),
        }
    }
}
impl std::error::Error for PopupUnavailable {}

/// Converts between desktop physical pixels and the owning UI tree's logical coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupEnvironment {
    pub origin: Point,
    pub scale: f32,
    pub work_area: Rect,
}

impl PopupEnvironment {
    #[must_use]
    pub fn from_physical(origin: Point, scale: f32, work_area: Rect) -> Option<Self> {
        if !scale.is_finite()
            || scale <= 0.0
            || ![
                origin.x,
                origin.y,
                work_area.origin.x,
                work_area.origin.y,
                work_area.size.width,
                work_area.size.height,
            ]
            .into_iter()
            .all(f32::is_finite)
            || work_area.size.width <= 0.0
            || work_area.size.height <= 0.0
        {
            return None;
        }
        Some(Self {
            origin,
            scale,
            work_area: Rect::new(
                Point::new(
                    (work_area.origin.x - origin.x) / scale,
                    (work_area.origin.y - origin.y) / scale,
                ),
                Size::new(work_area.size.width / scale, work_area.size.height / scale),
            ),
        })
    }

    /// Accept physical pixel rounding before layout to avoid alternating sizes at fractional DPI.
    #[must_use]
    pub fn snap(self, bounds: Rect) -> Rect {
        let position = self.position(bounds);
        let size = self.size(bounds);
        Rect::new(
            Point::new(
                (position.x as f32 - self.origin.x) / self.scale,
                (position.y as f32 - self.origin.y) / self.scale,
            ),
            Size::new(
                size.width as f32 / self.scale,
                size.height as f32 / self.scale,
            ),
        )
    }

    #[must_use]
    pub fn position(self, bounds: Rect) -> PhysicalPosition<i32> {
        PhysicalPosition::new(
            (self.origin.x + bounds.origin.x * self.scale).round() as i32,
            (self.origin.y + bounds.origin.y * self.scale).round() as i32,
        )
    }

    #[must_use]
    pub fn size(self, bounds: Rect) -> PhysicalSize<u32> {
        PhysicalSize::new(
            (bounds.size.width * self.scale).round().max(1.0) as u32,
            (bounds.size.height * self.scale).round().max(1.0) as u32,
        )
    }
}

/// Each OS implements only ownership, window flags and the usable desktop area.
trait PopupBackend {
    fn work_area(parent: &Window) -> Result<Rect, PopupUnavailable>;
    fn attributes(
        parent: &Window,
        kind: PopupKind,
        attributes: WindowAttributes,
    ) -> Result<WindowAttributes, PopupUnavailable>;
    fn attach(parent: &Window, window: &Window, kind: PopupKind) -> Result<(), PopupUnavailable>;
    fn detach(window: &Window);
    fn focus(window: &Window) {
        window.focus_window();
    }
}

/// An undecorated surface owned by its logical parent. It never creates a UI model.
pub struct NativePopup {
    window: Arc<Window>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl NativePopup {
    pub fn environment(parent: &Window) -> Result<PopupEnvironment, PopupUnavailable> {
        let area = Backend::work_area(parent)?;
        let origin = parent
            .inner_position()
            .map_err(|_| PopupUnavailable::UnknownGeometry)?;
        PopupEnvironment::from_physical(
            Point::new(origin.x as f32, origin.y as f32),
            parent.scale_factor() as f32,
            area,
        )
        .ok_or(PopupUnavailable::UnknownGeometry)
    }

    pub fn create(
        event_loop: &ActiveEventLoop,
        parent: &Window,
        kind: PopupKind,
        environment: PopupEnvironment,
        bounds: Rect,
    ) -> Result<Self, PopupUnavailable> {
        let attributes = Window::default_attributes()
            .with_title("Argui popup")
            .with_decorations(false)
            .with_resizable(false)
            .with_visible(false)
            .with_active(false)
            .with_inner_size(environment.size(bounds))
            .with_position(environment.position(bounds));
        let attributes = Backend::attributes(parent, kind, attributes)?;
        let window = event_loop
            .create_window(attributes)
            .map_err(|error| PopupUnavailable::Platform(error.to_string()))?;
        Backend::attach(parent, &window, kind)?;
        Ok(Self {
            window: Arc::new(window),
        })
    }

    #[must_use]
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn focus(&self) {
        Backend::focus(&self.window);
    }

    pub fn reposition(&self, environment: PopupEnvironment, bounds: Rect) {
        self.window.set_outer_position(environment.position(bounds));
        let size = environment.size(bounds);
        if self.window.inner_size() != size {
            let _ = self.window.request_inner_size(size);
        }
    }
}

impl Drop for NativePopup {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn drop(&mut self) {
        self.window.set_visible(false);
        Backend::detach(&self.window);
    }
}
