use argui_paint::ShaderEffectId;

use crate::{RendererError, effect::validated_custom_source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectShader {
    pub id: ShaderEffectId,
    pub wgsl: &'static str,
}

impl EffectShader {
    #[must_use]
    pub const fn new(id: ShaderEffectId, wgsl: &'static str) -> Self {
        Self { id, wgsl }
    }

    pub fn validate(self) -> Result<(), RendererError> {
        validated_custom_source(self.wgsl).map(drop)
    }
}
