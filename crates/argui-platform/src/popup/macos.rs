use super::{PopupBackend, PopupKind, PopupUnavailable};
use argui_core::{Point, Rect, Size};
use objc2::rc::Retained;
use objc2_app_kit::{NSView, NSWindow, NSWindowOrderingMode};
use winit::{
    platform::macos::WindowAttributesExtMacOS,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes},
};

pub(super) struct Backend;

#[allow(unsafe_code)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn native(window: &Window) -> Result<Retained<NSWindow>, PopupUnavailable> {
    let Ok(RawWindowHandle::AppKit(handle)) = window.window_handle().map(|handle| handle.as_raw())
    else {
        return Err(PopupUnavailable::UnsupportedBackend);
    };
    // SAFETY: winit's live AppKit handle is an NSView, borrowed only on the
    // event-loop (main) thread. Retaining its window keeps the returned object alive.
    let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
    view.window().ok_or(PopupUnavailable::UnsupportedBackend)
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PopupBackend for Backend {
    fn work_area(parent: &Window) -> Result<Rect, PopupUnavailable> {
        let screen = native(parent)?
            .screen()
            .ok_or(PopupUnavailable::UnknownGeometry)?;
        let frame = screen.frame();
        let visible = screen.visibleFrame();
        let monitor = parent
            .current_monitor()
            .ok_or(PopupUnavailable::UnknownGeometry)?;
        let origin = monitor.position();
        let scale = monitor.scale_factor();
        // AppKit's screen coordinates start at the bottom left; winit uses top left.
        Ok(Rect::new(
            Point::new(
                origin.x as f32 + ((visible.origin.x - frame.origin.x) * scale) as f32,
                origin.y as f32
                    + ((frame.origin.y + frame.size.height
                        - visible.origin.y
                        - visible.size.height)
                        * scale) as f32,
            ),
            Size::new(
                (visible.size.width * scale) as f32,
                (visible.size.height * scale) as f32,
            ),
        ))
    }

    fn attributes(
        parent: &Window,
        _: PopupKind,
        attributes: WindowAttributes,
    ) -> Result<WindowAttributes, PopupUnavailable> {
        native(parent)?;
        Ok(attributes
            .with_has_shadow(true)
            .with_accepts_first_mouse(true))
    }

    #[allow(unsafe_code)]
    fn attach(parent: &Window, window: &Window, _: PopupKind) -> Result<(), PopupUnavailable> {
        let parent = native(parent)?;
        let window = native(window)?;
        window.setHidesOnDeactivate(true);
        // SAFETY: both live, retained windows were created on this main event-loop
        // thread. The new popup cannot be an ancestor of its existing parent.
        unsafe {
            parent.addChildWindow_ordered(&window, NSWindowOrderingMode::Above);
        }
        Ok(())
    }

    fn detach(window: &Window) {
        if let Ok(window) = native(window)
            && let Some(parent) = window.parentWindow()
        {
            parent.removeChildWindow(&window);
        }
    }
}
