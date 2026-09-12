use super::{PopupBackend, PopupKind, PopupUnavailable};
use argui_core::{Point, Rect, Size};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_CHILD, WS_EX_APPWINDOW,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    },
};
use winit::{
    platform::windows::WindowAttributesExtWindows,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes},
};

pub(super) struct Backend;

#[cfg_attr(coverage_nightly, coverage(off))]
fn hwnd(window: &Window) -> Result<HWND, PopupUnavailable> {
    match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Win32(handle)) => Ok(HWND(handle.hwnd.get() as *mut _)),
        _ => Err(PopupUnavailable::UnsupportedBackend),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PopupBackend for Backend {
    #[allow(unsafe_code)]
    fn work_area(parent: &Window) -> Result<Rect, PopupUnavailable> {
        let handle = hwnd(parent)?;
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // SAFETY: the borrowed winit window owns this live HWND throughout the calls;
        // MONITORINFO has the required size and writable storage for the synchronous API.
        unsafe {
            let monitor = MonitorFromWindow(handle, MONITOR_DEFAULTTONEAREST);
            if !GetMonitorInfoW(monitor, &mut info).as_bool() {
                return Err(PopupUnavailable::UnknownGeometry);
            }
        }
        let area = info.rcWork;
        Ok(Rect::new(
            Point::new(area.left as f32, area.top as f32),
            Size::new(
                (area.right - area.left) as f32,
                (area.bottom - area.top) as f32,
            ),
        ))
    }

    fn attributes(
        parent: &Window,
        _: PopupKind,
        attributes: WindowAttributes,
    ) -> Result<WindowAttributes, PopupUnavailable> {
        Ok(attributes
            .with_owner_window(hwnd(parent)?.0 as isize)
            .with_skip_taskbar(true))
    }

    #[allow(unsafe_code)]
    fn attach(_: &Window, window: &Window, kind: PopupKind) -> Result<(), PopupUnavailable> {
        let handle = hwnd(window)?;
        // SAFETY: a live window created on this event-loop thread owns the handle.
        // Only popup/activation flags change; no pointers or callbacks are installed.
        unsafe {
            let style = GetWindowLongPtrW(handle, GWL_STYLE);
            SetWindowLongPtrW(
                handle,
                GWL_STYLE,
                (style & !(WS_CHILD.0 as isize)) | WS_POPUP.0 as isize,
            );
            let style = GetWindowLongPtrW(handle, GWL_EXSTYLE);
            let mut style = (style & !(WS_EX_APPWINDOW.0 as isize)) | WS_EX_TOOLWINDOW.0 as isize;
            if kind == PopupKind::Tooltip {
                style |= WS_EX_NOACTIVATE.0 as isize;
            }
            SetWindowLongPtrW(handle, GWL_EXSTYLE, style);
            if GetWindowLongPtrW(handle, GWL_EXSTYLE) != style {
                return Err(PopupUnavailable::Platform(
                    "Windows rejected popup styles".into(),
                ));
            }
        }
        Ok(())
    }

    fn detach(_: &Window) {}
}
