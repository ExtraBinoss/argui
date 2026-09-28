//! Native pointer input regions, independent of the window's visual alpha.

use argui_core::Rect;
use winit::window::Window;

/// Reads the current X11 pointer position relative to `window` in UI pixels.
///
/// `ui_scale` is DPI multiplied by application zoom. Returns `None` for an
/// unsupported backend, invalid scale, or an unavailable native pointer.
pub fn window_pointer_position(window: &Window, ui_scale: f64) -> Option<argui_core::Point> {
    #[cfg(all(target_os = "linux", feature = "window-input-regions"))]
    return x11::pointer_position(window, ui_scale);
    #[cfg(not(all(target_os = "linux", feature = "window-input-regions")))]
    {
        let _ = (window, ui_scale);
        None
    }
}

/// Native pointer policy for one window. Rectangles use Argui UI logical units.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum WindowInputRegion {
    /// The entire client area receives pointer input.
    #[default]
    Full,
    /// The whole window lets pointer input reach windows behind it.
    PassThrough,
    /// The rectangle lets input through; the remainder receives input.
    Exclude(Rect),
}

/// Failure to install a native pointer input region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowInputRegionError {
    /// Geometry or scale was invalid or cannot fit native coordinates.
    InvalidGeometry,
    /// The window backend cannot provide the requested policy.
    Unsupported,
    /// The platform rejected an otherwise valid region.
    Platform(String),
}

impl std::fmt::Display for WindowInputRegionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGeometry => {
                formatter.write_str("window input region has invalid geometry")
            }
            Self::Unsupported => {
                formatter.write_str("window input regions are unsupported on this backend")
            }
            Self::Platform(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for WindowInputRegionError {}

/// Applies a pointer policy to a native window.
///
/// `window` owns the native surface, `region` is the policy to install, and
/// `ui_scale` converts Argui UI units to physical pixels (DPI multiplied by UI
/// zoom). The return value reports whether the native operation succeeded.
///
/// # Errors
/// Returns invalid geometry for non-finite, negative, or unrepresentable coordinates, unsupported
/// for a region-capability gap, or platform error when the native operation fails.
pub fn apply_window_input_region(
    window: &Window,
    region: &WindowInputRegion,
    ui_scale: f64,
) -> Result<(), WindowInputRegionError> {
    if !ui_scale.is_finite() || ui_scale <= 0.0 {
        return Err(WindowInputRegionError::InvalidGeometry);
    }
    if let WindowInputRegion::Exclude(rect) = region
        && (!rect.origin.x.is_finite()
            || !rect.origin.y.is_finite()
            || !rect.size.width.is_finite()
            || !rect.size.height.is_finite()
            || rect.size.width < 0.0
            || rect.size.height < 0.0)
    {
        return Err(WindowInputRegionError::InvalidGeometry);
    }
    #[cfg(all(target_os = "linux", feature = "window-input-regions"))]
    if let Some(result) = x11::apply(window, region, ui_scale) {
        return result;
    }
    match region {
        WindowInputRegion::Full | WindowInputRegion::PassThrough => window
            .set_cursor_hittest(matches!(region, WindowInputRegion::Full))
            .map_err(|error| WindowInputRegionError::Platform(error.to_string())),
        WindowInputRegion::Exclude(_) => Err(WindowInputRegionError::Unsupported),
    }
}

#[cfg(all(target_os = "linux", feature = "window-input-regions"))]
mod x11;
