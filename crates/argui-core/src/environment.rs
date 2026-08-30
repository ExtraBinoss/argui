/// Effective color scheme used to render a window.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}
