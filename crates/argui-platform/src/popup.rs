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
/// Native window style requested for a popup surface.
pub enum PopupKind {
    /// Short-lived informational tooltip.
    Tooltip,
    /// Context or command menu.
    Menu,
    /// Anchored popover panel.
    Popover,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Reason a native popup could not be queried or created.
pub enum PopupUnavailable {
    /// The current window backend has no native popup API.
    UnsupportedBackend,
    /// Parent or screen geometry could not be determined.
    UnknownGeometry,
    /// A native platform operation failed.
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
    /// Physical screen origin of the owning window.
    pub origin: Point,
    /// Physical pixels per logical coordinate.
    pub scale: f32,
    /// Usable screen rectangle in the logical coordinate space.
    pub work_area: Rect,
}

impl PopupEnvironment {
    /// Builds a logical-coordinate environment from physical screen geometry.
    /// Returns `None` for non-finite, non-positive, or invalid geometry.
    /// `origin` is the physical parent origin, `scale` is pixels per logical unit, and `work_area` is the usable physical screen rectangle.
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
    /// `bounds` is the popup rectangle in logical coordinates.
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
    /// Converts logical popup bounds to a rounded physical screen position.
    pub fn position(self, bounds: Rect) -> PhysicalPosition<i32> {
        PhysicalPosition::new(
            (self.origin.x + bounds.origin.x * self.scale).round() as i32,
            (self.origin.y + bounds.origin.y * self.scale).round() as i32,
        )
    }

    #[must_use]
    /// Converts logical popup bounds to a rounded physical size of at least one pixel.
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
    /// Resolves the parent's screen origin, scale, and usable work area.
    ///
    /// # Errors
    /// Returns an error if the backend cannot query usable geometry.
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

    /// Creates a hidden native popup owned by `parent` at the supplied logical bounds.
    ///
    /// # Errors
    /// Returns an error if the backend rejects the popup, creation fails, or ownership attachment fails.
    /// `event_loop` creates the OS window; `parent` owns it; `kind`, `environment`, and `bounds` define its style and placement.
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
    /// Returns the native popup window.
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    /// Requests focus for the popup window.
    pub fn focus(&self) {
        Backend::focus(&self.window);
    }

    /// Moves and resizes the popup to the supplied logical bounds.
    /// `environment` supplies the current screen conversion and `bounds` is the desired logical rectangle.
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
