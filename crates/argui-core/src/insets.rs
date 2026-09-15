use crate::Rect;

/// Distances from a window edge to the region kept clear for system UI.
///
/// Values use logical pixels, matching Argui layout coordinates. The platform
/// adapter decides which system regions to include.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// Distance from the top edge.
    pub top: f32,
    /// Distance from the right edge.
    pub right: f32,
    /// Distance from the bottom edge.
    pub bottom: f32,
    /// Distance from the left edge.
    pub left: f32,
}

impl Insets {
    /// Insets with all edges equal to zero.
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    /// Creates insets in top, right, bottom, left order.
    #[must_use]
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Derive insets when both physical rectangles describe non-empty areas.
    ///
    /// Platform APIs can report an empty safe rectangle while a native window
    /// is being created. Treat that as unavailable until a later update rather
    /// than interpreting it as the whole window being obscured.
    /// * `scale_factor` — physical-to-logical pixel scale.
    #[must_use]
    pub fn try_from_physical_rects(window: Rect, safe: Rect, scale_factor: f32) -> Option<Self> {
        let coordinates = [
            window.origin.x,
            window.origin.y,
            window.size.width,
            window.size.height,
            safe.origin.x,
            safe.origin.y,
            safe.size.width,
            safe.size.height,
        ];
        if coordinates.iter().any(|value| !value.is_finite())
            || window.size.width <= 0.0
            || window.size.height <= 0.0
            || safe.size.width <= 0.0
            || safe.size.height <= 0.0
        {
            return None;
        }

        let safe = window.intersection(safe)?;
        Some(Self::from_physical_rects(window, safe, scale_factor))
    }

    /// Derive logical insets from a physical window rectangle and its safe
    /// subrectangle. Coordinates in both rectangles use the same screen space.
    /// * `scale_factor` — physical-to-logical pixel scale.
    #[must_use]
    pub fn from_physical_rects(window: Rect, safe: Rect, scale_factor: f32) -> Self {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let width = window.size.width.max(0.0);
        let height = window.size.height.max(0.0);
        let safe_width = safe.size.width.max(0.0);
        let safe_height = safe.size.height.max(0.0);
        let left = (safe.origin.x - window.origin.x).clamp(0.0, width);
        let right = (safe.origin.x + safe_width - window.origin.x).clamp(left, width);
        let top = (safe.origin.y - window.origin.y).clamp(0.0, height);
        let bottom = (safe.origin.y + safe_height - window.origin.y).clamp(top, height);
        Self::new(
            top / scale_factor,
            (width - right) / scale_factor,
            (height - bottom) / scale_factor,
            left / scale_factor,
        )
    }

    #[must_use]
    /// Returns the combined left and right inset.
    pub const fn horizontal(self) -> f32 {
        self.left + self.right
    }

    #[must_use]
    /// Returns the combined top and bottom inset.
    pub const fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}
