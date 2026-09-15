/// Effective color scheme used to render a window.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorScheme {
    /// Light color palette.
    #[default]
    Light,
    /// Dark color palette.
    Dark,
}
