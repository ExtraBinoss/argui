use argui_core::{Point, Rect, Size};
use winit::{
    platform::x11::{WindowAttributesExtX11, WindowType},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowAttributes},
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConnectionExt, PropMode},
    wrapper::ConnectionExt as _,
};

use super::{PopupBackend, PopupKind, PopupUnavailable};

pub(super) struct Backend;

#[cfg_attr(coverage_nightly, coverage(off))]
fn xid(window: &Window) -> Result<u32, PopupUnavailable> {
    match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Xlib(handle)) => Ok(handle.window as u32),
        Ok(RawWindowHandle::Xcb(handle)) => Ok(handle.window.get()),
        _ => Err(PopupUnavailable::UnsupportedBackend),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn platform(error: impl std::fmt::Display) -> PopupUnavailable {
    PopupUnavailable::Platform(error.to_string())
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PopupBackend for Backend {
    fn work_area(parent: &Window) -> Result<Rect, PopupUnavailable> {
        let parent_id = xid(parent)?;
        let monitor = parent
            .current_monitor()
            .ok_or(PopupUnavailable::UnknownGeometry)?;
        let position = monitor.position();
        let size = monitor.size();
        let monitor_bounds = Rect::new(
            Point::new(position.x as f32, position.y as f32),
            Size::new(size.width as f32, size.height as f32),
        );
        let (connection, _) = x11rb::connect(None).map_err(platform)?;
        let root = connection
            .query_tree(parent_id)
            .map_err(platform)?
            .reply()
            .map_err(platform)?
            .root;
        let property = |name: &[u8]| -> Result<Vec<u32>, PopupUnavailable> {
            let atom = connection
                .intern_atom(false, name)
                .map_err(platform)?
                .reply()
                .map_err(platform)?
                .atom;
            let reply = connection
                .get_property(false, root, atom, AtomEnum::CARDINAL, 0, 4096)
                .map_err(platform)?
                .reply()
                .map_err(platform)?;
            Ok(reply
                .value32()
                .map(|values| values.collect())
                .unwrap_or_default())
        };
        let desktop = property(b"_NET_CURRENT_DESKTOP")?
            .first()
            .copied()
            .unwrap_or(0) as usize;
        let areas = property(b"_NET_WORKAREA")?;
        // A bare X server has no reserved desktop regions; the monitor is its work area.
        let Some(area) = areas.as_chunks::<4>().0.get(desktop) else {
            return Ok(monitor_bounds);
        };
        let bounds = Rect::new(
            Point::new(area[0] as i32 as f32, area[1] as i32 as f32),
            Size::new(area[2] as f32, area[3] as f32),
        );
        monitor_bounds
            .intersection(bounds)
            .ok_or(PopupUnavailable::UnknownGeometry)
    }

    fn attributes(
        parent: &Window,
        kind: PopupKind,
        attributes: WindowAttributes,
    ) -> Result<WindowAttributes, PopupUnavailable> {
        xid(parent)?;
        let role = match kind {
            PopupKind::Tooltip => WindowType::Tooltip,
            PopupKind::Menu => WindowType::PopupMenu,
            PopupKind::Popover => WindowType::Combo,
        };
        Ok(attributes
            .with_override_redirect(true)
            .with_x11_window_type(vec![role]))
    }

    fn attach(parent: &Window, window: &Window, _kind: PopupKind) -> Result<(), PopupUnavailable> {
        let (connection, _) = x11rb::connect(None).map_err(platform)?;
        connection
            .change_property32(
                PropMode::REPLACE,
                xid(window)?,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                &[xid(parent)?],
            )
            .map_err(platform)?
            .check()
            .map_err(platform)?;
        connection.flush().map_err(platform)
    }

    fn detach(_window: &Window) {}
    fn focus(window: &Window) {
        if let Ok(id) = xid(window)
            && let Ok((connection, _)) = x11rb::connect(None)
        {
            let _ = connection.set_input_focus(
                x11rb::protocol::xproto::InputFocus::PARENT,
                id,
                x11rb::CURRENT_TIME,
            );
            let _ = connection.flush();
        }
    }
}
