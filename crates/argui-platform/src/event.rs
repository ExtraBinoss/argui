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
