use super::PortalTarget;

/// Where an anchored overlay is presented. Native presentation always has an in-window fallback.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlaySurface {
    #[default]
    InWindow,
    PreferNative,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum WindowLayer {
    Background,
    #[default]
    Content,
    Floating,
    Popover,
    Modal,
    Debug,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Portal {
    pub layer: WindowLayer,
    pub target: PortalTarget,
    pub dismiss: DismissPolicy,
    /// None inherits the nearest enclosing portal's preference.
    pub surface: Option<OverlaySurface>,
}

impl Portal {
    /// Creates a portal at the given window layer and target.
    ///
    /// * `layer` — stacking layer used for the portal.
    /// * `target` — layout, anchor, rectangle, or viewport placement.
    #[must_use]
    pub const fn new(layer: WindowLayer, target: PortalTarget) -> Self {
        Self {
            layer,
            target,
            dismiss: DismissPolicy::Manual,
            surface: None,
        }
    }

    /// Sets how the portal can be dismissed.
    /// * `dismiss` — dismissal policy applied to the portal.
    #[must_use]
    pub const fn dismiss(mut self, dismiss: DismissPolicy) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// Sets the preferred surface used to present this portal.
    #[must_use]
    pub const fn surface(mut self, surface: OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DismissPolicy {
    /// Dismissal is controlled by application state.
    #[default]
    Manual,
    /// A press outside the portal requests dismissal.
    OutsidePointer,
    /// Escape requests dismissal after key listeners accept it.
    Escape,
    /// Either an outside press or Escape requests dismissal.
    OutsidePointerOrEscape,
    /// Leaving both the portal and its anchor by more than a small pointer gap requests dismissal; Escape also dismisses.
    OutsideHoverOrEscape,
}
