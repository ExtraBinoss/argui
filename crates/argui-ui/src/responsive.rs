#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContainerScopeId(&'static str);

impl ContainerScopeId {
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContainerQuery {
    MinWidth { scope: ContainerScopeId, value: f32 },
    MaxWidth { scope: ContainerScopeId, value: f32 },
    MinHeight { scope: ContainerScopeId, value: f32 },
    MaxHeight { scope: ContainerScopeId, value: f32 },
    Landscape(ContainerScopeId),
    Portrait(ContainerScopeId),
}

impl ContainerQuery {
    #[must_use]
    pub const fn min_width(scope: ContainerScopeId, value: f32) -> Self {
        Self::MinWidth { scope, value }
    }

    #[must_use]
    pub const fn max_width(scope: ContainerScopeId, value: f32) -> Self {
        Self::MaxWidth { scope, value }
    }

    #[must_use]
    pub const fn min_height(scope: ContainerScopeId, value: f32) -> Self {
        Self::MinHeight { scope, value }
    }

    #[must_use]
    pub const fn max_height(scope: ContainerScopeId, value: f32) -> Self {
        Self::MaxHeight { scope, value }
    }

    #[must_use]
    pub const fn landscape(scope: ContainerScopeId) -> Self {
        Self::Landscape(scope)
    }

    #[must_use]
    pub const fn portrait(scope: ContainerScopeId) -> Self {
        Self::Portrait(scope)
    }

    #[must_use]
    pub const fn scope(self) -> ContainerScopeId {
        match self {
            Self::MinWidth { scope, .. }
            | Self::MaxWidth { scope, .. }
            | Self::MinHeight { scope, .. }
            | Self::MaxHeight { scope, .. }
            | Self::Landscape(scope)
            | Self::Portrait(scope) => scope,
        }
    }

    #[must_use]
    pub fn matches(self, width: f32, height: f32) -> bool {
        match self {
            Self::MinWidth { value, .. } => width >= value,
            Self::MaxWidth { value, .. } => width < value,
            Self::MinHeight { value, .. } => height >= value,
            Self::MaxHeight { value, .. } => height < value,
            Self::Landscape(_) => width >= height,
            Self::Portrait(_) => width < height,
        }
    }
}

impl crate::Element {
    pub(crate) fn has_container_queries(&self) -> bool {
        self.conditional_styles.has_container_queries()
            || self.scroll.as_ref().is_some_and(|config| {
                config.scrollbar.as_ref().is_some_and(|scrollbar| {
                    scrollbar.track.has_container_queries()
                        || scrollbar.thumb.has_container_queries()
                })
            })
    }
}
