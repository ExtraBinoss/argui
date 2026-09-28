//! Native drawable preservation for transparent client surfaces.

use winit::window::Window;

/// Prepares `window` to preserve its already-presented pixels during resizing.
/// Transparent X11 clients retain the top-left backing and clear new pixels to
/// zero alpha. Other backends keep their compositor-managed presentation policy.
/// Call this after creation, before the first visible frame.
/// `transparent` is the native window's requested transparency flag.
///
/// # Errors
/// Returns an error if an X11 connection or checked window-attribute request fails.
pub fn prepare_window_presentation(window: &Window, transparent: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if transparent {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use x11rb::{
            connection::Connection,
            protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt as _, Gravity},
        };
        let xid = match window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        {
            RawWindowHandle::Xlib(handle) => handle.window as u32,
            RawWindowHandle::Xcb(handle) => handle.window.get(),
            _ => return Ok(()),
        };
        let (connection, _) = x11rb::connect(None).map_err(|error| error.to_string())?;
        connection
            .change_window_attributes(
                xid,
                &ChangeWindowAttributesAux::new()
                    .bit_gravity(Gravity::NORTH_WEST)
                    .background_pixel(0),
            )
            .map_err(|error| error.to_string())?
            .check()
            .map_err(|error| error.to_string())?;
        connection.flush().map_err(|error| error.to_string())?;
    }
    #[cfg(not(target_os = "linux"))]
    let _ = (window, transparent);
    Ok(())
}
