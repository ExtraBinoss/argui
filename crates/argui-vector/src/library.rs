use argui_paint::{VectorAsset, VectorId};

use crate::{VectorError, parse_svg};

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

    #[must_use]
    pub fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}
