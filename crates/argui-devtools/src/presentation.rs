#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Presentation location for the DevTools panel.
pub enum DockMode {
    #[default]
    Bottom,
    Right,
    Detached,
}

pub(crate) fn dock_options(detached: bool) -> [argui_widgets::SelectOption; 3] {
    [
        argui_widgets::SelectOption::new("Bottom"),
        argui_widgets::SelectOption::new("Right"),
        argui_widgets::SelectOption::new("Separate window").enabled(detached),
    ]
}
