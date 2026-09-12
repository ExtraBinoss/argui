use super::{BackdropBackend, BackdropError, Request};
use argui_core::{ColorScheme, Rect};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{
        DWMSBT_NONE, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
    },
    UI::Controls::MARGINS,
};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub(super) struct Backend {
    window: HWND,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl BackdropBackend for Backend {
    fn new(_: RawDisplayHandle, window: RawWindowHandle) -> Result<Self, BackdropError> {
        let RawWindowHandle::Win32(handle) = window else {
            return Err(BackdropError::Unsupported);
        };
        Ok(Self {
            window: HWND(handle.hwnd.get() as *mut _),
        })
    }
    fn available(&mut self) -> Result<bool, BackdropError> {
        Ok(true)
    }
    #[allow(unsafe_code)]
    fn apply(&mut self, regions: &[Rect], request: &Request) -> Result<(), BackdropError> {
        let material = if regions.is_empty() {
            DWMSBT_NONE
        } else {
            DWMSBT_TRANSIENTWINDOW
        };
        let dark: i32 = i32::from(request.scheme == ColorScheme::Dark);
        let margin = if regions.is_empty() { 0 } else { -1 };
        let margins = MARGINS {
            cxLeftWidth: margin,
            cxRightWidth: margin,
            cyTopHeight: margin,
            cyBottomHeight: margin,
        };
        // SAFETY: the HWND owner is retained by NativeBackdrop; argument pointers and byte
        // sizes match DWM's documented attributes. Calls run on the host event-loop thread.
        unsafe {
            DwmSetWindowAttribute(
                self.window,
                DWMWA_SYSTEMBACKDROP_TYPE,
                (&raw const material).cast(),
                size_of_val(&material) as u32,
            )
            .map_err(|error| BackdropError::Platform(error.to_string()))?;
            DwmSetWindowAttribute(
                self.window,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&raw const dark).cast(),
                size_of_val(&dark) as u32,
            )
            .map_err(|error| BackdropError::Platform(error.to_string()))?;
            DwmExtendFrameIntoClientArea(self.window, &margins)
                .map_err(|error| BackdropError::Platform(error.to_string()))?;
        }
        // DWM supplies Acrylic to the window; only the authored translucent pixels expose it.
        // Sidebar/Header are semantic hints and intentionally keep real desktop Acrylic here.
        Ok(())
    }
}
#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for Backend {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let material = DWMSBT_NONE;
        // SAFETY: NativeBackdrop drops this adapter before releasing the HWND owner.
        unsafe {
            let _ = DwmSetWindowAttribute(
                self.window,
                DWMWA_SYSTEMBACKDROP_TYPE,
                (&raw const material).cast(),
                size_of_val(&material) as u32,
            );
            let _ = DwmExtendFrameIntoClientArea(self.window, &MARGINS::default());
        }
    }
}
