use super::PortalTarget;

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
}

impl Portal {
    #[must_use]
    pub const fn new(layer: WindowLayer, target: PortalTarget) -> Self {
        Self {
            layer,
            target,
            dismiss: DismissPolicy::Manual,
        }
    }

    #[must_use]
    pub const fn dismiss(mut self, dismiss: DismissPolicy) -> Self {
        self.dismiss = dismiss;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DismissPolicy {
    #[default]
    Manual,
    OutsidePointer,
}
