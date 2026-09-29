//! Native presentation hints applied before mapping a client surface.

use crate::WindowConfig;
use winit::window::Window;

/// Prepares the unmapped `window` using its requested `config`.
/// Transparent X11 clients retain the top-left backing and clear new pixels to
/// zero alpha. Passive X11 clients declare that mapping does not request focus;
/// tool surfaces can also opt out of taskbar entries and desktop attention banners.
/// Call before the first mapping and when remapping a retained hidden window.
/// Other backends use their configured native attributes.
///
/// # Errors
/// Returns an error if an X11 connection or checked window-attribute request fails.
pub fn prepare_window_presentation(window: &Window, config: &WindowConfig) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if config.transparent
        || config.desktop_backdrop.is_some()
        || !config.focus_on_launch
        || config.skip_taskbar
    {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use x11rb::{
            connection::Connection,
            protocol::xproto::{
                AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, Gravity, PropMode,
            },
            wrapper::ConnectionExt as _,
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
        if config.transparent || config.desktop_backdrop.is_some() {
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
        }
        let atom = |name: &[u8]| {
            connection
                .intern_atom(false, name)
                .map_err(|error| error.to_string())?
                .reply()
                .map(|reply| reply.atom)
                .map_err(|error| error.to_string())
        };
        if !config.focus_on_launch {
            // Winit's X11 backend ignores WindowAttributes::active. EWMH defines
            // a zero user time as an explicit request for inactive presentation.
            connection
                .change_property32(
                    PropMode::REPLACE,
                    xid,
                    atom(b"_NET_WM_USER_TIME")?,
                    AtomEnum::CARDINAL,
                    &[0],
                )
                .map_err(|error| error.to_string())?
                .check()
                .map_err(|error| error.to_string())?;
        }
        if config.skip_taskbar {
            let state = atom(b"_NET_WM_STATE")?;
            let skip = atom(b"_NET_WM_STATE_SKIP_TASKBAR")?;
            let reply = connection
                .get_property(false, xid, state, AtomEnum::ATOM, 0, u32::MAX)
                .map_err(|error| error.to_string())?
                .reply()
                .map_err(|error| error.to_string())?;
            let mut states: Vec<_> = reply
                .value32()
                .map(|values| values.collect())
                .unwrap_or_default();
            if !states.contains(&skip) {
                states.push(skip);
                connection
                    .change_property32(PropMode::REPLACE, xid, state, AtomEnum::ATOM, &states)
                    .map_err(|error| error.to_string())?
                    .check()
                    .map_err(|error| error.to_string())?;
            }
        }
        connection.flush().map_err(|error| error.to_string())?;
    }
    #[cfg(not(target_os = "linux"))]
    let _ = (window, config);
    Ok(())
}
