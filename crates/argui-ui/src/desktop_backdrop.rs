use argui_core::Color;

/// A background behind the application, with an explicit paint fallback.
/// Keep ancestors transparent; this replaces the element's background, not its content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopBackdrop {
    pub tint: Color,
    pub inactive_tint: Color,
    pub fallback: Color,
    pub inactive_fallback: Color,
}

impl DesktopBackdrop {
    #[must_use]
    pub const fn new(tint: Color, fallback: Color) -> Self {
        Self {
            tint,
            inactive_tint: tint,
            fallback,
            inactive_fallback: fallback,
        }
    }

    #[must_use]
    pub const fn inactive_tint(mut self, tint: Color) -> Self {
        self.inactive_tint = tint;
        self
    }

    #[must_use]
    pub const fn inactive_fallback(mut self, color: Color) -> Self {
        self.inactive_fallback = color;
        self
    }

    #[must_use]
    pub const fn color(self, state: DesktopBackdropState) -> Color {
        if !state.available && !state.focused {
            self.inactive_fallback
        } else if !state.available {
            self.fallback
        } else if state.focused {
            self.tint
        } else {
            self.inactive_tint
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopBackdropState {
    pub available: bool,
    pub focused: bool,
}

impl Default for DesktopBackdropState {
    fn default() -> Self {
        Self {
            available: false,
            focused: true,
        }
    }
}

impl crate::Element {
    #[must_use]
    pub fn desktop_backdrop(mut self, backdrop: DesktopBackdrop) -> Self {
        self.desktop_backdrop = Some(backdrop);
        self
    }
}

impl crate::UiTree {
    #[must_use]
    pub const fn desktop_backdrop_state(&self) -> DesktopBackdropState {
        self.desktop_backdrop_state
    }

    /// Returns whether a repaint is needed. Does not invalidate layout or editor state.
    pub fn set_desktop_backdrop_state(&mut self, state: DesktopBackdropState) -> bool {
        let changed = self.desktop_backdrop_state != state;
        self.desktop_backdrop_state = state;
        changed
    }
}
