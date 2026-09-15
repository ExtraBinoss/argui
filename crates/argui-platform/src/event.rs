#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// State transition of a physical pointer button.
pub enum ButtonState {
    /// The button became pressed.
    Pressed,
    /// The button became released.
    Released,
}

#[derive(Clone, Debug, PartialEq)]
/// Normalized lifecycle, input, and window events delivered by a platform host.
pub enum PlatformEvent {
    /// The native window or Web canvas is ready.
    Opened {
        /// Drawable width in physical pixels.
        width: u32,
        /// Drawable height in physical pixels.
        height: u32,
        /// Display scale factor.
        scale_factor: f64,
        /// Capabilities detected for the current backend.
        capabilities: crate::WindowCapabilities,
    },
    /// The platform temporarily suspended rendering.
    Suspended,
    /// Window visibility changed.
    VisibilityChanged(bool),
    /// The native window was closed.
    Closed,
    /// Drawable size changed.
    Resized {
        /// New drawable width in physical pixels.
        width: u32,
        /// New drawable height in physical pixels.
        height: u32,
    },
    /// Display scale factor changed.
    ScaleFactorChanged(f64),
    /// Safe-area insets changed.
    SafeAreaChanged(crate::Insets),
    /// Resolved system preferences changed.
    PreferencesChanged(crate::SystemPreferences),
    /// Pointer input occurred.
    Pointer(PointerEvent),
    /// Pointer scroll input occurred.
    PointerScrolled(ScrollDelta),
    /// Keyboard input occurred.
    Keyboard(KeyInput),
    /// Input-method composition changed.
    Ime(ImeInput),
    /// Window focus changed.
    Focused(bool),
    /// The host requested a frame.
    RedrawRequested,
    /// The user or platform requested window closure.
    CloseRequested,
    /// Native window creation failed.
    WindowCreationFailed(String),
}

impl PlatformEvent {
    /// Indicates whether handling this event should request another render.
    #[must_use]
    pub const fn requires_redraw(&self) -> bool {
        matches!(
            self,
            Self::Opened { .. }
                | Self::Resized { .. }
                | Self::ScaleFactorChanged(_)
                | Self::SafeAreaChanged(_)
                | Self::Focused(true)
                | Self::VisibilityChanged(true)
        )
    }

    /// Indicates whether this event represents a window close request.
    #[must_use]
    pub const fn closes_window(&self) -> bool {
        matches!(self, Self::CloseRequested | Self::WindowCreationFailed(_))
    }
}
pub use argui_core::{ImeInput, Key, KeyInput, KeyState, Modifiers, PointerEvent, ScrollDelta};
