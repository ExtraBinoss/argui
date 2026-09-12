use super::{BackdropBackend, BackdropError, Request};
use argui_core::Rect;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub(super) struct Backend;
impl BackdropBackend for Backend {
    fn new(_: RawDisplayHandle, _: RawWindowHandle) -> Result<Self, BackdropError> {
        Err(BackdropError::Unsupported)
    }
    fn available(&mut self) -> Result<bool, BackdropError> {
        Ok(false)
    }
    fn apply(&mut self, _: &[Rect], _: &Request) -> Result<(), BackdropError> {
        Ok(())
    }
}
