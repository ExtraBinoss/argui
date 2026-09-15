#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContainerScopeId(&'static str);

impl ContainerScopeId {
    /// Creates a container scope identifier from a stable application-defined name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the name used to create this scope identifier.
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
    /// Matches containers whose width is at least `value`.
    /// * `scope` — container scope whose width is tested.
    #[must_use]
    pub const fn min_width(scope: ContainerScopeId, value: f32) -> Self {
        Self::MinWidth { scope, value }
    }

    /// Matches containers whose width is less than `value`.
    /// * `scope` — container scope whose width is tested.
    #[must_use]
    pub const fn max_width(scope: ContainerScopeId, value: f32) -> Self {
        Self::MaxWidth { scope, value }
    }

    /// Matches containers whose height is at least `value`.
    /// * `scope` — container scope whose height is tested.
    #[must_use]
    pub const fn min_height(scope: ContainerScopeId, value: f32) -> Self {
        Self::MinHeight { scope, value }
    }

    /// Matches containers whose height is less than `value`.
    /// * `scope` — container scope whose height is tested.
    #[must_use]
    pub const fn max_height(scope: ContainerScopeId, value: f32) -> Self {
        Self::MaxHeight { scope, value }
    }

    /// Matches containers that are landscape or square.
    /// * `scope` — container scope whose aspect is tested.
    #[must_use]
    pub const fn landscape(scope: ContainerScopeId) -> Self {
        Self::Landscape(scope)
    }

    /// Matches containers that are taller than they are wide.
    /// * `scope` — container scope whose aspect is tested.
    #[must_use]
    pub const fn portrait(scope: ContainerScopeId) -> Self {
        Self::Portrait(scope)
    }

    /// Returns the scope whose dimensions this query examines.
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

    /// Tests this query against a container's width and height in logical pixels.
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
