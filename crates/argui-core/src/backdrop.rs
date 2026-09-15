/// Semantic materials for an operating-system background effect.
/// The compositor controls the blur kernel and may use the same material for all variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BackdropMaterial {
    /// General translucent glass material.
    #[default]
    Glass,
    /// Material intended for sidebar surfaces.
    Sidebar,
    /// Material intended for header surfaces.
    Header,
}
