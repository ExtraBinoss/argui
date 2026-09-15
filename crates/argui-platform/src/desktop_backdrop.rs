//! Optional compositor effects. No capture of the user's desktop is involved.
use argui_core::{BackdropMaterial, ColorScheme, Rect, Size};
use argui_paint::ClipChain;
use std::{any::Any, sync::Arc};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

mod geometry;
pub use geometry::region_rectangles;
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

#[derive(Clone, Debug, PartialEq, Eq)]
/// Error creating or updating a compositor backdrop.
pub enum BackdropError {
    /// The current platform has no supported compositor backdrop API.
    Unsupported,
    /// Input geometry or scale is invalid.
    InvalidGeometry,
    /// The native platform operation failed.
    Platform(String),
}
impl std::fmt::Display for BackdropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("desktop blur is unavailable on this window backend"),
            Self::InvalidGeometry => f.write_str("invalid desktop backdrop geometry"),
            Self::Platform(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for BackdropError {}

#[derive(Clone, Debug, PartialEq)]
struct Request {
    shapes: Vec<ClipChain>,
    material: BackdropMaterial,
    scheme: ColorScheme,
    size: Size,
    scale: f32,
}

trait BackdropBackend: Sized {
    fn new(display: RawDisplayHandle, window: RawWindowHandle) -> Result<Self, BackdropError>;
    fn available(&mut self) -> Result<bool, BackdropError>;
    fn apply(&mut self, regions: &[Rect], request: &Request) -> Result<(), BackdropError>;
}

/// Retains the surface owner and only owns the effect objects it creates.
/// Create, update and drop on the window's event-loop thread.
pub struct NativeBackdrop {
    backend: Backend,
    last: Option<Request>,
    available: bool,
    _owner: Box<dyn Any>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl NativeBackdrop {
    /// Creates a compositor backdrop associated with the native window owner.
    ///
    /// # Errors
    /// Returns an error if native handles cannot be obtained or the platform backend cannot initialize.
    /// `window` is retained as the owner of the native handles for this backdrop.
    pub fn new<T: HasWindowHandle + HasDisplayHandle + 'static>(
        window: Arc<T>,
    ) -> Result<Self, BackdropError> {
        let display = window
            .display_handle()
            .map_err(|error| BackdropError::Platform(error.to_string()))?
            .as_raw();
        let handle = window
            .window_handle()
            .map_err(|error| BackdropError::Platform(error.to_string()))?
            .as_raw();
        let backend = Backend::new(display, handle)?;
        Ok(Self {
            backend,
            last: None,
            available: false,
            _owner: Box::new(window),
        })
    }

    /// Returns native availability, even for an empty region set. Identical updates do no OS work.
    ///
    /// # Errors
    /// Returns an error for invalid scale or size, invalid region geometry, or a platform failure.
    /// `shapes` describe regions, `material` and `scheme` choose appearance, and `size`/`scale` describe the surface.
    pub fn update(
        &mut self,
        shapes: &[ClipChain],
        material: BackdropMaterial,
        scheme: ColorScheme,
        size: Size,
        scale: f32,
    ) -> Result<bool, BackdropError> {
        if !scale.is_finite() || scale <= 0.0 || !size.width.is_finite() || !size.height.is_finite()
        {
            return Err(BackdropError::InvalidGeometry);
        }
        let available = self.backend.available()?;
        let request = Request {
            shapes: shapes.to_vec(),
            material,
            scheme,
            size,
            scale,
        };
        if self.last.as_ref() != Some(&request) || self.available != available {
            let regions = if available {
                region_rectangles(shapes)?
            } else {
                Vec::new()
            };
            self.backend.apply(&regions, &request)?;
            self.last = Some(request);
            self.available = available;
        }
        Ok(available)
    }
}
