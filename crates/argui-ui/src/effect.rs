use argui_paint::LayerStyle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectScope {
    WholeElement,
    Background,
    Border,
    Content,
    Text,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScopedEffect {
    pub scope: EffectScope,
    pub layer: LayerStyle,
}

impl ScopedEffect {
    #[must_use]
    pub const fn new(scope: EffectScope, layer: LayerStyle) -> Self {
        Self { scope, layer }
    }
}
