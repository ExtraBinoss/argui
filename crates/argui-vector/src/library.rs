use argui_paint::{VectorAsset, VectorId};

use crate::{VectorError, morph_svg, parse_svg};

/// Owns immutable vector assets and generates their stable handles.
#[derive(Clone, Debug, Default)]
pub struct VectorLibrary {
    assets: Vec<VectorAsset>,
}

impl VectorLibrary {
    #[must_use]
    pub const fn new() -> Self {
        Self { assets: Vec::new() }
    }

    pub fn insert_svg(&mut self, svg: &[u8]) -> Result<VectorId, VectorError> {
        let id = VectorId::fresh();
        self.assets.push(parse_svg(id, svg)?);
        Ok(id)
    }

    pub fn insert_morph(&mut self, from: &[u8], to: &[u8]) -> Result<VectorId, VectorError> {
        let id = VectorId::fresh();
        self.assets.push(morph_svg(id, from, to)?);
        Ok(id)
    }

    #[must_use]
    pub fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}
