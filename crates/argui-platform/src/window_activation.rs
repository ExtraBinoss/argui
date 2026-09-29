//! Restores application windows while retaining the chosen activation.

use winit::window::Window;

/// Restores and raises `window`, retaining `source` as the activated application window.
/// X11 restores minimized peers through the WM, then reactivates `source`;
/// already-visible peers are raised without activation.
/// Returns `Ok(false)` on backends without independent stacking, including Wayland.
/// The caller must exclude explicitly hidden or closed windows before calling this.
///
/// # Errors
/// Returns an error if the native handle or stacking request fails.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn restore_and_raise_window(window: &Window, source: &Window) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use x11rb::{
            connection::Connection,
            protocol::xproto::{
                ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, StackMode,
            },
        };
        let xid = match window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        {
            RawWindowHandle::Xlib(handle) => handle.window as u32,
            RawWindowHandle::Xcb(handle) => handle.window.get(),
            _ => return Ok(false),
        };
        let active_xid = match source
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        {
            RawWindowHandle::Xlib(handle) => handle.window as u32,
            RawWindowHandle::Xcb(handle) => handle.window.get(),
            _ => return Ok(false),
        };
        let (connection, screen) = x11rb::connect(None).map_err(|error| error.to_string())?;
        let root = connection.setup().roots[screen].root;
        let minimized = window.is_minimized() == Some(true);
        // Winit's X11 set_minimized(false) sends _NET_ACTIVE_WINDOW even for
        // visible windows, causing focus ping-pong if used unconditionally.
        // MapWindow cannot restore WMs that keep minimized clients mapped.
        let active_atom = minimized
            .then(|| {
                connection
                    .intern_atom(false, b"_NET_ACTIVE_WINDOW")
                    .map_err(|error| error.to_string())?
                    .reply()
                    .map(|reply| reply.atom)
                    .map_err(|error| error.to_string())
            })
            .transpose()?;
        let activate = |target, current, atom| {
            connection
                .send_event(
                    false,
                    root,
                    EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                    ClientMessageEvent::new(
                        32,
                        target,
                        atom,
                        [1, x11rb::CURRENT_TIME, current, 0, 0],
                    ),
                )
                .map_err(|error| error.to_string())?
                .check()
                .map_err(|error| error.to_string())
        };
        if let Some(atom) = active_atom {
            activate(xid, active_xid, atom)?;
        }
        connection
            .configure_window(xid, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE))
            .map_err(|error| error.to_string())?
            .check()
            .map_err(|error| error.to_string())?;
        if let Some(atom) = active_atom {
            activate(active_xid, xid, atom)?;
        }
        connection.flush().map_err(|error| error.to_string())?;
        Ok(true)
    }
    #[cfg(all(target_os = "windows", feature = "native-popups"))]
    {
        let _ = source;
        use windows::Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{
                HWND_TOP, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
                ShowWindow,
            },
        };
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let RawWindowHandle::Win32(handle) = window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        else {
            return Ok(false);
        };
        // SAFETY: Winit owns the live HWND throughout this synchronous call;
        // only stacking changes, with no size, position, focus, or pointer changes.
        #[allow(unsafe_code)]
        unsafe {
            if window.is_minimized() == Some(true) {
                let _ = ShowWindow(HWND(handle.hwnd.get() as *mut _), SW_SHOWNOACTIVATE);
            }
            SetWindowPos(
                HWND(handle.hwnd.get() as *mut _),
                Some(HWND_TOP),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(true)
    }
    #[cfg(all(target_os = "macos", feature = "native-popups"))]
    {
        let _ = source;
        use objc2_app_kit::NSView;
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let RawWindowHandle::AppKit(handle) = window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        else {
            return Ok(false);
        };
        // SAFETY: Winit's live AppKit handle is an NSView, borrowed only on
        // the main event-loop thread. Its window is retained for this call.
        #[allow(unsafe_code)]
        let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
        let native = view.window().ok_or("AppKit window is unavailable")?;
        window.set_minimized(false);
        native.orderFrontRegardless();
        Ok(true)
    }
    #[cfg(not(any(
        target_os = "linux",
        all(
            feature = "native-popups",
            any(target_os = "windows", target_os = "macos")
        )
    )))]
    {
        let _ = (window, source);
        Ok(false)
    }
}
