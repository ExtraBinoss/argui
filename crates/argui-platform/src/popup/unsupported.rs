use super::{PopupBackend, PopupKind, PopupUnavailable};
use argui_core::Rect;
use winit::window::{Window, WindowAttributes};
pub(super) struct Backend;
impl PopupBackend for Backend {
    fn work_area(_: &Window) -> Result<Rect, PopupUnavailable> {
        Err(PopupUnavailable::UnsupportedBackend)
    }
    fn attributes(
        _: &Window,
        _: PopupKind,
        _: WindowAttributes,
    ) -> Result<WindowAttributes, PopupUnavailable> {
        Err(PopupUnavailable::UnsupportedBackend)
    }
    fn attach(_: &Window, _: &Window, _: PopupKind) -> Result<(), PopupUnavailable> {
        Err(PopupUnavailable::UnsupportedBackend)
    }
    fn detach(_: &Window) {}
}
