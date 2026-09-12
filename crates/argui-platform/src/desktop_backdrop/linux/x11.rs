use super::{BackdropBackend, BackdropError, Request};
use argui_core::Rect;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, PropMode},
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

pub(in crate::desktop_backdrop) struct Backend {
    connection: RustConnection,
    window: u32,
    root: u32,
    atom: u32,
    available: bool,
}
#[cfg_attr(coverage_nightly, coverage(off))]
impl BackdropBackend for Backend {
    fn new(_: RawDisplayHandle, window: RawWindowHandle) -> Result<Self, BackdropError> {
        let window = match window {
            RawWindowHandle::Xlib(handle) => handle.window as u32,
            RawWindowHandle::Xcb(handle) => handle.window.get(),
            _ => return Err(BackdropError::Unsupported),
        };
        let (connection, screen) = x11rb::connect(None).map_err(error)?;
        let root = connection.setup().roots[screen].root;
        let atom = connection
            .intern_atom(false, b"_KDE_NET_WM_BLUR_BEHIND_REGION")
            .map_err(error)?
            .reply()
            .map_err(error)?
            .atom;
        connection
            .change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
            )
            .map_err(error)?
            .check()
            .map_err(error)?;
        let available = connection
            .list_properties(root)
            .map_err(error)?
            .reply()
            .map_err(error)?
            .atoms
            .contains(&atom);
        Ok(Self {
            connection,
            window,
            root,
            atom,
            available,
        })
    }
    fn available(&mut self) -> Result<bool, BackdropError> {
        while let Some(event) = self.connection.poll_for_event().map_err(error)? {
            if matches!(event, Event::PropertyNotify(event) if event.window == self.root && event.atom == self.atom)
            {
                self.available = self
                    .connection
                    .list_properties(self.root)
                    .map_err(error)?
                    .reply()
                    .map_err(error)?
                    .atoms
                    .contains(&self.atom);
            }
        }
        Ok(self.available)
    }
    fn apply(&mut self, regions: &[Rect], request: &Request) -> Result<(), BackdropError> {
        if regions.is_empty() {
            self.connection
                .delete_property(self.window, self.atom)
                .map_err(error)?
                .check()
                .map_err(error)?;
        } else {
            let data: Vec<_> = regions
                .iter()
                .flat_map(|rect| {
                    [
                        (rect.origin.x * request.scale).floor() as i32 as u32,
                        (rect.origin.y * request.scale).floor() as i32 as u32,
                        (rect.size.width * request.scale).ceil() as u32,
                        (rect.size.height * request.scale).ceil() as u32,
                    ]
                })
                .collect();
            self.connection
                .change_property32(
                    PropMode::REPLACE,
                    self.window,
                    self.atom,
                    AtomEnum::CARDINAL,
                    &data,
                )
                .map_err(error)?
                .check()
                .map_err(error)?;
        }
        self.connection.flush().map_err(error)
    }
}
#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for Backend {
    fn drop(&mut self) {
        let _ = self.connection.delete_property(self.window, self.atom);
        let _ = self.connection.flush();
    }
}
fn error(error: impl std::fmt::Display) -> BackdropError {
    BackdropError::Platform(error.to_string())
}
