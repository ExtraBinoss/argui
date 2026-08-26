#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEvent {
    Opened {
        width: u32,
        height: u32,
        scale_factor: f64,
    },
    Suspended,
    Resized {
        width: u32,
        height: u32,
    },
    ScaleFactorChanged(f64),
    PointerMoved {
        x: f32,
        y: f32,
    },
    PointerEntered,
    PointerLeft,
    PointerButton {
        button: PointerButton,
        state: ButtonState,
    },
    PointerScrolled(ScrollDelta),
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
pub use argui_core::ScrollDelta;
