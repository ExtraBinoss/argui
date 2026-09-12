/// Semantic materials for an operating-system background effect.
/// The compositor controls the blur kernel and may use the same material for all variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BackdropMaterial {
    #[default]
    Glass,
    Sidebar,
    Header,
}
