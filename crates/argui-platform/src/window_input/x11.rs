use argui_core::Rect;
use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};
use x11rb::{
    connection::Connection,
    protocol::{
        shape::{ConnectionExt as _, SK, SO},
        xproto::{ClipOrdering, Rectangle},
    },
};

use super::{WindowInputRegion, WindowInputRegionError};

/// Installs a checked input shape for an X11 window; returns `None` on another backend.
/// `scale` converts UI logical coordinates to physical X11 coordinates.
pub(super) fn apply(
    window: &Window,
    region: &WindowInputRegion,
    scale: f64,
) -> Option<Result<(), WindowInputRegionError>> {
    let xid = match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Xlib(handle)) => handle.window as u32,
        Ok(RawWindowHandle::Xcb(handle)) => handle.window.get(),
        _ => return None,
    };
    Some(apply_x11(window, xid, region, scale))
}

/// Sends one X Shape request and waits for the server's result.
/// `xid` identifies the window and `scale` maps UI units to physical pixels.
fn apply_x11(
    window: &Window,
    xid: u32,
    region: &WindowInputRegion,
    scale: f64,
) -> Result<(), WindowInputRegionError> {
    let size = window.inner_size();
    let rectangles = match region {
        WindowInputRegion::Full => vec![Rectangle {
            x: 0,
            y: 0,
            width: size
                .width
                .try_into()
                .map_err(|_| WindowInputRegionError::InvalidGeometry)?,
            height: size
                .height
                .try_into()
                .map_err(|_| WindowInputRegionError::InvalidGeometry)?,
        }],
        WindowInputRegion::PassThrough => Vec::new(),
        WindowInputRegion::Exclude(hole) => {
            outside_rectangles(size.width, size.height, *hole, scale)?
        }
    };
    let (connection, _) = x11rb::connect(None)
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))?;
    connection
        .shape_query_version()
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))?
        .reply()
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))?;
    connection
        .shape_rectangles(
            SO::SET,
            SK::INPUT,
            ClipOrdering::UNSORTED,
            xid,
            0,
            0,
            &rectangles,
        )
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))?
        .check()
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))?;
    connection
        .flush()
        .map_err(|error| WindowInputRegionError::Platform(error.to_string()))
}

/// Builds the four physical rectangles surrounding a UI-logical hole.
/// `width` and `height` are the drawable extent, and `scale` is DPI times UI zoom.
/// Returns nonempty X11 input rectangles.
///
/// # Errors
/// Returns invalid geometry if the extent or scaled coordinates exceed X11 bounds.
pub(super) fn outside_rectangles(
    width: u32,
    height: u32,
    hole: Rect,
    scale: f64,
) -> Result<Vec<Rectangle>, WindowInputRegionError> {
    if width > i16::MAX as u32 || height > i16::MAX as u32 || !scale.is_finite() || scale <= 0.0 {
        return Err(WindowInputRegionError::InvalidGeometry);
    }
    let edge = |value: f64, ceil: bool, limit: u32| -> Result<u32, WindowInputRegionError> {
        if !value.is_finite() {
            return Err(WindowInputRegionError::InvalidGeometry);
        }
        Ok((if ceil { value.ceil() } else { value.floor() }).clamp(0.0, f64::from(limit)) as u32)
    };
    let left = edge(f64::from(hole.origin.x) * scale, false, width)?;
    let top = edge(f64::from(hole.origin.y) * scale, false, height)?;
    let right = edge(
        f64::from(hole.origin.x + hole.size.width) * scale,
        true,
        width,
    )?;
    let bottom = edge(
        f64::from(hole.origin.y + hole.size.height) * scale,
        true,
        height,
    )?;
    let mut output = Vec::with_capacity(4);
    let mut push = |x: u32, y: u32, w: u32, h: u32| {
        if w > 0 && h > 0 {
            output.push(Rectangle {
                x: x as i16,
                y: y as i16,
                width: w as u16,
                height: h as u16,
            });
        }
    };
    push(0, 0, width, top);
    push(0, top, left, bottom.saturating_sub(top));
    push(
        right,
        top,
        width.saturating_sub(right),
        bottom.saturating_sub(top),
    );
    push(0, bottom, width, height.saturating_sub(bottom));
    Ok(output)
}
