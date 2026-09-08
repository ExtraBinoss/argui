#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEvent {
    Opened {
        width: u32,
        height: u32,
        scale_factor: f64,
        capabilities: crate::WindowCapabilities,
    },
    Suspended,
    VisibilityChanged(bool),
    Closed,
    Resized {
        width: u32,
        height: u32,
    },
    ScaleFactorChanged(f64),
    PreferencesChanged(crate::SystemPreferences),
    Pointer(PointerEvent),
    PointerScrolled(ScrollDelta),
    Keyboard(KeyInput),
    Ime(ImeInput),
    Focused(bool),
    RedrawRequested,
    CloseRequested,
    WindowCreationFailed(String),
}

impl PlatformEvent {
    #[must_use]
    pub const fn requires_redraw(&self) -> bool {
        matches!(
            self,
            Self::Opened { .. } | Self::Resized { .. } | Self::ScaleFactorChanged(_)
        )
    }

    #[must_use]
    pub const fn closes_window(&self) -> bool {
        matches!(self, Self::CloseRequested | Self::WindowCreationFailed(_))
    }
}
pub use argui_core::{ImeInput, Key, KeyInput, KeyState, Modifiers, PointerEvent, ScrollDelta};
