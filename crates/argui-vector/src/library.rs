use argui_core::Size;
use argui_paint::{VectorAsset, VectorId};

use crate::{PathCommand, PathError, PathStyle, VectorError, parse_svg, path_asset};

/// Owns immutable vector assets and generates their stable handles.
#[derive(Clone, Debug, Default)]
pub struct VectorLibrary {
    assets: Vec<VectorAsset>,
}

impl VectorLibrary {
    /// Creates an empty vector asset library.
    #[must_use]
    pub const fn new() -> Self {
        Self { assets: Vec::new() }
    }

    /// Parses and stores an SVG asset, returning its generated identifier.
    ///
    /// # Arguments
    /// * `svg` — SVG document bytes.
    ///
    /// # Errors
    /// Returns an error if the SVG cannot be parsed.
    pub fn insert_svg(&mut self, svg: &[u8]) -> Result<VectorId, VectorError> {
        let id = VectorId::fresh();
        self.assets.push(parse_svg(id, svg)?);
        Ok(id)
    }

    /// Builds and stores a tintable path, returning its generated identifier.
    ///
    /// `size` defines the path's local view box, `commands` describes its
    /// geometry, and `style` selects fill and stroke paint.
    ///
    /// # Errors
    /// Returns [`PathError`] if geometry or paint is invalid or SVG parsing fails.
    pub fn insert_path(
        &mut self,
        size: Size,
        commands: &[PathCommand],
        style: PathStyle,
    ) -> Result<VectorId, PathError> {
        let id = VectorId::fresh();
        self.assets.push(path_asset(id, size, commands, style)?);
        Ok(id)
    }

    #[must_use]
    /// Returns the vector assets currently owned by this library.
    pub fn assets(&self) -> &[VectorAsset] {
        &self.assets
    }
}
