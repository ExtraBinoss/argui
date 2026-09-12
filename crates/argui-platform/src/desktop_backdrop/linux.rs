use super::{BackdropBackend, BackdropError, Request};
use argui_core::Rect;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};
mod wayland;
mod x11;

pub(super) enum Backend {
    Wayland(wayland::Backend),
    X11(x11::Backend),
}
#[cfg_attr(coverage_nightly, coverage(off))]
impl BackdropBackend for Backend {
    fn new(display: RawDisplayHandle, window: RawWindowHandle) -> Result<Self, BackdropError> {
        match window {
            RawWindowHandle::Wayland(_) => {
                wayland::Backend::new(display, window).map(Self::Wayland)
            }
            RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_) => {
                x11::Backend::new(display, window).map(Self::X11)
            }
            _ => Err(BackdropError::Unsupported),
        }
    }
    fn available(&mut self) -> Result<bool, BackdropError> {
        match self {
            Self::Wayland(backend) => backend.available(),
            Self::X11(backend) => backend.available(),
        }
    }
    fn apply(&mut self, regions: &[Rect], request: &Request) -> Result<(), BackdropError> {
        match self {
            Self::Wayland(backend) => backend.apply(regions, request),
            Self::X11(backend) => backend.apply(regions, request),
        }
    }
}
