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
    #[must_use]
    pub const fn new(layer: WindowLayer, target: PortalTarget) -> Self {
        Self {
            layer,
            target,
            dismiss: DismissPolicy::Manual,
            surface: None,
        }
    }

    #[must_use]
    pub const fn dismiss(mut self, dismiss: DismissPolicy) -> Self {
        self.dismiss = dismiss;
        self
    }

    #[must_use]
    pub const fn surface(mut self, surface: OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DismissPolicy {
    #[default]
    Manual,
    OutsidePointer,
}
