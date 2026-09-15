use argui_core::Color;

/// A background behind the application, with an explicit paint fallback.
/// Keep ancestors transparent; this replaces the element's background, not its content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopBackdrop {
    /// Request native blur; when false, only the fallback colors are painted.
    pub blur: bool,
    pub tint: Color,
    pub inactive_tint: Color,
    pub fallback: Color,
    pub inactive_fallback: Color,
}

impl DesktopBackdrop {
    /// Creates a backdrop with matching active and inactive colors.
    ///
    /// * `tint` — color tint requested while the compositor blur is available.
    /// * `fallback` — color painted when compositor blur is unavailable.
    #[must_use]
    pub const fn new(tint: Color, fallback: Color) -> Self {
        Self {
            blur: true,
            tint,
            inactive_tint: tint,
            fallback,
            inactive_fallback: fallback,
        }
    }

    /// Disable the compositor effect while keeping the active and inactive fallback colors.
    ///
    /// * `enabled` — whether compositor blur is requested.
    #[must_use]
    pub const fn blur(mut self, enabled: bool) -> Self {
        self.blur = enabled;
        self
    }

    /// Sets the tint used while the window is inactive and blur is available.
    #[must_use]
    pub const fn inactive_tint(mut self, tint: Color) -> Self {
        self.inactive_tint = tint;
        self
    }

    /// Sets the fallback color used while the window is inactive.
    #[must_use]
    pub const fn inactive_fallback(mut self, color: Color) -> Self {
        self.inactive_fallback = color;
        self
    }

    /// Resolves the backdrop color for the current compositor and focus state.
    #[must_use]
    pub const fn color(self, state: DesktopBackdropState) -> Color {
        if self.blur && state.available {
            if state.focused {
                self.tint
            } else {
                self.inactive_tint
            }
        } else if state.focused {
            self.fallback
        } else {
            self.inactive_fallback
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
    /// Sets the desktop backdrop painted behind this element's content.
    #[must_use]
    pub fn desktop_backdrop(mut self, backdrop: DesktopBackdrop) -> Self {
        self.desktop_backdrop = Some(backdrop);
        self
    }
}

impl crate::UiTree {
    /// Returns the current desktop backdrop capabilities used to resolve materials.
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
